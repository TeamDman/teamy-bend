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
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-scheduler-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn run(&self, source: &str, foreign: &str) -> Output {
        fs::write(self.0.join("main.bend"), source).unwrap();
        fs::write(self.0.join("effect.js"), foreign).unwrap();
        let source = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&source).unwrap();
        let javascript = compile_executable_javascript(&checked).unwrap();
        fs::write(self.0.join("main.cjs"), javascript).unwrap();
        Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
            .arg(self.0.join("main.cjs"))
            .output()
            .expect("scheduler tests require Node.js")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn success(output: &Output, expected: &str) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected.as_bytes());
    assert!(output.stderr.is_empty());
}

#[test]
fn children_outlive_main_and_runnable_tasks_precede_zero_timers() {
    let output = Fixture::new().run(
        r#"import Base
def late(tag: String) -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.sleep(0)
    Unit <- IO.sleep(0)
    IO.print(tag)
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, late("a"))
    Unit <- IO.spawn(Unit, late("b"))
    Unit <- IO.spawn(Unit, IO.print("ready"))
    IO.print("main")
"#,
        "",
    );
    success(&output, "main\nready\na\nb\n");
}

#[test]
fn saved_continuation_resumes_after_another_task_runs() {
    let output = Fixture::new().run(
        r#"import Base
def park() -> IO(U32): import "effect.js"
def release() -> IO(Unit): import "effect.js"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, release())
    answer : U32 <- park()
    IO.print(U32.show(answer))
"#,
        "let saved; function park(k){if(arguments.length!==1||k!==this.kont)throw Error('continuation ABI');saved=k;} function release(){if(!saved)throw Error('task ran before main suspended');io_push(saved,42,false);saved=null;return {$:'Unit'};}",
    );
    success(&output, "42\n");
}

#[test]
fn timer_hook_is_nullary_and_its_run_can_suspend_into_the_queue() {
    let output = Fixture::new().run(
        r#"import Base
def delayed(ms: U32) -> IO(U32): import "effect.js"
def main() -> IO(Unit):
  do IO<Unit>:
    answer : U32 <- delayed(0)
    IO.print(U32.show(answer))
"#,
        "let state=0;function delayed_need(){if(arguments.length!==0||this.args[0]!==0||state!==0)throw Error('need ABI');state=1;return {time:true};}function delayed(ms,k){if(state!==1||this.kont!==k||arguments.length!==2)throw Error('run ABI');state=2;io_push(k,43,false);}",
    );
    success(&output, "43\n");
}

#[test]
fn monotonic_now_observes_the_sleep_lower_bound() {
    let output = Fixture::new().run(
        r#"import Base
def report(early: Bool) -> String:
  match early:
    case False{}: "ok"
    case True{}: "early"
def main() -> IO(Unit):
  do IO<Unit>:
    before : Nat <- IO.now()
    Unit <- IO.sleep(15)
    after : Nat <- IO.now()
    IO.print(report(Nat.is_lt(Nat.sub(after, before), 15n)))
"#,
        "",
    );
    success(&output, "ok\n");
}

#[test]
fn halt_from_main_or_a_child_cancels_pending_tasks_and_timers() {
    let fixture = Fixture::new();
    let helpers = r#"import Base
def late() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.sleep(1)
    IO.print("LATE")
"#;
    for main in [
        r#"def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, late())
    IO.die(Unit, 7, "stop")
"#,
        r#"def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, IO.die(Unit, 7, "stop"))
    Unit <- IO.sleep(20)
    IO.print("AFTER")
"#,
    ] {
        let output = fixture.run(&format!("{helpers}{main}"), "");
        assert_eq!(output.status.code(), Some(7));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, b"stop\n");
    }
}

#[test]
fn unresumed_suspension_reports_deadlock_without_calling_the_continuation() {
    let output = Fixture::new().run(
        "import Base\ndef park() -> IO(Unit): import \"effect.js\"\ndef main() -> IO(Unit):\n  do IO<Unit>:\n    Unit <- park()\n    IO.print(\"AFTER\")\n",
        "function park(){}",
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("deadlock"));
}

#[test]
fn promise_readiness_and_unbranded_request_values_fail_before_following_effects() {
    let fixture = Fixture::new();
    let source = "import Base\ndef effect() -> IO(Unit): import \"effect.js\"\ndef main() -> IO(Unit):\n  do IO<Unit>:\n    Unit <- effect()\n    IO.print(\"AFTER\")\n";
    for (foreign, diagnostic) in [
        (
            "function effect(){return Promise.resolve({$:'Unit'});}",
            "asynchronous foreign results",
        ),
        (
            "function effect_need(){return {read:true};}function effect(){process.stdout.write('BAD');}",
            "readiness scheduling",
        ),
        (
            "function effect(){io_park_on(0,false,()=>0,()=>0);}",
            "readiness scheduling",
        ),
    ] {
        let output = fixture.run(source, foreign);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains(diagnostic));
    }
    let output = fixture.run(
        "import Base\ndef forged() -> IO(IO(Unit)): import \"effect.js\"\ndef main() -> IO(Unit):\n  do IO<Unit>:\n    action : IO(Unit) <- forged()\n    action\n",
        "function forged(){return k=>({$:'$FFI',run(){process.stdout.write('BAD');return {$:'Unit'}},args:[],kont:k});}",
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("neither an operation nor a foreign request")
    );
}

#[test]
fn foreign_lexical_names_cannot_replace_sealed_task_and_clock_implementations() {
    let output = Fixture::new().run(
        r#"import Base
def probe() -> IO(Unit): import "effect.js"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- probe()
    Unit <- IO.spawn(Unit, IO.print("child"))
    Unit <- IO.sleep(0)
    now : Nat <- IO.now()
    IO.print("main")
"#,
        "function probe(){return {$:'Unit'}} function $tbSpawn(){throw Error('SHADOW')}function $tbSleepNeed(){throw Error('SHADOW')}function $tbNow(){throw Error('SHADOW')}function io_push(){throw Error('SHADOW')}",
    );
    success(&output, "child\nmain\n");
}

#[test]
fn queued_resumptions_share_the_step_budget_and_pending_queue_is_bounded() {
    let fixture = Fixture::new();
    let source = "import Base\ndef effect() -> IO(Unit): import \"effect.js\"\ndef main() -> IO(Unit): effect()\n";
    for (foreign, diagnostic) in [
        (
            "function again(){io_push(again,null,false)}function effect(){io_push(again,null,false)}",
            "step budget exhausted",
        ),
        (
            "function effect(){for(let i=0;i<=131072;i++)io_push(()=>undefined,null,false)}",
            "pending task budget exhausted",
        ),
    ] {
        let output = fixture.run(source, foreign);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains(diagnostic));
    }
}
