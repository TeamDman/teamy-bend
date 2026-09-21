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
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-c-intrinsics-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn run(&self, source: &str, definitions: &[&str]) -> Output {
        fs::write(self.0.join("main.bend"), source).unwrap();
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        let generated = compile_executable_c(&checked).unwrap();
        let mut generated = generated.replace("int main(void)", "static int checked_main(void)");
        generated.push_str(
            r#"
int main(void) {
  int status = checked_main();
  if (status == 0 && (tb_live_words != 0 || tb_live_blocks != 0 || tb_tasks != 0
      || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0)) {
    (void)fputs("primitive program retained runtime owners\n", stderr);
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

const PRIMITIVE_LOOP: &str = r"import Base
def calculate(n: Nat, total: U32) -> U32:
  match n:
    case 0n: total
    case 1n+p: calculate(p, U32.add(U32.mul(total, 3), 1))
def main() -> U32: calculate(U32.to_nat(1000), 0)
";

#[test]
fn saturated_primitive_loop_runs_with_a_bounded_dispatch_budget() {
    // The dispatcher budget includes the recursive calls. Primitive arithmetic
    // itself must not create thunk/curried calls for every operand and iteration.
    success(
        &Fixture::new().run(
            PRIMITIVE_LOOP,
            &[
                "BEND_MAX_STEPS=18000",
                "BEND_MAX_CONTINUATIONS=16",
                "BEND_MAX_ALLOC=4096",
            ],
        ),
        "3923520912\n",
    );
}

const PARTIAL_NUMERIC: &str = r"import Base
def apply(f: U32 -> U32, value: U32) -> U32: f(value)
def apply_float(f: F32 -> F32, value: F32) -> F32: f(value)
def returned(prefix: U32) -> U32 -> U32:
  x => U32.add(U32.mul(prefix, 2), x)
def erased(-ignored: Nat, value: U32) -> U32: value
def main() -> U32 & U32 & U32:
  (apply(U32.add(40), 2),
    F32.bits(apply_float(F32.add(1.5), 2.25)),
    apply(returned(20), erased(Nat.pow(2n, 48n), 3)))
";

#[test]
fn partial_primitive_values_and_returned_closures_keep_erasure_and_captures() {
    success(
        &Fixture::new().run(PARTIAL_NUMERIC, &["BEND_MAX_ALLOC=4096"]),
        "(42, 1081081856, 43)\n",
    );
}

const OWNED_ARRAYS: &str = r#"import Base
def read(pair: Array<String> & String) -> List<String> & String:
  (array, old) = pair
  (Array.to_list(~String, array), old)
def alter(pair: Array<String> & Array<String>) -> (List<String> & String) & List<String>:
  (left, right) = pair
  update = Array.swap(String, left, 3)
  (read(update(String.append("n", "ew"))), Array.to_list(~String, right))
def main() -> (List<String> & String) & List<String>:
  alter(Array.clone(String, Array.new(String, 1n, String.append("o", "ld"))))
"#;

#[test]
fn direct_and_partial_array_primitives_preserve_owned_values_and_copy_on_write() {
    success(
        &Fixture::new().run(OWNED_ARRAYS, &["BEND_MAX_ALLOC=8192"]),
        "(([\"old\", \"new\"], \"old\"), [\"old\", \"old\"])\n",
    );
}

const NUMERIC_TEXT: &str = r#"import Base
def convert(prefix: String) -> F32 -> String:
  value => String.append(prefix, F32.show(F32.add(value, 0.5)))
def finish(value: Maybe<&2, F32>) -> String:
  match value:
    case None{}: "bad"
    case Some{x}: convert(String.append("v", "="), x)
def main() -> String: finish(F32.read(String.append("1", ".25")))
"#;

#[test]
fn direct_numeric_text_operations_transfer_strings_through_dynamic_closures() {
    success(
        &Fixture::new().run(NUMERIC_TEXT, &["BEND_MAX_ALLOC=8192"]),
        "\"v=1.75\"\n",
    );
}

#[test]
fn direct_primitive_arguments_preserve_left_to_right_failure_order() {
    let nat = "U32.from_nat(Nat.pow(2n, 48n))";
    let array = "discard(Array.new(U32, 63n, 0))";
    for (left, right, diagnostic) in [(nat, array, "Nat"), (array, nat, "array")] {
        let source = format!(
            "import Base\ndef discard(array: Array<U32>) -> U32: 0\ndef main() -> U32: U32.add({left}, {right})\n"
        );
        let output = Fixture::new().run(&source, &[]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(diagnostic),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn direct_tail_primitives_return_values_with_their_original_layouts() {
    for (kind, expression, expected) in [
        ("U32", "U32.add(4294967295, 2)", "1\n"),
        ("Nat", "Nat.mul(11n, 13n)", "143n\n"),
        ("F32", "F32.neg(0.0)", "-0.0\n"),
        ("Bool", "F32.is_lt(1.25, 2.5)", "True{}\n"),
        ("Cmp", "U32.cmp(9, 3)", "GT{}\n"),
        ("Nat & Nat", "Nat.divmod(17n, 5n)", "(3n, 2n)\n"),
        ("Array<U32>", "Array.new(U32, 1n, 7)", "[7, 7]\n"),
    ] {
        let source = format!("import Base\ndef main() -> {kind}: {expression}\n");
        success(&Fixture::new().run(&source, &[]), expected);
    }
}

const PRIMITIVE_FORK: &str = r"import Base
def calculate(n: Nat, total: U32) -> U32:
  match n:
    case 0n: total
    case 1n+p: calculate(p, U32.add(U32.mul(total, 3), 1))
def main() -> U32:
  left right = calculate(U32.to_nat(1000), 0) calculate(U32.to_nat(1000), 0)
  U32.add(left, right)
";

#[test]
fn generated_workers_and_joins_execute_direct_primitives_consistently() {
    for workers in [
        "BEND_CPU_WORKERS=1",
        "BEND_CPU_WORKERS=2",
        "BEND_CPU_WORKERS=4",
    ] {
        success(
            &Fixture::new().run(
                PRIMITIVE_FORK,
                &[workers, "BEND_MAX_STEPS=36000", "BEND_MAX_ALLOC=65536"],
            ),
            "3552074528\n",
        );
    }
}
