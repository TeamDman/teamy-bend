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
            "teamy-bend-c-gpu-off-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn run(&self, policy: &str, error_class: &str, succeeds: bool) {
        let path = self.0.join("main.bend");
        fs::write(&path, PERMUTATION).unwrap();
        let generated =
            compile_executable_c(&check_executable(&load_executable(path).unwrap()).unwrap())
                .unwrap();
        assert!(generated.contains("#define TB_GPU_ENABLED 1"));
        // Keep the complete real adapter body for the off case. Other cases
        // inject a classified failure before touching any CUDA library, so
        // policy and replay behavior are testable on every host.
        let signature = "static bool tb_cuda_libraries(void) {";
        let cpu_signature = "OUTLINE void tb_task_start_cpu(Env e, Term task, TBTaskRun *run) {";
        assert_eq!(generated.matches(signature).count(), 1);
        assert_eq!(generated.matches(cpu_signature).count(), 1);
        let mut source = String::from(
            r"
static unsigned int cuda_attempts, cpu_marked_runs;
static volatile int cuda_error_injection;
",
        );
        source.push_str(
            &generated
                .replace(
                    signature,
                    &format!(
                        "{signature}\n  ++cuda_attempts;\n  if (cuda_error_injection != TB_CUDA_ERROR_NONE) return tb_cuda_message(cuda_error_injection, \"test CUDA initialization failure\");"
                    ),
                )
                .replace(
                    cpu_signature,
                    &format!(
                        "{cpu_signature}\n  if (tb_gpu_marked((Fid)term_aux(task))) ++cpu_marked_runs;"
                    ),
                )
                .replace("static int tb_program_main(void)", "#define TB_NO_MAIN 1\nstatic int checked_main(void)"),
        );
        let attempts = u32::from(error_class != "TB_CUDA_ERROR_NONE");
        let expected_status = i32::from(!succeeds);
        writeln!(
            source,
            r"
int main(void) {{
  cuda_error_injection = {error_class};
  int status = checked_main();
  if (cuda_attempts != {attempts}u || tb_cuda_info()->compilations != 0
      || tb_cuda_info()->allocations != 0 || tb_cuda_info()->launches != 0
      || tb_cuda_error_class() != {error_class}) return 91;
  if (tb_memory != NULL || tb_heap_meta != NULL || tb_host_current != NULL
      || tb_cuda.initialized || tb_cuda.context != NULL || tb_cuda.module != NULL
      || tb_cuda.stream != NULL || tb_cuda.source != NULL
      || tb_cuda.driver != NULL || tb_cuda.compiler != NULL) return 92;
  if (status == 0 && (tb_live_words != 0 || tb_live_blocks != 0 || tb_tasks != 0
      || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0)) return 93;
  if (status != 0 && cpu_marked_runs != 0) return 94;
  return status;
}}
"
        )
        .unwrap();
        let executable = executable_c_compiler::compile(
            &self.0,
            &source,
            &[
                "BEND_MAX_TASKS=16",
                "BEND_MAX_CONTINUATIONS=16",
                "BEND_MAX_ALLOC=65536",
                "BEND_CPU_WORKERS=4",
            ],
        );
        let output = executable_c_compiler::bounded(
            Command::new(executable)
                .current_dir(&self.0)
                .env("BEND_GPU", policy),
            Duration::from_secs(30),
        );
        assert_eq!(
            output.status.code(),
            Some(expected_status),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if succeeds {
            assert_eq!(output.stdout, b"Shipment{11, 7, \"permuted\"}\n");
            assert!(
                output.stderr.is_empty(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        } else {
            assert!(output.stdout.is_empty());
            assert_eq!(
                output.stderr,
                b"teamy-bend executable C: test CUDA initialization failure\n"
            );
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

const PERMUTATION: &str = r#"import Base
type Shipment is Data: Shipment{left: U32, right: U32, text: String}

def cycle(n: Nat, left: U32, right: U32, text: String) -> Shipment:
  match n:
    case 0n: Shipment{left, right, text}
    case 1n+p: cycle!(p, right, left, text)

def main() -> Shipment:
  cycle!(U32.to_nat(513), 7, 11, String.append("per", "muted"))
"#;

#[test]
fn marked_tail_calls_reuse_cpu_runs_without_initializing_disabled_cuda() {
    Fixture::new().run("off", "TB_CUDA_ERROR_NONE", true);
}

#[test]
fn unavailable_cuda_is_probed_once_and_marked_tail_calls_reuse_cpu_runs() {
    Fixture::new().run("auto", "TB_CUDA_ERROR_UNAVAILABLE", true);
}

#[test]
fn forced_cuda_unavailability_fails_without_replaying_the_marked_body_on_cpu() {
    Fixture::new().run("on", "TB_CUDA_ERROR_UNAVAILABLE", false);
}

#[test]
fn auto_cuda_compilation_failure_does_not_fall_back_to_cpu() {
    Fixture::new().run("auto", "TB_CUDA_ERROR_COMPILE", false);
}

#[test]
fn auto_cuda_runtime_initialization_failure_does_not_fall_back_to_cpu() {
    Fixture::new().run("auto", "TB_CUDA_ERROR_RUNTIME", false);
}
