// SPDX-License-Identifier: MPL-2.0
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str, foreign: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-foreign-compiled-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("main.bend"), source).unwrap();
        fs::write(path.join("effect.js"), foreign).unwrap();
        Self(path)
    }

    fn run(&self) -> Output {
        let source = load_executable(self.0.join("main.bend")).expect("load source");
        let checked = check_executable(&source).expect("check executable");
        let javascript = compile_executable_javascript(&checked).expect("compile executable");
        fs::write(self.0.join("main.cjs"), javascript).unwrap();
        Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
            .arg(self.0.join("main.cjs"))
            .output()
            .expect("compiled foreign tests require Node.js")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn erased_parameters_are_omitted_and_live_types_and_proofs_are_null() {
    let output = Fixture::new(
        r#"import Base
def inspect(-A: Type, live: Type, proof: {0 == 0 : U32}, x: U32) -> IO(U32):
  import "./effect.js"
def main() -> IO(Unit):
  do IO<Unit>:
    value : U32 <- inspect(U32, Nat, {==}, 7)
    IO.print(U32.show(value))
"#,
        "function inspect(live,proof,x,k){if(arguments.length!==4||live!==null||proof!==null||x!==7||typeof k!=='function')throw Error('wrong foreign argument slots');return x;}",
    ).run();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"7\n");
}

#[test]
fn foreign_callbacks_complete_bend_tail_calls() {
    let output = Fixture::new(
        r#"import Base
def callback(f: U32 -> U32, x: U32) -> IO(U32):
  import "./effect.js"
def plus_one(x: U32) -> U32: U32.add(x, 1)
def main() -> IO(Unit):
  do IO<Unit>:
    value : U32 <- callback(plus_one, 41)
    IO.print(U32.show(value))
"#,
        "function callback(f,x){return f(x);}",
    )
    .run();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"42\n");
}

#[test]
fn matching_a_request_fails_before_its_foreign_implementation_runs() {
    for cases in [
        "    case Emit{x}: IO.pure(Unit, Unit{})\n    case Halt{code, message}: IO.print(message)\n",
        "    case Emit{x}: IO.pure(Unit, Unit{})\n    case _: IO.print(\"FALLBACK\")\n",
    ] {
        let source = format!(
            "import Base\ndef secret() -> IO(Unit):\n  import \"./effect.js\"\ndef inspect(op: IO.OP<U32>) -> IO(Unit):\n  match op:\n{cases}def main() -> IO(Unit): inspect(secret()(U32, x => Emit{{7}}))\n"
        );
        let output = Fixture::new(
            &source,
            "function secret(){process.stdout.write('SECRET');return {$:'Unit'};}",
        )
        .run();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("request"));
    }
}

#[test]
fn undefined_foreign_return_without_a_resumer_reports_deadlock() {
    let output = Fixture::new(
        "import Base\ndef missing() -> IO(U32):\n  import \"./effect.js\"\ndef main() -> IO(Unit):\n  do IO<Unit>:\n    value : U32 <- missing()\n    IO.print(\"AFTER\")\n",
        "function missing(){}",
    ).run();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("deadlock"));
}
