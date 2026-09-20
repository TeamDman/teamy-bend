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
            "teamy-bend-executable-js-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, source: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, source).unwrap();
        path
    }
    fn compile(&self, source: &str) -> Result<String, teamy_bend::compiler::CompileError> {
        let loaded = load_executable(self.write("main.bend", source)).unwrap();
        let checked = check_executable(&loaded).unwrap();
        compile_executable_javascript(&checked)
    }
    fn run(&self, source: &str) -> Output {
        let generated = self.compile(source).unwrap();
        let script = self.write("main.cjs", &generated);
        Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
            .arg(script)
            .output()
            .expect("executable compiler tests require Node.js")
    }
    fn expect(&self, source: &str, expected: &str) {
        let output = self.run(source);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        assert!(output.stderr.is_empty());
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn pure_printer_preserves_native_values_recursive_data_and_type_aliases() {
    let fixture = Fixture::new();
    for (result_type, expression, expected) in [
        ("Nat", "7n", "7n\n"),
        ("Bool", "True{}", "True{}\n"),
        ("F32", "F32.neg(0.0)", "-0.0\n"),
        ("Char", "'❁'", "'❁'\n"),
        ("String", "\"a\\0b\\n\"", "\"a\\0b\\n\"\n"),
        ("List<Nat>", "[1n, 2n]", "[1n, 2n]\n"),
        ("Nat & Bool", "(1n, False{})", "(1n, False{})\n"),
        ("Array<Nat>", "ANode{ALeaf{1n}, ALeaf{2n}}", "[1n, 2n]\n"),
    ] {
        fixture.expect(&format!("import Base\ndef Alias() -> Type: {result_type}\ndef main() -> Alias: {expression}\n"), expected);
    }
}

#[test]
fn native_array_updates_and_clones_keep_affine_ownership() {
    let fixture = Fixture::new();
    fixture.expect(
        r"
import Base
def done(value: Array<U32> & Array<U32>) -> List<U32> & List<U32>:
  (left, right) = value
  (Array.to_list(~U32, Array.set(U32, left, 1, 9)), Array.to_list(~U32, right))
def main() -> List<U32> & List<U32>:
  done(Array.clone(U32, ANode{ALeaf{1}, ALeaf{2}}))
",
        "([1, 9], [1, 2])\n",
    );
}

#[test]
fn foreign_callbacks_complete_and_erased_slots_do_not_shift_live_arguments() {
    let fixture = Fixture::new();
    fixture.write(
        "effect.js",
        "function fold(f) { return f(f(1)(2))(3); }\nfunction apply(f) { return f(7); }\n",
    );
    fixture.expect(
        r#"
import Base
def fold(f: U32 -> U32 -> U32) -> IO(U32):
  import "effect.js"
def apply(f: @-tag: Nat -> @x: U32 -> U32) -> IO(U32):
  import "effect.js"
def combine(a: U32, b: U32) -> U32: U32.add(U32.mul(a, 10), b)
def bump(-tag: Nat, x: U32) -> U32: U32.add(x, 1)
def main() -> IO(Unit):
  do IO<Unit>:
    a : U32 <- fold(combine)
    Unit <- IO.print(U32.show(a))
    b : U32 <- apply(bump)
    IO.print(U32.show(b))
"#,
        "123\n8\n",
    );
}

#[test]
fn long_tail_recursion_uses_the_trampoline_and_respects_step_budget() {
    let fixture = Fixture::new();
    fixture.write("count.js", "function count_input() { return 20000n; }\n");
    let source = r#"
import Base
def count_input() -> IO(Nat):
  import "count.js"
def count(n: Nat, accumulator: U32) -> U32:
  match n:
    case Zero{}: accumulator
    case Succ{previous}: count(previous, U32.add(accumulator, 1))
def main() -> IO(Unit):
  do IO<Unit>:
    n : Nat <- count_input()
    IO.print(U32.show(count(n, 0)))
"#;
    fixture.expect(source, "20000\n");
    fixture.write("count.js", "function count_input() { return 500000n; }\n");
    let output = fixture.run(source);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("step budget exhausted"));
    assert!(output.stdout.is_empty());
}

#[test]
fn namespaced_foreign_constructors_use_original_tags_and_named_live_fields() {
    let fixture = Fixture::new();
    fixture.write(
        "far.bend",
        r#"
import Base
type Box is Data:
  Local.Box{-tag: Nat, value: U32}
def make() -> IO(Box):
  import "far.js"
def open(box: Box) -> U32:
  match box:
    case Local.Box{tag, value}: value
"#,
    );
    fixture.write(
        "far.js",
        "function make() { return {$:'Local.Box',value:42}; }\n",
    );
    fixture.expect(
        r"
import Base
import far.bend as F
def main() -> IO(Unit):
  do IO<Unit>:
    box : F.Box <- F.make()
    IO.print(U32.show(F.open(box)))
",
        "42\n",
    );
}

#[test]
fn raw_foreign_values_are_not_reconstructed_or_validated_at_the_seam() {
    let fixture = Fixture::new();
    fixture.write("raw.js", "function truth() { return 1; }\nfunction fraction() { return 4.5; }\nfunction text() { return 'a\\uD800'; }\n");
    fixture.expect(
        r#"
import Base
def truth() -> IO(Bool):
  import "raw.js"
def fraction() -> IO(U32):
  import "raw.js"
def text() -> IO(String):
  import "raw.js"
def choose(value: Bool) -> String:
  match value:
    case False{}: "false"
    case True{}: "true"
def main() -> IO(Unit):
  do IO<Unit>:
    b : Bool <- truth()
    Unit <- IO.print(choose(b))
    n : U32 <- fraction()
    Unit <- IO.print(U32.show(n))
    s : String <- text()
    IO.print(s)
"#,
        "true\n4\na�\n",
    );
}

#[test]
fn user_intrinsic_names_and_local_nat_types_keep_their_checked_meaning() {
    let fixture = Fixture::new();
    fixture.expect(
        r"
type Nat is Data:
  Zero{}
  Succ{pred: Nat}
def Nat.add(a: Nat, b: Nat) -> Nat: b
def main() -> Nat: Nat.add(Succ{Zero{}}, Succ{Succ{Zero{}}})
",
        "Succ{Succ{Zero{}}}\n",
    );
    fixture.expect(
        r"
import Base
def U32.Div(a: U32, b: U32) -> U32: U32.add(a, b)
def main() -> U32: U32.Div(10, 3)
",
        "13\n",
    );
}

#[test]
fn pure_function_and_erased_dependent_results_fail_explicitly() {
    let fixture = Fixture::new();
    for source in [
        "import Base\ndef main() -> Nat -> Nat: n => n\n",
        "import Base\ntype Pack is Type:\n  Pack{-A:Type, value:A}\ndef main() -> Pack: Pack{Nat, 1n}\n",
    ] {
        let error = fixture
            .compile(source)
            .expect_err("unsupported pure printer type");
        assert!(error.to_string().contains("cannot be printed"));
    }
}

#[test]
fn source_char_construction_checks_even_when_io_discards_the_value() {
    let fixture = Fixture::new();
    let output = fixture.run("import Base\ndef main() -> IO(Char): IO.pure(Char, Chr{55296})\n");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not a Unicode scalar"));
    assert!(output.stdout.is_empty());
}

#[test]
fn numeric_currying_preserves_quiet_and_signaling_nan_payloads() {
    let fixture = Fixture::new();
    fixture.expect(
        r"
import Base
def make(value: U32) -> F32:
  match value:
    case U32{word}: F32{word}
def main() -> List<U32>:
  [F32.bits(make(2143289345)), F32.bits(make(2139095041)), F32.bits(make(4290772993)), F32.bits(make(4286578689))]
",
        "[2143289345, 2143289345, 4290772993, 4290772993]\n",
    );
}

#[test]
fn non_tail_recursion_and_checked_string_growth_fail_with_explicit_budgets() {
    let fixture = Fixture::new();
    fixture.write("depth.js", "function input() { return 2000n; }\n");
    let output = fixture.run(
        r#"
import Base
def input() -> IO(Nat):
  import "depth.js"
def copy(n: Nat) -> Nat:
  match n:
    case Zero{}: Zero{}
    case Succ{p}: Succ{copy(p)}
def main() -> IO(Nat):
  do IO<Nat>:
    n : Nat <- input()
    IO.pure(Nat, copy(n))
"#,
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("call depth budget exhausted"));
    assert!(output.stdout.is_empty());

    let output = fixture.run(
        r#"
import Base
def grow(n: Nat, +text: String) -> String:
  match n:
    case Zero{}: text
    case Succ{p}: grow(p, String.append(text, text))
def main() -> IO(String): IO.pure(String, grow(24n, "a"))
"#,
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("string byte budget exhausted"));
    assert!(output.stdout.is_empty());
}
