// SPDX-License-Identifier: MPL-2.0
//! Executable contracts must never become strict proof certificates.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::kernel::ExecutableBook;
use teamy_bend::kernel::ExecutableEntry;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::ExecutableSource;
use teamy_bend::syntax::load;
use teamy_bend::syntax::load_executable;
use teamy_bend::syntax::parse;

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-executable-check-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("create fixture directory");
        Self(path)
    }

    fn write(&self, name: &str, body: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, body).expect("write source fixture");
        path
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn source(body: &str) -> ExecutableSource {
    let fixture = Fixture::new();
    load_executable(fixture.write("main.bend", body)).expect("executable source parses")
}

fn executable(body: &str) -> ExecutableBook {
    check_executable(&source(body)).expect("executable contract checks")
}

#[test]
fn foreign_results_are_runtime_assumptions_including_proof_and_type_values() {
    let program = executable(
        r#"
import Base
def false_proof() -> IO({0n == 1n : Nat}):
  import "assume.js"
def foreign_type() -> IO(Type):
  import "assume.js"
def action() -> IO({0n == 1n : Nat}): false_proof()
"#,
    );
    assert!(program.foreign_names().any(|name| name == "false_proof"));
    assert!(program.foreign_names().any(|name| name == "foreign_type"));
    assert!(program.definition_type("action").is_some());
    assert_eq!(program.entry_kind().unwrap(), ExecutableEntry::Missing);

    let error = check_executable(&source(
        r#"
import Base
def false_proof() -> IO({0n == 1n : Nat}):
  import "assume.js"
law wrong: {0n == 1n : Nat}
def wrong(): {==}
"#,
    ))
    .expect_err("foreign contracts cannot make false ordinary proofs valid");
    assert!(error.to_string().contains("reflexivity"), "{error}");
}

#[test]
fn foreign_return_gate_requires_direct_actual_base_io() {
    for body in [
        "import Base\ndef bad() -> Nat:\n  import \"value.js\"\n",
        "import Base\ndef Alias(-A: Type) -> Type: IO(A)\ndef bad() -> Alias(Nat):\n  import \"value.js\"\n",
        "type IO<-A: Type> is Type:\n  Fake{value: A}\ntype Unit is Data:\n  Unit{}\ndef bad() -> IO<Unit>:\n  import \"value.js\"\n",
    ] {
        let error = check_executable(&source(body)).expect_err("foreign gate must reject");
        assert!(error.to_string().contains("actual Base IO"), "{error}");
    }
}

#[test]
fn executable_signatures_keep_erased_arguments_and_resource_type_checks() {
    let program = executable(
        r#"
import Base
law foreign_identity:
  @-A: Type -> @x: A -> IO(A)
def foreign_identity(A, x):
  import "identity.js"
def main() -> IO(Nat): foreign_identity(Nat, 1n)
"#,
    );
    assert_eq!(program.entry_kind().unwrap(), ExecutableEntry::Io);
    for (body, expected) in [
        (
            "import Base\ndef foreign_identity(-A: Type, x: A) -> IO(A):\n  import \"identity.js\"\ndef main() -> IO(Nat): foreign_identity(Bool, 1n)\n",
            "remaining constructor",
        ),
        (
            "import Base\ntype Ticket is Type:\n  Ticket{}\ndef bad(+ticket: Ticket) -> IO(Unit):\n  import \"identity.js\"\n",
            "type mismatch",
        ),
        (
            "import Base\ndef bad(x: ?unfinished) -> IO(Unit):\n  import \"identity.js\"\n",
            "unfinished proof hole",
        ),
    ] {
        let error = check_executable(&source(body)).expect_err("invalid foreign signature/use");
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn ordinary_laws_templates_and_source_order_remain_strict_in_executables() {
    for (body, expected) in [
        ("import Base\nlaw missing: Nat\n", "unfilled laws"),
        (
            "import Base\nlaw later: Nat\ndef early() -> Nat: later\ndef later(): 0n\n",
            "cannot be used as live evidence",
        ),
        (
            "import Base\ndef bad(~A: Type, x: A) -> Nat: True{}\ndef main() -> Nat: bad(~Nat, 0n)\n",
            "remaining constructor",
        ),
        (
            "import Base\ndef loop(n: Nat) -> Nat: loop(n)\n",
            "decrease structurally",
        ),
    ] {
        let error =
            check_executable(&source(body)).expect_err("ordinary checking must remain strict");
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn entry_detection_unfolds_aliases_and_ignores_forged_io_names() {
    let alias = executable(
        "import Base\ndef Action(-A: Type) -> Type: IO(A)\ndef main() -> Action(Unit): IO.pure(Unit, Unit{})\n",
    );
    assert_eq!(alias.entry_kind().unwrap(), ExecutableEntry::Io);
    let forged = executable("type IO is Data:\n  Forged{}\ndef main() -> IO: Forged{}\n");
    assert_eq!(forged.entry_kind().unwrap(), ExecutableEntry::Pure);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        forged
            .run_main(&mut stdout, &mut stderr, &|| false)
            .unwrap(),
        0
    );
    assert_eq!(stdout, b"Forged{}\n");
    assert!(stderr.is_empty());

    for (body, expected) in [
        (
            "import Base\ndef main() -> IO(Unit):\n  import \"foreign.js\"\n",
            "foreign main",
        ),
        (
            "import Base\ndef main(n: Nat) -> Nat: n\n",
            "no declared parameters",
        ),
    ] {
        let error = executable(body).entry_kind().expect_err("invalid entry");
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn missing_and_pure_entries_have_separate_output_without_effects() {
    for (body, expected) in [
        ("", "All terms check.\n"),
        (
            "import Base\ndef main() -> Nat: 2n\n",
            "Succ{Succ{Zero{}}}\n",
        ),
    ] {
        let program = executable(body);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(
            program
                .run_main(&mut stdout, &mut stderr, &|| false)
                .unwrap(),
            0
        );
        assert_eq!(stdout, expected.as_bytes());
        assert!(stderr.is_empty());
    }
}

#[test]
fn strict_loading_and_proof_checking_never_accept_foreign_flags() {
    let fixture = Fixture::new();
    let path = fixture.write(
        "foreign.bend",
        "import Base\ndef foreign() -> IO(Unit):\n  import \"foreign.js\"\n",
    );
    load(path).expect_err("strict loader rejects foreign source");
    let mut book = parse("type Unit is Data:\n  Unit{}\ndef value() -> Unit: Unit{}\n").unwrap();
    let teamy_bend::kernel::Declaration::Def(definition) = &mut book.declarations[1] else {
        panic!("test definition");
    };
    definition.foreign = true;
    definition.body = None;
    let error = check_book(&book).expect_err("public raw AST cannot bypass strict checking");
    assert!(
        error.to_string().contains("strict proof checking rejects"),
        "{error}"
    );
}

#[test]
fn foreign_refills_are_rejected() {
    let fixture = Fixture::new();
    let path = fixture.write(
        "invalid.bend",
        "import Base\ndef foreign() -> IO(Unit):\n  import \"foreign.js\"\ndef foreign(): IO.pure(Unit, Unit{})\n",
    );
    match load_executable(path) {
        Ok(parsed) => {
            check_executable(&parsed).expect_err("foreign declarations cannot be refilled");
        }
        Err(error) => assert!(error.message.contains("duplicate"), "{error}"),
    }
}
