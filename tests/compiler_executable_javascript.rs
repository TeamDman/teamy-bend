// SPDX-License-Identifier: MPL-2.0
#[path = "support/nat_failure_cases.rs"]
mod nat_failure_cases;
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

#[test]
fn natural_multiply_add_double_and_successor_share_the_language_failure_contract() {
    for expression in nat_failure_cases::cases() {
        let output = Fixture::new().run_bounded(&nat_failure_cases::source(expression));
        assert_eq!(output.status.code(), Some(1), "{expression}");
        assert_eq!(output.stdout, b"before");
        assert_eq!(
            output.stderr,
            b"bend: a Nat past the largest immediate 2^48-1\n"
        );
    }
}

#[test]
fn array_depth_language_wall_stays_distinct_from_the_element_budget() {
    let fixture = Fixture::new();
    let original = fixture.run_bounded(include_str!(
        "fixtures/cpu-runtime-negatives/array_depth_bound.bend"
    ));
    assert_eq!(original.status.code(), Some(1));
    assert!(original.stdout.is_empty());
    assert_eq!(
        original.stderr,
        b"bend: an array past the deepest block class 31\n"
    );
    for depth in [0, 17, 18, 31, 32, 33] {
        let output = fixture.run_bounded(&format!(
            r#"import Base
def observe(r: Array<U32> & U32) -> IO(Unit):
  (a, x) = r
  IO.print(U32.show(x))
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.write("before")
    observe(Array.get(U32, Array.new(U32, {depth}n, 7), 0))
"#
        ));
        assert_eq!(
            output.status.code(),
            Some(i32::from(depth > 17)),
            "{depth}: {output:?}"
        );
        assert_eq!(
            output.stdout,
            if depth <= 17 {
                &b"before7\n"[..]
            } else {
                &b"before"[..]
            }
        );
        assert_eq!(
            output.stderr,
            if depth > 31 {
                &b"bend: an array past the deepest block class 31\n"[..]
            } else if depth > 17 {
                &b"teamy-bend generated program: executable JavaScript array element budget exhausted\n"[..]
            } else {
                &b""[..]
            }
        );
    }
}

#[test]
fn scalar_boundaries_preserve_exact_language_failures_and_first_error_order() {
    let fixture = Fixture::new();
    for code in [55_295, 55_296, 57_343, 57_344, 1_114_111, 1_114_112] {
        let invalid = matches!(code, 55_296 | 57_343 | 1_114_112);
        let output = fixture.run_bounded(&format!(
            r#"import Base
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.write("before")
    Unit <- IO.print(U32.show(Char.to_u32(Chr{{{code}}})))
    IO.print("after")
"#
        ));
        assert_eq!(
            output.status.code(),
            Some(i32::from(invalid)),
            "{code}: {output:?}"
        );
        assert_eq!(
            output.stdout,
            if invalid {
                b"before".to_vec()
            } else {
                format!("before{code}\nafter\n").into_bytes()
            }
        );
        assert_eq!(
            output.stderr,
            if invalid {
                format!("bend: {code} is not a Unicode scalar value\n").into_bytes()
            } else {
                Vec::new()
            }
        );
    }
    for (expression, diagnostic) in [
        (
            "Char.to_u32(Chr{55296})",
            "bend: 55296 is not a Unicode scalar value\n",
        ),
        (
            "U32.from_nat(Nat.mul(U32.to_nat(16777216), U32.to_nat(16777216)))",
            "bend: a Nat past the largest immediate 2^48-1\n",
        ),
    ] {
        let output = fixture.run_bounded(&format!("import Base\ndef observe(r: Array<U32> & U32) -> IO(Unit):\n  (a, x) = r\n  IO.print(U32.show(x))\ndef main() -> IO(Unit):\n  observe(Array.get(U32, Array.new(U32, 32n, {expression}), 0))\n"));
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, diagnostic.as_bytes());
    }
}

#[test]
fn original_foreign_scalar_marshalling_has_the_exact_failure_contract() {
    let fixture = Fixture::new();
    fixture.write(
        "marshal_char_scalar.js",
        include_str!("fixtures/cpu-runtime-negatives/marshal_char_scalar.js"),
    );
    let output = fixture.run_bounded(include_str!(
        "fixtures/cpu-runtime-negatives/marshal_char_scalar.bend"
    ));
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"bend: 55296 is not a Unicode scalar value\n"
    );
}

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
    fn run_bounded(&self, source: &str) -> Output {
        let generated = self.compile(source).unwrap();
        let script = self.write("bounded.cjs", &generated);
        let stdout = self.0.join("bounded.stdout");
        let stderr = self.0.join("bounded.stderr");
        let mut child =
            Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
                .arg(script)
                .stdout(fs::File::create(&stdout).unwrap())
                .stderr(fs::File::create(&stderr).unwrap())
                .spawn()
                .expect("constructor-tail tests require Node.js");
        let started = std::time::Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if started.elapsed() > std::time::Duration::from_secs(30) {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("constructor-tail program exceeded its 30-second deadline");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        Output {
            status,
            stdout: fs::read(stdout).unwrap(),
            stderr: fs::read(stderr).unwrap(),
        }
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
fn upstream_whole_list_programs_complete_with_the_default_node_stack() {
    let fixture = Fixture::new();
    for source in [
        include_str!("fixtures/cpu-comptime/hand.bend"),
        include_str!("fixtures/cpu-comptime/tmpl.bend"),
    ] {
        let output = fixture.run_bounded(source);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert_eq!(output.stdout, b"50005000\n");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

#[test]
fn constructor_tails_preserve_erased_fields_closures_and_float_bits() {
    let fixture = Fixture::new();
    let output = fixture.run_bounded(
        r"
import Base
type Chain is Type:
  End{}
  Link{-tag: Nat, value: F32, callback: U32 -> U32, tail: Chain}
def make_float(value: U32) -> F32:
  match value:
    case U32{word}: F32{word}
def make(n: Nat) -> Chain:
  match n:
    case 0n: End{}
    case 1n+p:
      Link{0n, make_float(2147483648), x => U32.add(x, 1), make(p)}
def consume(chain: Chain, acc: U32) -> U32:
  match chain:
    case End{}: acc
    case Link{tag, value, callback, tail}:
      consume(tail, callback(U32.add(acc, U32.sub(F32.bits(value), 2147483648))))
def main() -> IO(Unit):
  IO.print(U32.show(consume(make(U32.to_nat(10000)), 0)))
",
    );
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"10000\n");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn infinite_constructor_tails_fail_closed_and_prefix_errors_keep_their_order() {
    let fixture = Fixture::new();
    let source = r#"
import Base
type Chain is Data: Link{head: Char, tail: Chain}
@unsafe
def forever(n: Nat) -> Chain: Link{'a', forever(n)}
def discard(chain: Chain) -> IO(Unit): IO.print("unreachable")
def main() -> IO(Unit): discard(forever(0n))
"#;
    let output = fixture.run_bounded(source);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"teamy-bend generated program: executable JavaScript constructor chain budget exhausted\n"
    );
    let output = fixture.run_bounded(&source.replace("'a'", "Char.from_u32(1114112)"));
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"bend: 1114112 is not a Unicode scalar value\n"
    );
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
        "import Base\ndef main(n: Nat) -> Nat: n\n",
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
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        output.stderr,
        b"bend: 55296 is not a Unicode scalar value\n"
    );
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

#[test]
fn extended_numeric_operations_round_to_binary32() {
    let fixture = Fixture::new();
    fixture.expect(
        r"
import Base
def main() -> List<U32>:
  [F32.bits(F32.pow(2.0, 3.0)), F32.bits(F32.atan2(1.0, 1.0)),
   F32.bits(F32.sqrt(2.0)), F32.bits(F32.exp(1.0)), F32.bits(F32.log(2.0)),
   F32.bits(F32.log2(8.0)), F32.bits(F32.log10(100.0)),
   F32.bits(F32.sin(1.0)), F32.bits(F32.cos(1.0)), F32.bits(F32.tan(1.0)),
   F32.bits(F32.asin(0.5)), F32.bits(F32.acos(0.5)), F32.bits(F32.atan(1.0)),
   F32.bits(F32.sinh(1.0)), F32.bits(F32.cosh(1.0)), F32.bits(F32.tanh(1.0)),
   F32.bits(F32.floor(F32.neg(1.5))), F32.bits(F32.ceil(F32.neg(1.5))),
   F32.bits(F32.trunc(F32.neg(1.5)))]
",
        "[1090519040, 1061752795, 1068827891, 1076754516, 1060205080, 1077936128, 1073741824, 1062693540, 1057640768, 1070029091, 1057360530, 1065749138, 1061752795, 1066822910, 1069908907, 1061353430, 3221225472, 3212836864, 3212836864]\n",
    );
}

#[test]
fn numeric_text_preserves_special_values_rounding_and_decimal_style() {
    let fixture = Fixture::new();
    fixture.expect(
        r"
import Base
def make(value: U32) -> F32:
  match value:
    case U32{word}: F32{word}
def main() -> List<String>:
  [F32.show(F32.sqrt(F32.neg(1.0))), F32.show(F32.log(0.0)),
   F32.show(F32.exp(1000.0)), F32.show(F32.pow(F32.neg(0.0), F32.neg(3.0))),
   F32.show(F32.sqrt(F32.neg(0.0))), F32.show(F32.ceil(F32.neg(0.1))),
   F32.show(F32.trunc(F32.neg(0.1))), F32.show(F32.sin(F32.neg(0.0))),
   F32.show(0.0), F32.show(1.0), F32.show(0.1), F32.show(16777217.0),
   F32.show(0.000001), F32.show(0.0000001), F32.show(1000000000000000000000.0),
   F32.show(make(1)), F32.show(make(8388608)), F32.show(make(2139095039))]
",
        "[\"nan\", \"-inf\", \"inf\", \"-inf\", \"-0\", \"-0\", \"-0\", \"-0\", \"0\", \"1\", \"0.1\", \"16777216\", \"0.000001\", \"1e-7\", \"1e+21\", \"1e-45\", \"1.1754944e-38\", \"3.4028235e+38\"]\n",
    );
}

#[test]
fn float_read_matches_upstream_decimal_and_special_value_grammar() {
    let fixture = Fixture::new();
    fixture.expect(
        r#"
import Base
def show_read(parsed: Maybe<&2, F32>) -> String:
  match parsed:
    case None{}: "invalid"
    case Some{value}: F32.show(value)
def main() -> List<String>:
  [show_read(F32.read("  +1.25")), show_read(F32.read("\t\n\r-0")),
   show_read(F32.read(".5")), show_read(F32.read("1.")), show_read(F32.read("01.2e+1")),
   show_read(F32.read("INF")), show_read(F32.read("-infinity")), show_read(F32.read("+NaN")),
   show_read(F32.read("-nan")), show_read(F32.read("1e1000")),
   show_read(F32.read("-1e-1000")), show_read(F32.read("1e-1000")),
   show_read(F32.read("16777217")), show_read(F32.read("1 ")), show_read(F32.read("1\n")),
   show_read(F32.read("1\r")), show_read(F32.read("0x10")), show_read(F32.read("1_000")),
   show_read(F32.read("1e")), show_read(F32.read(".")), show_read(F32.read("")),
   show_read(F32.read(" ")), show_read(F32.read("infinite")), show_read(F32.read("nan(payload)"))]
"#,
        "[\"1.25\", \"-0\", \"0.5\", \"1\", \"12\", \"inf\", \"-inf\", \"nan\", \"nan\", \"inf\", \"-0\", \"0\", \"16777216\", \"invalid\", \"invalid\", \"invalid\", \"invalid\", \"invalid\", \"invalid\", \"invalid\", \"invalid\", \"invalid\", \"invalid\", \"invalid\"]\n",
    );
}

#[test]
fn extracted_fields_remain_matchable_in_executable_javascript() {
    Fixture::new().expect(
        include_str!("fixtures/cpu-destructure/field_match_in_order.bend"),
        "9\n",
    );
}
