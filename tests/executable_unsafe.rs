// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel::ExecutableBook;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-executable-unsafe-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("main.bend"), source).unwrap();
        Self(directory)
    }

    fn checked(&self) -> ExecutableBook {
        let source = load_executable(self.0.join("main.bend")).unwrap();
        check_executable(&source).unwrap()
    }

    fn rejection(&self) -> String {
        let source = load_executable(self.0.join("main.bend")).unwrap();
        check_executable(&source).unwrap_err().to_string()
    }

    fn cli(&self, command: &str, arguments: &[&str]) -> Output {
        executable_c_compiler::bounded(
            Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
                .current_dir(&self.0)
                .args([command, "main.bend"])
                .args(arguments)
                .stdin(Stdio::null()),
            Duration::from_secs(20),
        )
    }

    fn javascript(&self, checked: &ExecutableBook) -> Output {
        fs::write(
            self.0.join("main.cjs"),
            compile_executable_javascript(checked).unwrap(),
        )
        .unwrap();
        executable_c_compiler::bounded(
            Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
                .current_dir(&self.0)
                .arg("main.cjs"),
            Duration::from_secs(20),
        )
    }

    fn native_c(&self, checked: &ExecutableBook, definitions: &[&str]) -> Output {
        let generated = compile_executable_c(checked).unwrap();
        assert_eq!(
            generated
                .matches("static int tb_program_main(void)")
                .count(),
            1
        );
        let mut generated = generated.replace(
            "static int tb_program_main(void)",
            "#define TB_NO_MAIN 1\nstatic int checked_main(void)",
        );
        generated.push_str(
            r#"
int main(void) {
  int status = checked_main();
  if (status == 0 && (tb_live_words != 0 || tb_live_blocks != 0 ||
      tb_tasks != 0 || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0)) {
    (void)fputs("unsafe executable retained runtime owners\n", stderr);
    return 91;
  }
  return status;
}
"#,
        );
        let executable = executable_c_compiler::compile(&self.0, &generated, definitions);
        executable_c_compiler::bounded(
            Command::new(executable).current_dir(&self.0),
            Duration::from_secs(20),
        )
    }

    fn expect_all(&self, native: &str, compiled: &str) {
        let checked = self.checked();
        success(&self.cli("run", &[]), native);
        success(&self.javascript(&checked), compiled);
        success(&self.native_c(&checked, &["BEND_MAX_ALLOC=8192"]), compiled);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn success(output: &Output, expected: &str) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected.as_bytes());
    assert!(output.stderr.is_empty());
}

const COUNTDOWN: &str = r"import Base
@unsafe
def count(n: Nat) -> Nat:
  match n:
    case 0n: 0n
    case 1n+p: Nat.add(1n, count(Nat.add(p, 0n)))
def main() -> IO(Unit): IO.print(Nat.show(count(12n)))
";

const REUSABLE_CLOSURE: &str = r#"import Base
@unsafe
def twice(+function: Unit -> String) -> String & String:
  (function(Unit{}), function(Unit{}))
def show(pair: String & String) -> IO(Unit):
  (left, right) = pair
  do IO<Unit>:
    Unit <- IO.print(left)
    IO.print(right)
def main() -> IO(Unit):
  +text = String.append("own", "ed")
  show(twice(ignored => text))
"#;

const REUSABLE_LOCAL: &str = r"import Base
@unsafe
def twice(n: Nat) -> Nat:
  +function = {x => 1n+x : Nat -> Nat}
  function(function(n))
def main() -> IO(Unit): IO.print(Nat.show(twice(2n)))
";

#[test]
fn annotated_nonstructural_calls_and_reusable_functions_execute_on_all_targets() {
    for (source, expected, name) in [
        (COUNTDOWN, "12\n", "count"),
        (REUSABLE_CLOSURE, "owned\nowned\n", "twice"),
        (REUSABLE_LOCAL, "4\n", "twice"),
    ] {
        let fixture = Fixture::new(source);
        assert_eq!(fixture.checked().unsafe_names().collect::<Vec<_>>(), [name]);
        fixture.expect_all(expected, expected);
    }
}

const PRIOR_LAW: &str = r"import Base
law twice: @+function: (Nat -> Nat) -> Nat -> Nat
@unsafe
def twice(function, n): function(function(n))
def main() -> IO(Unit): IO.print(Nat.show(twice(x => 1n+x, 3n)))
";

#[test]
fn an_earlier_law_uses_its_annotated_implementation_signature_context() {
    let fixture = Fixture::new(PRIOR_LAW);
    assert_eq!(
        fixture.checked().unsafe_names().collect::<Vec<_>>(),
        ["twice"]
    );
    fixture.expect_all("5\n", "5\n");
}

const UNSAFE_TEMPLATE: &str = r"import Base
@unsafe
def count(~step: Nat -> Nat, n: Nat) -> Nat:
  match n:
    case 0n: 0n
    case 1n+p: step(count(~step, Nat.add(p, 0n)))
def main() -> IO(Unit): IO.print(Nat.show(count(~(x => 1n+x), 3n)))
";

const IMPORTED_UNSAFE: &str = r"import Base
law twice: @+function: (Nat -> Nat) -> Nat -> Nat
@unsafe
def twice(function, n): function(function(n))
";

const IMPORT_MAIN: &str = r"import Base
import helper.bend as Helpers
def main() -> IO(Unit): IO.print(Nat.show(Helpers.twice(x => 1n+x, 4n)))
";

#[test]
fn templates_and_namespaced_imports_preserve_annotation_scope() {
    let template = Fixture::new(UNSAFE_TEMPLATE);
    assert_eq!(
        template.checked().unsafe_names().collect::<Vec<_>>(),
        ["count~0"]
    );
    template.expect_all("3\n", "3\n");
    let imported = Fixture::new(IMPORT_MAIN);
    fs::write(imported.0.join("helper.bend"), IMPORTED_UNSAFE).unwrap();
    assert_eq!(
        imported.checked().unsafe_names().collect::<Vec<_>>(),
        ["helper.twice"]
    );
    imported.expect_all("6\n", "6\n");
}

#[test]
fn unused_template_text_stays_deferred_and_annotated_foreign_signatures_stay_opaque() {
    let deferred = Fixture::new(
        "import Base\n@unsafe\ndef deferred(~n: Nat) -> Nat: ?unused\ndef main() -> Nat: 0n\n",
    );
    check_book(&load(deferred.0.join("main.bend")).unwrap()).unwrap();
    assert_eq!(deferred.checked().unsafe_names().count(), 0);

    let foreign = Fixture::new(
        "import Base\n@unsafe\ndef foreign(+function: Unit -> U32) -> IO(U32):\n  import \"effect.js\"\ndef main() -> Nat: 0n\n",
    );
    let checked = foreign.checked();
    assert!(checked.foreign_names().any(|name| name == "foreign"));
    assert_eq!(checked.unsafe_names().collect::<Vec<_>>(), ["foreign"]);
    assert!(checked.definition_type("foreign").is_some());
}

const UNUSED_UNSAFE: &str = r"import Base
@unsafe
def loop(n: Nat) -> Nat: loop(n)
def main() -> Nat: 0n
";

const ASSUMED_EQUALITY: &str = r"import Base
@unsafe
def assumption() -> {0n == 1n : Nat}: assumption
def ignore(-proof: {0n == 1n : Nat}) -> Nat: 0n
def main() -> Nat: ignore(assumption)
";

#[test]
fn unsafe_assumptions_execute_without_becoming_strict_proof_certificates() {
    let assumed = Fixture::new(ASSUMED_EQUALITY);
    assert_eq!(
        assumed.checked().unsafe_names().collect::<Vec<_>>(),
        ["assumption"]
    );
    assumed.expect_all("Zero{}\n", "0n\n");
    for source in [UNUSED_UNSAFE, ASSUMED_EQUALITY] {
        let fixture = Fixture::new(source);
        let book = load(fixture.0.join("main.bend")).unwrap();
        let error = check_book(&book).unwrap_err();
        assert!(error.to_string().contains("strict proof checking rejects"));
        for command in ["check", "eval", "serve"] {
            let output = fixture.cli(command, &[]);
            assert_eq!(output.status.code(), Some(1), "{command}");
            assert!(output.stdout.is_empty(), "{command}");
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("strict proof checking rejects"),
                "{command}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        for target in ["c", "javascript"] {
            let output = fixture.cli("compile", &["--target", target, "--output", "pure.out"]);
            assert_eq!(output.status.code(), Some(1), "{target}");
            assert!(output.stdout.is_empty());
            assert!(!fixture.0.join("pure.out").exists());
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("strict proof checking rejects")
            );
        }
    }
}

#[test]
fn annotations_do_not_relax_usage_holes_reflexivity_or_later_ordinary_definitions() {
    for (name, source, diagnostic) in [
        (
            "affine duplicate",
            "type Ticket is Type: Ticket{}\n@unsafe\ndef bad(ticket: Ticket) -> Ticket & Ticket: (ticket, ticket)\n",
            "permits Lone use, observed Many",
        ),
        (
            "live erased value",
            "@unsafe\ndef bad(-n: Nat) -> Nat: n\n",
            "permits None use, observed Lone",
        ),
        (
            "body hole",
            "@unsafe\ndef bad() -> Nat: ?unfinished\n",
            "unfinished proof hole",
        ),
        (
            "type hole",
            "@unsafe\ndef bad(x: ?unfinished) -> Nat: 0n\n",
            "unfinished proof hole",
        ),
        (
            "false reflexivity",
            "@unsafe\ndef bad() -> {0n == 1n : Nat}: {==}\n",
            "reflexivity",
        ),
        (
            "ordinary later recursion",
            "@unsafe\ndef allowed(n: Nat) -> Nat: allowed(n)\ndef bad(n: Nat) -> Nat: bad(n)\n",
            "decrease structurally",
        ),
        (
            "ordinary later signature",
            "@unsafe\ndef allowed(+function: Nat -> Nat) -> Nat: 0n\ndef bad(+function: Nat -> Nat) -> Nat: 0n\n",
            "type mismatch",
        ),
        (
            "ordinary later local",
            "@unsafe\ndef allowed() -> Nat: 0n\ndef bad() -> Nat:\n  +function = {n => n : Nat -> Nat}\n  function(0n)\n",
            "type mismatch",
        ),
        (
            "ordinary later datatype",
            "@unsafe\ndef allowed() -> Nat: 0n\ntype Bad is Data: Bad{function: Nat -> Nat}\n",
            "type mismatch",
        ),
        (
            "forward live use",
            "law later: @+function: (Nat -> Nat) -> Nat\ndef early() -> Nat: later(x => x)\n@unsafe\ndef later(function): 0n\n",
            "cannot be used as live evidence",
        ),
        (
            "unfilled law",
            "law missing: Nat\n@unsafe\ndef bad() -> Nat: missing\n",
            "cannot be used as live evidence",
        ),
        (
            "unsafe foreign gate",
            "@unsafe\ndef bad() -> Nat:\n  import \"foreign.js\"\n",
            "actual Base IO",
        ),
    ] {
        let error = Fixture::new(&format!("import Base\n{source}")).rejection();
        assert!(error.contains(diagnostic), "{name}: {error}");
    }
}

const DIVERGENT_PURE: &str = r"import Base
@unsafe
def loop(n: Nat) -> Nat: loop(n)
def main() -> Nat: loop(0n)
";

const DIVERGENT_IO: &str = r"import Base
@unsafe
def loop(n: Nat) -> Nat: loop(n)
def main() -> IO(Unit): IO.print(Nat.show(loop(0n)))
";

#[test]
fn divergent_executables_fail_with_bounded_diagnostics_instead_of_host_overflow() {
    for source in [DIVERGENT_PURE, DIVERGENT_IO] {
        let fixture = Fixture::new(source);
        let checked = fixture.checked();
        let native = fixture.cli("run", &[]);
        assert_eq!(native.status.code(), Some(1));
        assert!(native.stdout.is_empty());
        let error = String::from_utf8_lossy(&native.stderr);
        assert!(
            error.contains("budget exhausted")
                || error.contains("nesting limit exhausted")
                || error.contains("data runtime continuation depth exhausted"),
            "{error}"
        );
        let javascript = fixture.javascript(&checked);
        assert_eq!(javascript.status.code(), Some(1));
        assert!(javascript.stdout.is_empty());
        assert!(String::from_utf8_lossy(&javascript.stderr).contains("step budget exhausted"));
        let native_c = fixture.native_c(
            &checked,
            &[
                "BEND_MAX_STEPS=128",
                "BEND_MAX_DEPTH=32",
                "BEND_MAX_FRAMES=32",
            ],
        );
        assert_eq!(native_c.status.code(), Some(1));
        assert!(native_c.stdout.is_empty());
        assert_eq!(
            native_c.stderr,
            b"teamy-bend executable C: evaluation budget exhausted\n"
        );
    }
}
