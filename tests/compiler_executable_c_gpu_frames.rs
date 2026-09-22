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
            "teamy-bend-c-gpu-frames-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn run(&self, helper_limit: u32, continuation_limit: u32, succeeds: bool) {
        let path = self.0.join("main.bend");
        fs::write(&path, NESTED_CONVERSION).unwrap();
        let generated =
            compile_executable_c(&check_executable(&load_executable(path).unwrap()).unwrap())
                .unwrap();
        assert!(generated.contains("#define TB_GPU_ENABLED 1"));
        let source = instrument(&generated, helper_limit, continuation_limit);
        let executable = executable_c_compiler::compile(
            &self.0,
            &source,
            &[
                "BEND_MAX_ALLOC=1048576",
                "BEND_MAX_FRAMES=128",
                "BEND_MAX_CONTINUATIONS=128",
                "BEND_CPU_WORKERS=4",
            ],
        );
        let output = executable_c_compiler::bounded(
            Command::new(executable)
                .current_dir(&self.0)
                .env("BEND_GPU", "on"),
            Duration::from_secs(45),
        );
        assert_eq!(
            output.status.code(),
            Some(i32::from(!succeeds)),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if succeeds {
            assert_eq!(output.stdout, b"7\n");
            assert!(output.stderr.is_empty(), "{:?}", output.stderr);
        } else {
            assert!(output.stdout.is_empty());
            assert_eq!(
                output.stderr,
                b"teamy-bend executable C: generated frame depth budget exhausted\n"
            );
        }
    }
}

fn instrument(generated: &str, helper_limit: u32, continuation_limit: u32) -> String {
    // Constrain only the device boundary, keeping the host's parsing, result
    // conversion and printing independently executable. The generated Bend
    // functions, shared conversions and real CUDA adapter stay unchanged.
    let helper = "control.helper_limit = BEND_MAX_FRAMES;";
    let completion = "control.helper_limit != BEND_MAX_FRAMES";
    let continuation = "control.frame_limit = BEND_MAX_CONTINUATIONS - tb_continuations;";
    let gpu_start = "static TBOutcome tb_gpu_execute(Env e, Term root) {";
    let cpu_start = "OUTLINE void tb_task_start_cpu(Env e, Term task, TBTaskRun *run) {";
    for marker in [helper, completion, continuation, gpu_start, cpu_start] {
        assert_eq!(generated.matches(marker).count(), 1, "{marker}");
    }
    let mut source = String::from(
        r"
static unsigned int gpu_entries, gpu_completions, cpu_marked_runs, helper_peak, helper_errors;
#define TB_GPU_COMPLETE(control, state, info) do { \
  (void)(state); (void)(info); ++gpu_completions; helper_peak = (control)->helper_peak; \
  if ((control)->helper_live != 0 || (control)->helper_depths != 0 \
      || (control)->live_frames != 0 || (control)->helper_calls == 0) ++helper_errors; \
} while (0)
",
    );
    source.push_str(
        &generated
            .replace(helper, &format!("control.helper_limit = {helper_limit}u;"))
            .replace(
                completion,
                &format!("control.helper_limit != {helper_limit}u"),
            )
            .replace(
                continuation,
                &format!("control.frame_limit = {continuation_limit}u;"),
            )
            .replace(gpu_start, &format!("{gpu_start}\n  ++gpu_entries;"))
            .replace(
                cpu_start,
                &format!(
                    "{cpu_start}\n  if (tb_gpu_marked((Fid)term_aux(task))) ++cpu_marked_runs;"
                ),
            )
            .replace(
                "static int tb_program_main(void)",
                "#define TB_NO_MAIN 1\nstatic int checked_main(void)",
            ),
    );
    writeln!(
        source,
        r"
int main(void) {{
  int status = checked_main();
  if (gpu_entries != 1 || cpu_marked_runs != 0 || helper_errors != 0) return 91;
  if (tb_cuda_info()->compilations != 1 || tb_cuda_info()->allocations != 5
      || tb_cuda_info()->launches < 6) return 92;
  if (tb_memory != NULL || tb_heap_meta != NULL || tb_host_current != NULL
      || tb_gpu_status != 0 || tb_cuda.initialized || tb_cuda.context != NULL
      || tb_cuda.module != NULL || tb_cuda.stream != NULL || tb_cuda.source != NULL
      || tb_cuda.driver != NULL || tb_cuda.compiler != NULL) return 93;
  for (u32 i = 0; i < TB_CUDA_BUFFERS; ++i)
    if (tb_cuda.buffers[i].address != 0 || tb_cuda.buffers[i].capacity != 0) return 94;
  if (status == 0) {{
    if (gpu_completions != 1 || helper_peak < 2 || helper_peak > {helper_limit}u
        || tb_live_words != 0 || tb_live_blocks != 0 || tb_tasks != 0
        || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0) return 95;
  }} else if (status != 1 || gpu_completions != 0
      || tb_cuda_error_class() != TB_CUDA_ERROR_RUNTIME) return 96;
  return status;
}}
"
    )
    .unwrap();
    source
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::env::var_os("TEAMY_BEND_KEEP_GPU_TESTS").is_some() {
            eprintln!("retained GPU frame test: {}", self.0.display());
            return;
        }
        let _removed = fs::remove_dir_all(&self.0);
    }
}

const NESTED_CONVERSION: &str = r"import Base
type FrameInner is Data: FrameInner{word: U32}
type FrameMiddle is Data: FrameMiddle{inner: FrameInner}
def inner(value: FrameInner) -> U32:
  match value:
    case FrameInner{word}: word
def middle(value: FrameMiddle) -> U32:
  match value:
    case FrameMiddle{nested}: inner(nested)
def apply(f: FrameMiddle -> U32, value: FrameMiddle) -> U32: f(value)
def main() -> U32: apply!(middle, FrameMiddle{FrameInner{7}})
";

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn cuda_nested_helpers_do_not_consume_persistent_continuation_capacity() {
    Fixture::new().run(128, 1, true);
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn cuda_nested_helpers_obey_their_independent_depth_limit() {
    Fixture::new().run(1, 128, false);
}
