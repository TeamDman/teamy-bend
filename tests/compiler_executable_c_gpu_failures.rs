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
            "teamy-bend-c-gpu-failures-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn source(&self, bend: &str, minimum_lanes: u32) -> String {
        let path = self.0.join("main.bend");
        fs::write(&path, bend).unwrap();
        let generated =
            compile_executable_c(&check_executable(&load_executable(path).unwrap()).unwrap())
                .unwrap();
        assert!(generated.contains("#define TB_GPU_ENABLED 1"));

        // Instrument host boundaries only. Both invocations run the same
        // unmodified generated Bend functions and the real CUDA adapter.
        let cpu_start = "OUTLINE void tb_task_start_cpu(Env e, Term task, TBTaskRun *run) {";
        let gpu_start = "static TBOutcome tb_gpu_execute(Env e, Term root) {";
        let destroy = "tb_cuda_cleanup_status(tb_cuda.cuCtxDestroy(tb_cuda.context), \"cuCtxDestroy during shutdown\");";
        let failure = "if (state.error != 0) {";
        for marker in [
            cpu_start,
            gpu_start,
            destroy,
            failure,
            "static int tb_program_main(void)",
        ] {
            assert_eq!(generated.matches(marker).count(), 1, "{marker}");
        }
        let mut source = String::from(
            r"
static unsigned int probe_recover, gpu_entries, gpu_completions, cpu_marked_runs;
static unsigned int gpu_contexts_destroyed, gpu_context_destroy_errors;
static unsigned int device_errors, failed_peak_lanes;
#define TB_GPU_COMPLETE(control, state, info) do { \
  (void)(control); (void)(state); (void)(info); ++gpu_completions; \
} while (0)
",
        );
        source.push_str(
            &generated
                .replace(
                    cpu_start,
                    &format!(
                        "{cpu_start}\n  if (tb_gpu_marked((Fid)term_aux(task))) ++cpu_marked_runs;"
                    ),
                )
                .replace(
                    gpu_start,
                    &format!(
                        r#"{gpu_start}
  ++gpu_entries;
  if (probe_recover != 0) {{
    /* Supply the base-case input at the public word-task boundary. The
     * generated function, device source, limits, and scheduler stay identical. */
    if (fid_arity((Fid)term_aux(root)) != 1)
      err_fail("recovery probe expected one Nat argument");
    e.mem[term_loc(root)] = 0;
    tb_mark_raw(e, term_loc(root), 1);
  }}"#
                    ),
                )
                .replace(
                    destroy,
                    r#"{
      int destroyed = tb_cuda.cuCtxDestroy(tb_cuda.context);
      if (destroyed == 0) ++gpu_contexts_destroyed;
      else ++gpu_context_destroy_errors;
      tb_cuda_cleanup_status(destroyed, "cuCtxDestroy during shutdown");
    }"#,
                )
                .replace(
                    failure,
                    &format!(
                        "{failure}\n      ++device_errors; failed_peak_lanes = control.peak_lanes;"
                    ),
                )
                .replace(
                    "static int tb_program_main(void)",
                    "#define TB_NO_MAIN 1\nstatic int checked_main(void)",
                ),
        );
        Self::append_probe_main(&mut source, minimum_lanes);
        source
    }

    fn append_probe_main(source: &mut String, minimum_lanes: u32) {
        writeln!(source, "\n#define PROBE_MINIMUM_LANES {minimum_lanes}u").unwrap();
        source.push_str(
            r"
static bool probe_handles_released(void) {
  if (tb_memory != NULL || tb_heap_meta != NULL || tb_host_current != NULL
      || tb_failure_guard != NULL || tb_task_current != NULL
      || tb_cpu_active || tb_cpu_pending != 0 || tb_cpu.created != 0 || tb_cpu.host != NULL
      || tb_gpu_status != 0 || tb_cuda.initialized || tb_cuda.context != NULL
      || tb_cuda.module != NULL || tb_cuda.stream != NULL || tb_cuda.source != NULL
      || tb_cuda.driver != NULL || tb_cuda.compiler != NULL) return false;
  for (u32 i = 0; i < TB_CUDA_BUFFERS; ++i)
    if (tb_cuda.buffers[i].address != 0 || tb_cuda.buffers[i].capacity != 0) return false;
  return true;
}
int main(void) {
  int status = checked_main();
  if (status != 1 || gpu_entries != 1 || gpu_completions != 0 || cpu_marked_runs != 0)
    return 91;
  if (tb_cuda_error_class() != TB_CUDA_ERROR_RUNTIME || tb_cuda_info()->compilations != 1
      || tb_cuda_info()->allocations != 5 || tb_cuda_info()->launches < 6) return 92;
  if (!probe_handles_released() || gpu_contexts_destroyed != 1 || gpu_context_destroy_errors != 0)
    return 93;
  if (device_errors != 1 || failed_peak_lanes < PROBE_MINIMUM_LANES) return 98;

  /* The failed invocation discards its context and all device storage.
   * Changing only the input must not require replacing generated functions. */
  u64 failed_launches = tb_cuda_info()->launches;
  probe_recover = 1;
  status = checked_main();
  if (status != 0 || gpu_entries != 2 || gpu_completions != 1 || cpu_marked_runs != 0)
    return 94;
  if (tb_cuda_error_class() != TB_CUDA_ERROR_NONE || tb_cuda_info()->compilations != 2
      || tb_cuda_info()->allocations != 10 || tb_cuda_info()->launches <= failed_launches)
    return 95;
  if (!probe_handles_released() || gpu_contexts_destroyed != 2 || gpu_context_destroy_errors != 0)
    return 96;
  if (tb_live_words != 0 || tb_live_blocks != 0 || tb_tasks != 0
      || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0) return 97;
  return 0;
}
",
        );
    }

    fn run(&self, bend: &str, budget: &str, expected: &str, diagnostic: &str, minimum_lanes: u32) {
        let source = self.source(bend, minimum_lanes);
        let executable = executable_c_compiler::compile(
            &self.0,
            &source,
            &["BEND_MAX_ALLOC=1048576", "BEND_CPU_WORKERS=4", budget],
        );
        let output = executable_c_compiler::bounded(
            Command::new(executable)
                .current_dir(&self.0)
                .env("BEND_GPU", "auto"),
            Duration::from_secs(45),
        );
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, expected.as_bytes());
        // Device budgets report their actual Bend error without poisoning CUDA.
        // Counters above require device execution, cleanup and a fresh offload.
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            format!("teamy-bend executable C: {diagnostic}\n")
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::env::var_os("TEAMY_BEND_KEEP_GPU_TESTS").is_some() {
            eprintln!("retained GPU failure test: {}", self.0.display());
            return;
        }
        let _removed = fs::remove_dir_all(&self.0);
    }
}

const INFINITE_TAIL: &str = r"import Base
@unsafe
def spin(n: Nat) -> U32:
  match n:
    case 0n: 7
    case 1n+p: spin(Nat.add(p, 1n))
def main() -> U32: spin!(1n)
";

const RECURSIVE_FORK: &str = r"import Base
def branch(+d: Nat) -> U32:
  match d:
    case 0n: 3
    case 1n+p:
      left right = branch(p) branch(p)
      U32.add(left, right)
def main() -> U32: branch!(8n)
";

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn a_device_step_limit_stops_an_unsafe_tail_loop_and_allows_a_clean_invocation() {
    Fixture::new().run(
        INFINITE_TAIL,
        "BEND_MAX_STEPS=512",
        "7\n",
        "evaluation budget exhausted",
        0,
    );
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn a_device_task_limit_stops_recursive_forks_and_allows_a_clean_invocation() {
    Fixture::new().run(
        RECURSIVE_FORK,
        "BEND_MAX_TASKS=16",
        "3\n",
        "task budget exhausted",
        0,
    );
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn a_device_scratch_limit_stops_recursive_forks_and_allows_a_clean_invocation() {
    Fixture::new().run(
        RECURSIVE_FORK,
        "BEND_MAX_GPU_SCRATCH=16384",
        "3\n",
        "device scratch allocation budget exhausted",
        0,
    );
}

const PARALLEL_FAILURE: &str = r"import Base
@unsafe
def spin(n: Nat) -> U32:
  match n:
    case 0n: 7
    case 1n+p: spin(Nat.add(p, 1n))

def parallel(n: Nat) -> U32:
  match n:
    case 0n: 7
    case 1n+p:
      left right = spin(1n) spin(1n)
      U32.add(left, right)

def main() -> U32: parallel!(1n)
";

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn a_device_failure_cancels_parallel_lanes_without_poisoning_the_next_invocation() {
    Fixture::new().run(
        PARALLEL_FAILURE,
        "BEND_MAX_STEPS=4096",
        "7\n",
        "evaluation budget exhausted",
        2,
    );
}
