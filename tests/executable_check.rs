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
fn source_holes_cannot_escape_through_erasure_or_unreachable_code() {
    for source_text in [
        "import Base\ndef main() -> U32: (x => {0 : U32})(?unused)\n",
        "import Base\ndef main() -> U32: (x => {0 : U32})(?TODO)\n",
        "import Base\ndef bad() -> U32: ?unreferenced\ndef main() -> U32: 0\n",
        "import Base\ndef bad(b: Bool) -> U32:\n  match b:\n    case True{}: 0\n    case False{}: ?branch\ndef main() -> U32: bad(True{})\n",
    ] {
        let fixture = Fixture::new();
        let path = fixture.write("main.bend", source_text);
        let strict = load(&path).unwrap();
        let input = load_executable(&path).unwrap();
        for error in [
            check_book(&strict).unwrap_err(),
            check_executable(&input).unwrap_err(),
        ] {
            assert!(
                error.to_string().contains("unfinished proof hole"),
                "{error}"
            );
        }
    }
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

    let foreign = executable("import Base\ndef main() -> IO(Unit):\n  import \"foreign.js\"\n");
    let error = foreign.entry_kind().expect_err("invalid foreign entry");
    assert!(error.to_string().contains("foreign main"), "{error}");
}

#[test]
fn parameterized_mains_are_values_even_when_their_result_is_io() {
    for body in [
        "import Base\ndef main(n: Nat) -> Nat: n\n",
        "import Base\ndef main(u: Unit) -> IO(Unit): IO.print(\"effect\")\n",
    ] {
        let checked = executable(body);
        assert_eq!(checked.entry_kind().unwrap(), ExecutableEntry::Pure);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(
            checked
                .run_main(&mut stdout, &mut stderr, &|| false)
                .unwrap(),
            0
        );
        let text = String::from_utf8(stdout).unwrap();
        assert!(
            text.starts_with("n => n\n") || text.starts_with("u => IO.print("),
            "{text}"
        );
        assert_ne!(text, "effect\n", "a returned IO function must not run");
        assert!(stderr.is_empty());
    }
}

#[test]
fn unsafe_function_normalization_keeps_limits_and_emits_no_partial_value() {
    for (body, refusal) in [
        ("main(n)", "kernel evaluation budget exhausted"),
        ("1n + main(n)", "kernel nesting limit exhausted"),
    ] {
        let checked = executable(&format!(
            "import Base\n@unsafe\ndef main(n: Nat) -> Nat: {body}\n"
        ));
        for cancelled in [false, true] {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let error = checked
                .run_main(&mut stdout, &mut stderr, &|| cancelled)
                .unwrap_err();
            let expected = if cancelled { "cancelled" } else { refusal };
            assert!(error.to_string().contains(expected), "{error}");
            assert!(stdout.is_empty() && stderr.is_empty());
        }
    }
}

#[test]
fn missing_and_pure_entries_have_separate_output_without_effects() {
    for (body, expected) in [
        ("", "All terms check.\n"),
        ("import Base\ndef main() -> Nat: 2n\n", "2n\n"),
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
