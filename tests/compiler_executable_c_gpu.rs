// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
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
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-c-gpu-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn run(
        &self,
        bend: &str,
        policy: &str,
        expected: &str,
        minimum_offloads: u32,
        minimum_forks: u32,
    ) {
        let path = self.0.join("main.bend");
        fs::write(&path, bend).unwrap();
        let generated =
            compile_executable_c(&check_executable(&load_executable(path).unwrap()).unwrap())
                .unwrap();
        assert!(generated.contains("#define TB_GPU_ENABLED 1"));
        let mut source = String::from(
            r"
static unsigned long long gpu_offloads, gpu_dispatches, gpu_forks, gpu_reuse_errors, gpu_parallel;
#define TB_GPU_COMPLETE(control, state, info) do { \
  (void)(state); ++gpu_offloads; gpu_dispatches += (control)->dispatches; \
  gpu_forks += (control)->forks; \
  if ((control)->peak_lanes > 1) ++gpu_parallel; \
  if ((info)->compilations != 1 || (info)->allocations != 5) ++gpu_reuse_errors; \
} while (0)
",
        );
        source.push_str(&generated.replace(
            "static int tb_program_main(void)",
            "#define TB_NO_MAIN 1\nstatic int checked_main(void)",
        ));
        writeln!(source, r"
int main(void) {{
  int status = checked_main();
  if (status != 0) return status;
  if (tb_live_words != 0 || tb_live_blocks != 0 || tb_tasks != 0
      || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0) return 91;
  if (gpu_offloads < {minimum_offloads}u || gpu_forks < {minimum_forks}u || gpu_reuse_errors != 0) return 92;
  if (gpu_offloads != 0 && gpu_dispatches == 0) return 93;
#if {minimum_forks} != 0
  if (gpu_parallel == 0) return 94;
#endif
  return 0;
}}
").unwrap();
        let executable = executable_c_compiler::compile(
            &self.0,
            &source,
            &["BEND_MAX_ALLOC=1048576", "BEND_CPU_WORKERS=4"],
        );
        let output = executable_c_compiler::bounded(
            Command::new(executable)
                .current_dir(&self.0)
                .env("BEND_GPU", policy),
            Duration::from_mins(1),
        );
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(
            output.stderr.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if std::env::var_os("TEAMY_BEND_KEEP_GPU_TESTS").is_some() {
            eprintln!("retained GPU test: {}", self.0.display());
            return;
        }
        let _removed = fs::remove_dir_all(&self.0);
    }
}

const RECURSIVE: &str = r"import Base
def branch(+d: Nat) -> U32:
  match d:
    case 0n: 3
    case 1n+p:
      left right = branch(p) branch(p)
      U32.add(left, right)
def main() -> U32:
  x = branch!(5n)
  y = branch!(5n)
  U32.add(U32.mul(x, 100), y)
";

#[test]
fn marked_recursive_program_keeps_cpu_semantics_when_gpu_is_off() {
    Fixture::new().run(RECURSIVE, "off", "9696\n", 0, 0);
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn generated_recursive_forks_execute_on_cuda_and_reuse_the_session() {
    Fixture::new().run(RECURSIVE, "on", "9696\n", 2, 2);
}

const OWNED_RESULT: &str = r#"import Base
type GpuReply is Data: GpuReply{left: U32, right: U32, text: String}
def cycle(n: Nat, left: U32, right: U32, text: String) -> GpuReply:
  match n:
    case 0n: GpuReply{left, right, text}
    case 1n+p: cycle(p, right, left, text)
def main() -> GpuReply:
  cycle!(U32.to_nat(5001), 7, 11, String.append("per", "muted"))
"#;

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn cuda_tail_calls_return_full_owned_results_to_the_cpu() {
    Fixture::new().run(OWNED_RESULT, "on", "GpuReply{11, 7, \"permuted\"}\n", 1, 0);
}

const CLOSURE_RESULT: &str = r"import Base
type FBox is Type: MkF{f: U32 -> U32}
def unbox(b: FBox, x: U32) -> U32:
  match b:
    case MkF{f}: f(x)
def main() -> U32:
  unbox!(MkF{y => U32.mul(y, 5)}, 4)
";

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn cuda_resolves_host_created_closures_inside_constructor_arguments() {
    Fixture::new().run(CLOSURE_RESULT, "on", "20\n", 1, 0);
}

const PARTIAL_MARK: &str = r"import Base
def walk(d: Nat, x: U32) -> U32:
  match d:
    case 0n: x
    case 1n+p: walk(p, U32.add(x, 1))
def use(f: U32 -> U32, x: U32) -> U32: f(x)
def main() -> U32:
  f = {walk!(4n) : U32 -> U32}
  use(f, 3)
";

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn a_partial_marked_ordinary_call_enters_cuda_after_its_last_argument() {
    Fixture::new().run(PARTIAL_MARK, "on", "7\n", 1, 0);
}

const SHARED_ARRAY: &str = r#"import Base
def change(array: Array<String>) -> Array<String> & String:
  Array.swap(String, array, 1, "changed")
def use(pair: Array<String> & Array<String>) -> Array<String> & Array<String> & String:
  (saved, copy) = pair
  changed = change!(copy)
  (saved, changed)
def main() -> Array<String> & Array<String> & String:
  use(Array.clone(String, Array.new(String, 1n, "saved")))
"#;

const NUMERIC_WORD: &str = r"import Base
def make(bits: U32) -> F32:
  match bits:
    case U32{word}: F32{word}
def walk(n: Nat, bits: U32) -> F32:
  match n:
    case 0n: make(bits)
    case 1n+p: walk(p, bits)
def both(+n: Nat, +bits: U32) -> U32:
  match n:
    case 0n:
      left right = walk(3n, bits) walk(3n, U32.add(bits, 1))
      U32.add(F32.bits(left), F32.bits(right))
    case 1n+p: both(p, bits)
def main() -> U32: both!(3n, 12345)
";

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn cuda_array_copy_on_write_preserves_the_parked_cpu_owner() {
    Fixture::new().run(
        SHARED_ARRAY,
        "on",
        "([\"saved\", \"saved\"], [\"saved\", \"changed\"], \"saved\")\n",
        1,
        0,
    );
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn cuda_word_conversion_reserves_all_32_nodes_before_building_the_word() {
    let fixture = Fixture::new();
    let path = fixture.0.join("main.bend");
    fs::write(&path, NUMERIC_WORD).unwrap();
    let generated =
        compile_executable_c(&check_executable(&load_executable(path).unwrap()).unwrap()).unwrap();
    assert!(generated.contains("tb_c_word(e,"));
    assert!(generated.contains("tb_device_corpus_reserve_pair"));
    fixture.run(NUMERIC_WORD, "on", "24691\n", 1, 1);
}

const TASK_JOIN: &str = r#"import Base
type TaskValue is Data: TaskValue{left: String, right: String}
def combine(a: TaskValue, b: TaskValue) -> TaskValue:
  match a:
    case TaskValue{left_a, right_a}:
      match b:
        case TaskValue{left_b, right_b}:
          TaskValue{String.append(left_a, left_b), String.append(right_a, right_b)}
def leaf(+text: String) -> TaskValue: TaskValue{text, text}
def first() -> Unit -> TaskValue: unit =>
  a b = leaf("a") leaf("b")
  combine(a, b)
def second() -> Unit -> TaskValue: unit =>
  a b = leaf("c") leaf("d")
  combine(a, b)
def both(left: Unit -> TaskValue, right: Unit -> TaskValue) -> TaskValue:
  a b = left(Unit{}) right(Unit{})
  combine(a, b)
def main() -> TaskValue: both!(first(), second())
"#;

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn cuda_nested_closure_task_joins_reserve_all_nodes_before_building_the_tasks() {
    let fixture = Fixture::new();
    let path = fixture.0.join("main.bend");
    fs::write(&path, TASK_JOIN).unwrap();
    let generated =
        compile_executable_c(&check_executable(&load_executable(path).unwrap()).unwrap()).unwrap();
    assert!(generated.contains("tb_c_join(e,"));
    assert!(generated.contains("tb_device_corpus_reserve_pair"));
    fixture.run(TASK_JOIN, "on", "TaskValue{\"abcd\", \"abcd\"}\n", 1, 1);
}

const MIXED_WORD_JOIN: &str = r"import Base
def pair_result() -> U32 & U32: (7, 8)
def combine(pair: U32 & U32, value: U32) -> U32:
  (left, right) = pair
  U32.add(U32.add(left, right), value)
def run(unit: Unit) -> U32:
  function = {x => U32.add(x, 9) : U32 -> U32}
  pair value = pair_result() function(1)
  combine(pair, value)
def main() -> U32: run!(Unit{})
";

#[test]
fn mixed_word_join_keeps_cpu_semantics_and_uses_the_bundle_builder() {
    let fixture = Fixture::new();
    let path = fixture.0.join("main.bend");
    fs::write(&path, MIXED_WORD_JOIN).unwrap();
    let generated =
        compile_executable_c(&check_executable(&load_executable(path).unwrap()).unwrap()).unwrap();
    assert!(generated.contains("tb_c_word_join_build(e,"));
    assert!(generated.contains("FID_CLO_APPLY"));
    fixture.run(MIXED_WORD_JOIN, "off", "25\n", 0, 0);
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn cuda_mixed_word_join_reserves_parent_and_heterogeneous_children_together() {
    let fixture = Fixture::new();
    let path = fixture.0.join("main.bend");
    fs::write(&path, MIXED_WORD_JOIN).unwrap();
    let generated =
        compile_executable_c(&check_executable(&load_executable(path).unwrap()).unwrap()).unwrap();
    assert!(generated.contains("tb_c_word_join_build(e,"));
    assert!(generated.contains("tb_device_corpus_reserve_task_children"));
    fixture.run(MIXED_WORD_JOIN, "on", "25\n", 1, 1);
}

const PENDING_ARGUMENT: &str = r"import Base
def plus_one(x: U32) -> U32: U32.add(x, 1)
def keep(x: U32) -> U32:
  +saved = x
  U32.add(plus_one(saved), saved)
def apply(f: U32 -> U32, x: U32) -> U32: f(x)
def main() -> U32: apply!(keep, 20)
";

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn a_cuda_boxed_callback_keeps_its_argument_after_a_pending_call() {
    Fixture::new().run(PENDING_ARGUMENT, "on", "41\n", 1, 0);
}
