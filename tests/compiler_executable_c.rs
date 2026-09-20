// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_bend::compiler::CompileError;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-executable-c-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, source: &str) {
        fs::write(self.0.join(name), source).unwrap();
    }

    fn generate(&self, source: &str) -> Result<String, CompileError> {
        self.write("main.bend", source);
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        compile_executable_c(&checked)
    }

    fn run(&self, source: &str, definitions: &[&str]) -> Output {
        let c = self.generate(source).unwrap();
        let executable = executable_c_compiler::compile(&self.0, &c, definitions);
        executable_c_compiler::bounded(
            Command::new(executable).current_dir(&self.0),
            Duration::from_secs(20),
        )
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

const SCALARS: &str = r"import Base
def main() -> List<U32>:
  [U32.add(4294967295, 2), U32.div(9, 0), U32.shln(1, 32n),
   F32.bits(F32.add(16777216.0, 1.0)), F32.bits(F32.neg(0.0)),
   Char.to_u32('λ'), U32.from_nat(Nat.pow(2n, 40n))]
";

#[test]
fn packed_words_float_rounding_characters_and_nat_conversion_keep_native_semantics() {
    success(
        &Fixture::new().run(SCALARS, &[]),
        "[1, 0, 0, 1266679808, 2147483648, 955, 0]\n",
    );
}

const NAT_ARITHMETIC: &str = r"import Base
def main() -> Nat:
  Nat.add(Nat.pow(2n, 40n), 3n)
";

const NAT_BOUNDARY: &str = r"import Base
def main() -> Nat:
  Nat.add(Nat.pow(2n, 47n), Nat.sub(Nat.pow(2n, 47n), 1n))
";

#[test]
fn native_naturals_print_large_values_without_unary_expansion() {
    success(&Fixture::new().run(NAT_ARITHMETIC, &[]), "1099511627779n\n");
    success(&Fixture::new().run(NAT_BOUNDARY, &[]), "281474976710655n\n");
}

const NAT_OVERFLOW: &str = r"import Base
def main() -> Nat: Nat.pow(2n, 48n)
";

#[test]
fn native_natural_overflow_fails_instead_of_wrapping_at_the_upstream_48_bit_bound() {
    let output = Fixture::new().run(NAT_OVERFLOW, &[]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Nat"));
}

const ARRAYS: &str = r"import Base
def done(value: Array<U32> & Array<U32>) -> List<U32> & List<U32>:
  (left, right) = value
  (Array.to_list(~U32, Array.set(U32, left, 5, 9)), Array.to_list(~U32, right))

def main() -> List<U32> & List<U32>:
  done(Array.clone(U32, ANode{ALeaf{1}, ALeaf{2}}))
";

#[test]
fn array_clone_keeps_original_and_wraps_update_index() {
    success(&Fixture::new().run(ARRAYS, &[]), "([1, 9], [1, 2])\n");
}

const CLOSURES: &str = r"import Base
def map(~f: U32 -> U32, xs: List<U32>) -> List<U32>:
  match xs:
    case Nil{}: Nil{}
    case Con{head, tail}: Con{f(head), map(~f, tail)}

def adder(n: U32) -> U32 -> U32: x => U32.add(n, x)
def apply(f: U32 -> U32, value: U32) -> U32: f(value)
def main() -> List<U32>:
  map(~(x => apply(adder(3), x)), [1, 2, 3])
";

#[test]
fn templates_and_closures_preserve_capture_and_recursive_list_order() {
    success(&Fixture::new().run(CLOSURES, &[]), "[4, 5, 6]\n");
}

#[test]
fn pure_printer_preserves_boxed_data_aliases_and_escaped_text() {
    for (kind, value, expected) in [
        ("Bool", "True{}", "True{}\n"),
        ("F32", "F32.neg(0.0)", "-0.0\n"),
        ("Char", "'❁'", "'❁'\n"),
        ("String", "\"a\\0b\\n\"", "\"a\\0b\\n\"\n"),
        ("Nat & Bool", "(1n, False{})", "(1n, False{})\n"),
        ("Maybe<&2, U32>", "Some{7}", "Some{7}\n"),
    ] {
        let source =
            format!("import Base\ndef Alias() -> Type: {kind}\ndef main() -> Alias: {value}\n");
        success(&Fixture::new().run(&source, &[]), expected);
    }
}

#[test]
fn user_nat_and_u32_names_without_base_keep_their_checked_datatypes_and_functions() {
    success(
        &Fixture::new().run(
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef Nat.add(a: Nat, b: Nat) -> Nat: b\ndef main() -> Nat: Nat.add(Succ{Zero{}}, Succ{Succ{Zero{}}})\n",
            &[],
        ),
        "Succ{Succ{Zero{}}}\n",
    );
    success(
        &Fixture::new().run(
            "type Tag is Data: Off{} On{}\ntype U32 is Data: U32{tag: Tag}\ndef U32.add(a: U32, b: U32) -> U32: b\ndef main() -> U32: U32.add(U32{Off{}}, U32{On{}})\n",
            &[],
        ),
        "U32{On{}}\n",
    );
}

const CONSOLE: &str = r#"import Base
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.write("a\0é")
    Unit <- IO.print("🙂")
    Unit <- IO.print_err("diagnostic")
    IO.die(Unit, 7, "halt")
"#;

#[test]
fn console_preserves_utf8_nul_and_halt_streams_and_status() {
    let output = Fixture::new().run(CONSOLE, &[]);
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, "a\0é🙂\n".as_bytes());
    assert_eq!(output.stderr, b"diagnostic\nhalt\n");
}

#[test]
fn cli_compiles_executable_c_and_runs_without_a_socket_provider() {
    let fixture = Fixture::new();
    fixture.write(
        "main.bend",
        "import Base\ndef main() -> IO(Unit): IO.print(\"c-cli\")\n",
    );
    let output = executable_c_compiler::bounded(
        Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .current_dir(&fixture.0)
            .args([
                "compile",
                "--executable",
                "--target",
                "c",
                "main.bend",
                "--output",
                "main.c",
            ]),
        Duration::from_secs(20),
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!fixture.0.join("teamy-bend-sys.node").exists());
    let source = fs::read_to_string(fixture.0.join("main.c")).unwrap();
    let executable = executable_c_compiler::compile(&fixture.0, &source, &[]);
    success(
        &executable_c_compiler::bounded(
            Command::new(executable).current_dir(&fixture.0),
            Duration::from_secs(20),
        ),
        "c-cli\n",
    );
}

#[test]
fn checked_book_without_main_reports_success_without_calling_a_null_entry() {
    success(
        &Fixture::new().run("import Base\ndef helper() -> U32: 7\n", &[]),
        "All terms check.\n",
    );
}

#[test]
fn higher_order_polymorphic_arrays_fail_before_emitting_mismatched_layouts() {
    let error = Fixture::new()
        .generate(
            "import Base\ndef make(-A: Data, value: A) -> Array<Maybe<&2, A>>:\n  ALeaf{Some{value}}\ndef use(f: @-A: Data -> A -> Array<Maybe<&2, A>>) -> Array<Maybe<&2, U32>>:\n  f(U32, 3)\ndef main() -> Array<Maybe<&2, U32>>:\n  use(make)\n",
        )
        .unwrap_err();
    assert!(error.to_string().contains("unspecialized"), "{error}");
}

const FOREIGN_SCALAR: &str = r#"import Base
law twice:
  U32 -> IO(U32)
def twice(x):
  import "effect.c"
def main() -> IO(Unit):
  do IO<Unit>:
    value : U32 <- twice(21)
    IO.print(U32.show(value))
"#;
const FOREIGN_SCALAR_C: &str = r"
static Term twice_run(Env e, Term* fields, IoWork* work) {
  (void)e; (void)work;
  return (Term)(uint32_t)(2 * (uint32_t)fields[0]);
}
static void __attribute__((constructor)) twice_use(void) {
  io_eff(CID_TWICE, twice_run, 0);
}
";

#[test]
fn registered_foreign_effect_preserves_arrow_arity_and_raw_u32_argument() {
    let fixture = Fixture::new();
    fixture.write("effect.c", FOREIGN_SCALAR_C);
    success(&fixture.run(FOREIGN_SCALAR, &[]), "42\n");
}

const FOREIGN_ERASED: &str = r#"import Base
def inspect(-A: Type, live: Type, proof: {0 == 0 : U32}, x: U32) -> IO(U32):
  import "effect.c"
def main() -> IO(Unit):
  do IO<Unit>:
    value : U32 <- inspect(U32, Nat, {==}, 7)
    IO.print(U32.show(value))
"#;
const FOREIGN_ERASED_C: &str = r"
static Term inspect_run(Env e, Term* fields, IoWork* work) {
  (void)e; (void)work;
  if (fields[0] != 0 || fields[1] != 0 || term_tag(fields[3]) != TAG_CLO) return 999;
  return fields[2];
}
static void inspect_use(void) __attribute__((constructor)) {
  io_eff(CID_INSPECT, inspect_run, 0);
}
";

#[test]
fn foreign_erasure_keeps_live_type_proof_and_continuation_slots_in_upstream_order() {
    let fixture = Fixture::new();
    fixture.write("effect.c", FOREIGN_ERASED_C);
    success(&fixture.run(FOREIGN_ERASED, &[]), "7\n");
}

const FOREIGN_BOXED: &str = r#"import Base
def make() -> IO(Result<&1, &1, U32 & String, U32 & Bool & Char & String>):
  import "effect.c"

def flag(value: Bool) -> String:
  match value:
    case False{}: "false"
    case True{}: "true"

def show(value: U32 & Bool & Char & String) -> IO(Unit):
  (number, truth, character, text) = value
  do IO<Unit>:
    Unit <- IO.print(U32.show(number))
    Unit <- IO.print(flag(truth))
    Unit <- IO.print(U32.show(Char.to_u32(character)))
    IO.print(text)

def main() -> IO(Unit):
  do IO<Unit>:
    result : Result<&1, &1, U32 & String, U32 & Bool & Char & String> <- make()
    value : U32 & Bool & Char & String <- IO.pass(U32 & Bool & Char & String, result)
    show(value)
"#;
const FOREIGN_BOXED_C: &str = r#"
static Term make_run(Env e, Term* fields, IoWork* work) {
  (void)fields; (void)work;
  Term tail = io_tup(e, term_pak(CID_CHR, 955), io_str(e, "ok\0tail", 7));
  Term pair = io_tup(e, term_pak(CID_TRUE, 0), tail);
  return io_done(e, io_tup(e, 42, pair));
}
static void __attribute__((constructor)) make_use(void) {
  io_eff(CID_MAKE, make_run, 0);
}
"#;

#[test]
fn foreign_boxed_result_tuple_boolean_and_character_follow_upstream_layout() {
    let fixture = Fixture::new();
    fixture.write("effect.c", FOREIGN_BOXED_C);
    success(
        &fixture.run(FOREIGN_BOXED, &[]),
        "42\ntrue\n955\nok\0tail\n",
    );
}

const FOREIGN_CALLBACK: &str = r#"import Base
def callback(f: U32 -> U32, x: U32) -> IO(U32):
  import "effect.c"
def adder(n: U32) -> U32 -> U32: x => U32.add(n, x)
def main() -> IO(Unit):
  do IO<Unit>:
    value : U32 <- callback(adder(5), 37)
    IO.print(U32.show(value))
"#;
const FOREIGN_CALLBACK_C: &str = r"
static Term callback_run(Env e, Term* fields, IoWork* work) {
  (void)work;
  Loc application = task_node(e, FID_CLO_APPLY, TERM_HOLE, 0, 0);
  e.mem[application] = fields[0];
  e.mem[application + 1] = fields[1];
  return corpus_eval(e.mem, term_tsk(FID_CLO_APPLY, application));
}
static void __attribute__((constructor)) callback_use(void) {
  io_eff(CID_CALLBACK, callback_run, 0);
}
";

#[test]
fn foreign_callback_uses_captured_bend_closure_through_upstream_task_bridge() {
    let fixture = Fixture::new();
    fixture.write("effect.c", FOREIGN_CALLBACK_C);
    success(&fixture.run(FOREIGN_CALLBACK, &[]), "42\n");
}

const SHARED_MODULE: &str = r#"import Base
def tick(x: U32) -> IO(U32): import "effect.c"
def times(x: U32) -> IO(U32): import "./effect.c"
"#;
const SHARED_SOURCE: &str = r"import Base
import foreign.bend as A
import ./foreign.bend as B
def main() -> IO(Unit):
  do IO<Unit>:
    x : U32 <- A.tick(10)
    Unit <- IO.print(U32.show(x))
    y : U32 <- B.times(20)
    Unit <- IO.print(U32.show(y))
    z : U32 <- B.tick(20)
    IO.print(U32.show(z))
";
const SHARED_C: &str = r"
static uint32_t calls;
static uint32_t initializations;
static Term tick_run(Env e, Term* fields, IoWork* work) {
  (void)e; (void)work;
  calls += 1;
  return initializations == 1 ? (Term)(calls + (uint32_t)fields[0]) : (Term)999;
}
static Term times_run(Env e, Term* fields, IoWork* work) {
  (void)e; (void)work;
  return initializations == 1 ? (Term)(2 * (uint32_t)fields[0]) : (Term)999;
}
static void __attribute__((constructor)) shared_use(void) {
  initializations += 1;
  io_eff(CID_TICK, tick_run, 0);
  io_eff(CID_TIMES, times_run, 0);
}
";

#[test]
fn aliases_and_shared_companion_keep_one_registration_and_one_static_state() {
    let fixture = Fixture::new();
    fixture.write("foreign.bend", SHARED_MODULE);
    fixture.write("effect.c", SHARED_C);
    success(&fixture.run(SHARED_SOURCE, &[]), "11\n40\n22\n");
}

#[test]
fn runtime_budget_failure_is_bounded_and_does_not_emit_a_partial_value() {
    let output = Fixture::new().run(CLOSURES, &["BEND_MAX_STEPS=2"]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("budget"));
}

fn wide_recursive_program(depth: &str) -> String {
    let fields = (0..80)
        .map(|index| format!("field{index}: U32"))
        .collect::<Vec<_>>()
        .join(", ");
    let values = ["0"; 80].join(", ");
    let bindings = (0..10)
        .map(|index| format!("      value{index} = {{Wide{{{values}}} : Wide}}"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "import Base\ntype Wide is Data: Wide{{{fields}}}\ndef loop(n: Nat) -> U32:\n  match n:\n    case 0n: 0\n    case 1n+p:\n{bindings}\n      loop(p)\ndef main() -> U32: loop({depth})\n"
    )
}

#[test]
fn wide_recursive_values_complete_or_exhaust_a_budget_without_native_stack_overflow() {
    success(
        &Fixture::new().run(&wide_recursive_program("3n"), &[]),
        "0\n",
    );
    // This checked input formerly exhausted the default Windows native stack:
    // each recursive body declared ten 80-word automatic constructor buffers.
    let output = Fixture::new().run(&wide_recursive_program("Nat.pow(2n, 8n)"), &[]);
    if output.status.success() {
        success(&output, "0\n");
    } else {
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("budget"));
    }
}

#[test]
fn a_wide_generated_frame_respects_the_host_allocation_budget() {
    let output = Fixture::new().run(
        &wide_recursive_program("1n"),
        &["BEND_MAX_HOST_BUFFER=4096"],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("allocation budget"));
}

#[test]
fn reachable_foreign_definition_requires_a_c_companion() {
    let fixture = Fixture::new();
    fixture.write("effect.js", "function unsupported(){return 42;}");
    let error = fixture.generate(
        "import Base\ndef unsupported() -> IO(U32): import \"effect.js\"\ndef main() -> IO(U32): unsupported()\n",
    ).unwrap_err();
    assert!(error.to_string().contains('C'));
}
