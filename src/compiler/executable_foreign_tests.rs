// SPDX-License-Identifier: MPL-2.0
use super::DRIVER;
use super::FILES;
use super::ForeignAssembly;
use super::assemble;
use crate::kernel::check_executable;
use crate::syntax::load_executable;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static NEXT: AtomicUsize = AtomicUsize::new(0);
const CORE: &str = "function $tbTick(){}\nfunction $tbForceCall(f,args){if(args.length===0)return f();for(const a of args)f=f(a);return f;}\n";

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-foreign-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, source: &str) {
        fs::write(self.0.join(name), source).unwrap();
    }

    fn assembly(&self, source: &str) -> Result<ForeignAssembly, super::CompileError> {
        self.write("main.bend", source);
        let source = load_executable(self.0.join("main.bend")).expect("load fixture");
        let book = check_executable(&source).expect("check fixture");
        assemble(&book.lower_for_javascript().expect("lower fixture"))
    }

    fn run(&self, source: &str) -> Output {
        self.write("main.cjs", source);
        Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
            .arg(self.0.join("main.cjs"))
            .output()
            .expect("foreign runtime tests require Node.js")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn driver(foreign: &str, main: &str) -> Output {
    Fixture::new().run(&format!("{CORE}\n{DRIVER}\n{FILES}\nconst $tbForeign={foreign};\nprocess.exitCode=$tbRunMain({main},true,null);\n"))
}

#[test]
fn canonical_sources_share_one_scope_and_resolve_local_symbols() {
    let fixture = Fixture::new();
    fixture.write("effect.js", "let count=0;function tick_bump(){return ++count}function tick_peek(){return count}// final comment");
    fixture.write("module.bend", "import Base\ndef Tick.bump() -> IO(U32):\n  import \"./effect.js\"\ndef Tick.peek() -> IO(U32):\n  import \"././effect.js\"\n");
    let assembly = fixture.assembly("import Base\nimport ./module.bend as M\ndef main() -> IO(U32):\n  do IO<U32>:\n    a : U32 <- M.Tick.bump()\n    M.Tick.peek()\n").unwrap();
    assert_eq!(assembly.source.matches("let count=0").count(), 1);
    let bump = assembly.indices["module.Tick.bump"];
    let peek = assembly.indices["module.Tick.peek"];
    let output = fixture.run(&format!("{CORE}\n{}\nprocess.stdout.write(JSON.stringify([$tbForeign[{bump}].run(),$tbForeign[{bump}].run(),$tbForeign[{peek}].run()]));", assembly.source));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"[1,2,2]");
}

#[test]
fn first_javascript_import_is_selected_without_fallback() {
    let fixture = Fixture::new();
    fixture.write("good.js", "function effect(){return 1}");
    let source = "import Base\ndef effect() -> IO(U32):\n  import \"./unused.c\"\n  import \"./missing.js\"\n  import \"./good.js\"\ndef main() -> IO(U32): effect()\n";
    let error = fixture
        .assembly(source)
        .err()
        .expect("missing first import");
    assert!(
        error
            .to_string()
            .contains("cannot resolve JavaScript import")
    );
    let error = fixture.assembly("import Base\ndef effect() -> IO(U32):\n  import \"./unused.c\"\ndef main() -> IO(U32): effect()\n").err().expect("no JS import");
    assert!(error.to_string().contains("has no JavaScript import"));
}

#[test]
fn foreign_initializers_follow_reference_discovery_order_not_alphabetical_names() {
    let fixture = Fixture::new();
    fixture.write("zebra.js", "let order=['zebra'];function zebra(){return 1}");
    fixture.write(
        "alpha.js",
        "order.push('alpha');function alpha(){return order.join(',')}",
    );
    let assembly = fixture.assembly("import Base\ndef zebra() -> IO(U32):\n  import \"./zebra.js\"\ndef alpha() -> IO(String):\n  import \"./alpha.js\"\ndef main() -> IO(String):\n  do IO<String>:\n    a : U32 <- zebra()\n    alpha()\n").unwrap();
    let alpha = assembly.indices["alpha"];
    let output = fixture.run(&format!(
        "{CORE}\n{}\nprocess.stdout.write($tbForeign[{alpha}].run());",
        assembly.source
    ));
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"zebra,alpha");
}

#[test]
fn foreign_lexical_names_cannot_replace_bundled_console() {
    let fixture = Fixture::new();
    fixture.write("effect.js", "function effect(){return 1} function $tbPrint(){throw Error('SHADOW')} function $tbOutput(){throw Error('SHADOW')}");
    let assembly = fixture.assembly("import Base\ndef effect() -> IO(U32):\n  import \"./effect.js\"\ndef main() -> IO(Unit):\n  do IO<Unit>:\n    a : U32 <- effect()\n    IO.print(\"hello\")\n").unwrap();
    let print = assembly.indices["IO.print"];
    let output = fixture.run(&format!(
        "{CORE}\n{}\n$tbForeign[{print}].run('hello');",
        assembly.source
    ));
    assert!(output.status.success());
    assert_eq!(output.stdout, b"hello\n");
}

#[test]
fn driver_preserves_bytes_callback_values_and_discards_emit_payload() {
    let output = driver(
        "[{run:(callback,k)=>callback(7),need:undefined},{run:$tbPrint,need:undefined}]",
        "()=>k=>$tbMakeRequest(0,[x=>x+1],value=>$tbMakeRequest(1,['é\\0🙂'+value],_=>k(7)))",
    );
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, "é\0🙂8\n".as_bytes());
    assert!(output.stderr.is_empty());
}

#[test]
fn halt_preserves_order_status_and_stops_following_effects() {
    let output = driver(
        "[{run:$tbWrite,need:undefined}]",
        "()=>k=>$tbMakeRequest(0,['before'],_=>({$:'Halt',code:7,message:'stop\\0🙂'}))",
    );
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, b"before");
    assert_eq!(output.stderr, "stop\0🙂\n".as_bytes());
}

#[test]
fn unsupported_scheduling_fails_explicitly() {
    for (foreign, message) in [
        ("[{run:()=>undefined}]", "deadlock"),
        (
            "[{run:()=>Promise.resolve(7)}]",
            "asynchronous foreign results",
        ),
        (
            "[{run:()=>{process.stdout.write('BAD');return 1},need:()=>({read:true})}]",
            "readiness scheduling is not supported",
        ),
        ("[{}]", "foreign implementation is missing"),
    ] {
        let output = driver(foreign, "()=>k=>$tbMakeRequest(0,[],k)");
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains(message));
    }
}

#[test]
fn request_lookalikes_are_data_and_thrown_requests_fail_without_effects() {
    let output = driver(
        "[{run:()=>{process.stdout.write('BAD');return 1}}]",
        "()=>k=>{throw $tbMakeRequest(0,[],k)}",
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("request inspected outside"));
    let output = driver("[]", "()=>k=>k({index:0,args:[],continuation:k})");
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn raw_foreign_returns_keep_host_representation() {
    let output = driver(
        "[{run:()=>({native:[5n,true,null,'a\\uD800']})},{run:$tbPrint}]",
        "()=>k=>$tbMakeRequest(0,[],x=>$tbMakeRequest(1,[String(x.native[0])+x.native[1]+x.native[2]+x.native[3]],k))",
    );
    assert!(output.status.success());
    assert_eq!(output.stdout, "5truenulla�\n".as_bytes());
}

#[test]
fn foreign_calls_receive_declared_arguments_then_continuation_and_request_receiver() {
    let output = driver(
        "[{run:function implementation(a,b,k){if(arguments.length!==3||a!==7||b!==null||k!==this.kont||this.args[0]!==7||this.run!==implementation)throw Error('calling convention');return 1}}]",
        "()=>k=>$tbMakeRequest(0,[7,null],k)",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn pure_and_missing_entries_use_raw_text() {
    for (main, printer, expected) in [
        ("null", "null", "All terms check.\n"),
        ("()=>7", "x=>'value='+x", "value=7\n"),
    ] {
        let output = Fixture::new().run(&format!("{CORE}\n{DRIVER}\nconst $tbForeign=[];process.exitCode=$tbRunMain({main},false,{printer});"));
        assert!(output.status.success());
        assert_eq!(output.stdout, expected.as_bytes());
    }
}

#[test]
fn writes_retry_interrupts_complete_partial_writes_and_reject_zero() {
    let prefix = "const actualRequire=require;let calls=0;let captured='';const mock={writeSync(fd,buf,offset,len){calls++;if(calls===1)throw {code:'EINTR'};if(typeof buf==='string'){captured+=buf;return buf.length}captured+=Buffer.from(buf.subarray(offset,offset+1)).toString('latin1');return 1}};require=name=>name==='fs'?mock:actualRequire(name);";
    let output = Fixture::new().run(&format!("{prefix}\n{CORE}\n{DRIVER}\nconst $tbForeign=[];const code=$tbRunMain(()=>7,false,String);process.stdout.write(JSON.stringify({{code,captured,calls}}));"));
    assert!(output.status.success());
    assert_eq!(output.stdout, br#"{"code":0,"captured":"7\n","calls":3}"#);
    let prefix = "const actualRequire=require;const mock={writeSync(){return 0}};require=name=>name==='fs'?mock:actualRequire(name);";
    let output = Fixture::new().run(&format!("{prefix}\n{CORE}\n{DRIVER}\nconst $tbForeign=[];process.exitCode=$tbRunMain(()=>7,false,String);"));
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn console_byte_budget_is_checked_before_encoding_or_writing() {
    let output = driver(
        "[{run:$tbWrite}]",
        "()=>k=>$tbMakeRequest(0,['é'.repeat(4194305)],k)",
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("console output byte budget exhausted")
    );
}

#[test]
fn synchronous_companion_helpers_preserve_bytes_and_native_records() {
    let output = driver(
        "[{run:()=>{const bytes=io_bytes('é\\0🙂');io_out(1,bytes);io_errs(io_text(bytes,bytes.length));return io_done(io_tup(1,2,3))}},{run:$tbPrint}]",
        "()=>k=>$tbMakeRequest(0,[],x=>$tbMakeRequest(1,[JSON.stringify(x)],k))",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, "é\0🙂{\"$\":\"Done\",\"value\":{\"$\":\"Tuple\",\"fst\":1,\"snd\":{\"$\":\"Tuple\",\"fst\":2,\"snd\":3}}}\n".as_bytes());
    assert_eq!(output.stderr, "é\0🙂\n".as_bytes());
    for (call, expected) in [
        ("io_push()", "task scheduling"),
        ("io_park_on()", "readiness scheduling"),
    ] {
        let output = driver(
            &format!("[{{run:()=>{call}}}]"),
            "()=>k=>$tbMakeRequest(0,[],k)",
        );
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains(expected));
    }
}
