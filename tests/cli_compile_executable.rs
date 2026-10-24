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
        let root = std::env::temp_dir().join(format!(
            "teamy-bend-executable-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("main.bend"), source).unwrap();
        Self(root)
    }

    fn compile(&self, options: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .args(["--output-format", "json", "compile"])
            .arg(self.0.join("main.bend"))
            .arg("--output")
            .arg(self.0.join("main.cjs"))
            .args(options)
            .output()
            .expect("run compiler CLI")
    }

    fn run(&self) -> Output {
        Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
            .arg(self.0.join("main.cjs"))
            .output()
            .expect("executable compiler tests require Node.js")
    }

    fn compile_ok(&self) {
        let output = self.compile(&["--executable"]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("javascript"));
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn executable_compilation_preserves_console_bytes_and_numeric_results() {
    let fixture = Fixture::new(
        r#"
import Base
def main() -> IO(U32):
  do IO<U32>:
    a : Unit <- IO.write("é\0")
    b : Unit <- IO.print(U32.show(F32.to_u32(F32.mul(3.5, 12.0))))
    c : Unit <- IO.print_err("err❁")
    return 7
"#,
    );
    fixture.compile_ok();
    let output = fixture.run();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, "é\u{0}42\n".as_bytes());
    assert_eq!(output.stderr, "err❁\n".as_bytes());
}

#[test]
fn generated_halt_preserves_status_and_stops_following_effects() {
    let fixture = Fixture::new(
        r#"
import Base
def main() -> IO(Unit):
  do IO<Unit>:
    a : Unit <- IO.write("prefix")
    b : Unit <- IO.die(Unit, 7, "stopped")
    IO.print("AFTER")
"#,
    );
    fixture.compile_ok();
    let output = fixture.run();
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, b"prefix");
    assert_eq!(output.stderr, b"stopped\n");
}

#[test]
fn executable_options_do_not_change_the_default_strict_compiler() {
    let fixture = Fixture::new("import Base\ndef main() -> IO(Unit): IO.print(\"hello\")\n");
    let pure = fixture.compile(&[]);
    assert!(
        !pure.status.success(),
        "pure compilation cannot assume IO contracts"
    );
    assert!(!fixture.0.join("main.cjs").exists());
    let output = fixture.compile(&["--executable", "--entry", "other"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires the main entry"));
    assert!(!fixture.0.join("main.cjs").exists());
}

#[test]
fn invalid_proof_never_replaces_an_existing_output() {
    let fixture = Fixture::new(
        "import Base\nlaw wrong: {0n == 1n : Nat}\ndef wrong(): {==}\ndef main() -> IO(Unit): IO.print(\"BAD\")\n",
    );
    std::fs::write(fixture.0.join("main.cjs"), "preserve this output").unwrap();
    let output = fixture.compile(&["--executable", "--force"]);
    assert!(!output.status.success());
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("main.cjs")).unwrap(),
        "preserve this output"
    );
}

#[test]
fn compiling_foreign_source_does_not_execute_it() {
    let fixture = Fixture::new(
        "import Base\ndef stamp() -> IO(Unit):\n  import \"./effect.js\"\ndef main() -> IO(Unit): stamp()\n",
    );
    let marker = fixture.0.join("executed.txt");
    let marker_json = facet_json::to_string(&marker.to_string_lossy().as_ref()).unwrap();
    std::fs::write(fixture.0.join("effect.js"), format!(
        "function stamp() {{ require('node:fs').writeFileSync({marker_json}, 'done'); return {{$:'Unit'}}; }}\n"
    )).unwrap();
    fixture.compile_ok();
    assert!(
        !marker.exists(),
        "compilation must only read the foreign source"
    );
    let output = fixture.run();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "done");
}

#[test]
fn generated_pure_main_and_missing_entry_follow_distinct_contracts() {
    let pure = Fixture::new("import Base\ndef main() -> U32: 42\n");
    pure.compile_ok();
    let output = pure.run();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"42\n");

    for (source, expected) in [
        (
            "type Marker is Data:\n  Mark{}\ndef value() -> Marker: Mark{}\n",
            "Error: a build needs import Base\n",
        ),
        (
            "import Base\ndef value() -> U32: 1\n",
            "Error: no main to run\n",
        ),
        ("type IO is Data:\n  Marker{}\n", "Error: no main to run\n"),
        (
            "type Marker is Data:\n  Mark{}\ndef IO() -> Type: Marker\n",
            "Error: no main to run\n",
        ),
    ] {
        let empty = Fixture::new(source);
        for target in ["javascript", "c", "native"] {
            let output = empty.compile(&["--executable", "--target", target]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert_eq!(output.stderr, expected.as_bytes());
            assert!(!empty.0.join("main.cjs").exists());
        }
        std::fs::write(empty.0.join("main.cjs"), "preserved output").unwrap();
        let output = empty.compile(&["--executable", "--force"]);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stderr, expected.as_bytes());
        assert_eq!(
            std::fs::read_to_string(empty.0.join("main.cjs")).unwrap(),
            "preserved output"
        );
    }
}

#[test]
fn executable_checker_rejections_preserve_diagnostics_and_existing_outputs() {
    for (source, expected) in [
        (
            include_str!("fixtures/cpu-rewrite-checking/rewrite_non_equation.bend"),
            "Error:\n- expected : an annotated term (cannot infer)\n- observed : 5\nLocation: main\n 9 | \n10>| def main(): %5 : U32; 1\n11 | \n",
        ),
        (
            include_str!("fixtures/cpu-rewrite-checking/rewrite_ctor_brace.bend"),
            "Error:\n- expected : @_:Opt8<Un8> -> Opt8<Un8>\n- observed : @_:Opt8<Emp8> -> Opt8<Emp8>\nContext:\n- e : {(x => x) == (x => Zer8{}) : @_:Opt8<Emp8> -> Opt8<Emp8>}\n- _ : @_:Opt8<Emp8> -> Opt8<Emp8>\n- q : {(x => x) == _ : @_:Opt8<Emp8> -> Opt8<Emp8>}\nLocation: bad_fun\n21 | def bad_fun(e):\n22>|   %q@e : {(x => x) == _ : Opt8<Un8> -> Opt8<Un8>}; {==}\n23 | \n",
        ),
        (
            include_str!("fixtures/cpu-rewrite-checking/dead_motive_syntax.bend"),
            "Error:\n- expected : ':'\n- observed : 'e'\nLocation:\n12 | def sym3(x, y, e):\n13>|   %[h => {y == h : U32}] e {=}\n14 | \n",
        ),
        (
            include_str!("fixtures/cpu-plus-binders/plus_binder_term.bend"),
            "Error:\n- expected : a bound variable\n- observed : x^-1\nContext:\n- x : U32\nLocation: main\n6 |   x = U32.add(1, 1)\n7>|   U32.add(+x, 1)\n8 | \n",
        ),
        (
            include_str!("fixtures/cpu-destructure/destructure_let_001.bend"),
            "Error:\n- message  : a parameter or field scrutinee (a match cannot scrutinize a local binder: give it its own def)\nLocation:\n13 | def main(p):\n14>|   q = p\n15 |   P{a, b} = q\n",
        ),
    ] {
        let fixture = Fixture::new(source);
        for target in ["javascript", "c", "native"] {
            for existing in [false, true] {
                let destination = fixture.0.join("main.cjs");
                if existing {
                    std::fs::write(&destination, b"preserved output").unwrap();
                }
                let output = fixture.compile(&["--executable", "--target", target, "--force"]);
                assert_eq!(output.status.code(), Some(1));
                assert!(output.stdout.is_empty());
                assert_eq!(output.stderr, expected.as_bytes());
                if existing {
                    assert_eq!(std::fs::read(&destination).unwrap(), b"preserved output");
                    std::fs::remove_file(destination).unwrap();
                } else {
                    assert!(!destination.exists());
                }
            }
        }
    }
}
