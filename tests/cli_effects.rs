// SPDX-License-Identifier: MPL-2.0
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-effects-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).unwrap();
        std::fs::write(directory.join("main.bend"), source).unwrap();
        Self(directory)
    }

    fn invoke(&self, command: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .args(["--output-format", "json", command])
            .arg(self.0.join("main.bend"))
            .output()
            .expect("execute native CLI")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn console_output_is_raw_utf8_and_emit_payload_does_not_set_exit_status() {
    let fixture = Fixture::new(
        r#"
import Base
def main() -> IO(U32):
  do IO<U32>:
    a : Unit <- IO.write("é\0")
    b : Unit <- IO.print(U32.show(42))
    c : Unit <- IO.print_err("err❁")
    return 7
"#,
    );
    let output = fixture.invoke("run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, "é\u{0}42\n".as_bytes());
    assert_eq!(output.stderr, "err❁\n".as_bytes());
    let proof_check = fixture.invoke("check");
    assert!(!proof_check.status.success());
    assert!(proof_check.stdout.is_empty());
}

#[test]
fn halt_preserves_status_writes_once_and_stops_following_effects() {
    let fixture = Fixture::new(
        r#"
import Base
def main() -> IO(Unit):
  do IO<Unit>:
    a : Unit <- IO.write("before")
    b : Unit <- IO.die(Unit, 7, "è❁")
    IO.print("AFTER")
"#,
    );
    let output = fixture.invoke("run");
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, b"before");
    assert_eq!(output.stderr, "è❁\n".as_bytes());

    let large =
        Fixture::new("import Base\ndef main() -> IO(Unit): IO.die(Unit, 287454020, \"status\")\n");
    let output = large.invoke("run");
    let expected = if cfg!(windows) { 287_454_020 } else { 68 };
    assert_eq!(output.status.code(), Some(expected));
    assert_eq!(output.stderr, b"status\n");
}

#[test]
fn main_aliases_run_effects_but_a_user_io_name_remains_pure() {
    let alias = Fixture::new(
        r#"
import Base
def Program(-A: Type) -> Type: IO(A)
def Entry() -> Type: Program(Unit)
def main() -> Entry: IO.print("alias")
"#,
    );
    let output = alias.invoke("run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"alias\n");
    assert!(output.stderr.is_empty());

    let forged = Fixture::new(
        "type Box is Data: Full{}\ndef IO(-A: Type) -> Type: Box\ndef main() -> IO(Box): Full{}\n",
    );
    let output = forged.invoke("run");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"Full{}\n");

    let no_main = Fixture::new("type Box is Data: Full{}\ndef value() -> Box: Full{}\n");
    let output = no_main.invoke("run");
    assert!(output.status.success());
    assert_eq!(output.stdout, b"All terms check.\n");
}

#[test]
fn matching_an_effect_request_never_runs_the_requested_effect() {
    let fixture = Fixture::new(
        r#"
import Base
def inspect(op: IO.OP<Unit>) -> IO(Unit):
  match op:
    case Emit{value}: IO.print("EMITTED")
    case Halt{code, message}: IO.print(message)
def main() -> IO(Unit):
  inspect(IO.print("SECRET")(Unit, value => Emit{value}))
"#,
    );
    let output = fixture.invoke("run");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("request"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("SECRET"));
}

#[test]
fn invalid_proofs_and_foreign_main_fail_before_any_effect() {
    for source in [
        "import Base\nlaw wrong: {0n == 1n : Nat}\ndef wrong(): {==}\ndef main() -> IO(Unit): IO.print(\"BAD\")\n",
        "import Base\ndef main() -> IO(Unit):\n  import \"./effect.js\"\n",
    ] {
        let output = Fixture::new(source).invoke("run");
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn arbitrary_foreign_contracts_fail_explicitly_on_the_native_backend() {
    let fixture = Fixture::new(
        r#"
import Base
def external() -> IO(Unit):
  import "./effect.js"
def main() -> IO(Unit): external()
"#,
    );
    let output = fixture.invoke("run");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("foreign") && error.contains("external"),
        "{error}"
    );
}
