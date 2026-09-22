// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory =
            std::env::temp_dir().join(format!("teamy-bend-gpu-progress-{}", std::process::id()));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn run(&self) {
        let path = self.0.join("main.bend");
        fs::write(&path, BEND).unwrap();
        let generated =
            compile_executable_c(&check_executable(&load_executable(path).unwrap()).unwrap())
                .unwrap();
        assert!(generated.contains("#define TB_GPU_ENABLED 1"));
        let executable = executable_c_compiler::compile(
            &self.0,
            &instrument(generated),
            &["BEND_MAX_ALLOC=1048576", "BEND_CPU_WORKERS=4"],
        );
        let output = executable_c_compiler::bounded(
            Command::new(executable)
                .current_dir(&self.0)
                .env("BEND_GPU", "auto"),
            Duration::from_secs(90),
        );
        fs::write(self.0.join("stdout.log"), &output.stdout).unwrap();
        fs::write(self.0.join("stderr.log"), &output.stderr).unwrap();
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"7\n7\n7\n");
        assert_eq!(
            output.stderr,
            b"teamy-bend executable C: invalid CUDA progress boundary\n\
teamy-bend executable C: CUDA task graph made no progress\n\
teamy-bend executable C: invalid CUDA completion boundary\n"
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::env::var_os("TEAMY_BEND_KEEP_GPU_TESTS").is_some() {
            eprintln!("retained GPU progress test: {}", self.0.display());
            return;
        }
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn instrument(mut generated: String) -> String {
    // Change only host observation boundaries. Every invocation executes the
    // same generated Bend bodies and embedded CUDA program at the same limits.
    let insertions = [
        (
            "static TBOutcome tb_gpu_execute(Env e, Term root) {",
            GPU_ENTRY,
        ),
        (
            "OUTLINE void tb_task_start_cpu(Env e, Term task, TBTaskRun *run) {",
            "\n  if (tb_gpu_marked((Fid)term_aux(task))) ++probe_cpu_replays;",
        ),
        (
            "    tb_gpu_require(tb_cuda_download(TB_GPU_CONTROL, 0, &control, sizeof(control)));",
            SNAPSHOT,
        ),
        (
            "static TB_NORETURN void tb_gpu_fail(const char *message) {",
            FAILURE,
        ),
    ];
    for (marker, insertion) in insertions {
        assert_eq!(generated.matches(marker).count(), 1, "{marker}");
        generated = generated.replace(marker, &format!("{marker}{insertion}"));
    }
    for (marker, counter) in [
        ("  tb_heap_commit(state.bump);", "probe_readbacks"),
        (
            "  tb_gpu_require(tb_cuda_download(TB_GPU_CORPUS, 0, e.mem, used_bytes));",
            "probe_downloads",
        ),
        (
            "  tb_gpu_require(tb_cuda_download(TB_GPU_METADATA, 0, tb_heap_meta, used_bytes));",
            "probe_downloads",
        ),
    ] {
        assert_eq!(generated.matches(marker).count(), 1, "{marker}");
        generated = generated.replace(marker, &format!("  ++{counter};\n{marker}"));
    }
    let destroy = "tb_cuda_cleanup_status(tb_cuda.cuCtxDestroy(tb_cuda.context), \"cuCtxDestroy during shutdown\");";
    assert_eq!(generated.matches(destroy).count(), 1);
    generated = generated.replace(destroy, DESTROY);
    let main = "static int tb_program_main(void)";
    assert_eq!(generated.matches(main).count(), 1);
    generated = generated.replace(main, "#define TB_NO_MAIN 1\nstatic int checked_main(void)");
    format!("{GLOBALS}{generated}{MAIN}")
}

const BEND: &str = r"import Base
def make(depth: Nat) -> Array<U32>: Array.new(U32, depth, 7)
def pick(pair: Array<U32> & U32) -> U32:
  (array, value) = pair
  value
def main() -> U32: pick(Array.get(U32, make!(12n), 4095))
";

const GLOBALS: &str = r"
static unsigned int probe_mode, probe_injected, probe_errors, probe_failures;
static unsigned int probe_entries, probe_completions, probe_cpu_replays;
static unsigned int probe_readbacks, probe_downloads, probe_imports;
static unsigned int probe_destroyed, probe_destroy_errors, probe_unfinished;
static unsigned int probe_readbacks_before, probe_downloads_before, probe_imports_before;
static unsigned long long probe_bump, probe_live_words, probe_live_blocks, probe_steps;
static unsigned long long probe_corpus_commit, probe_metadata_commit;
static void *probe_memory, *probe_metadata;
#define TB_GPU_OBSERVE(state, control) do { \
  (void)(state); (void)(control); ++probe_imports; \
} while (0)
#define TB_GPU_COMPLETE(control, state, info) do { \
  (void)(state); (void)(info); ++probe_completions; \
  if ((control)->primitive_starts != 1 || (control)->primitive_progress != 6144 \
      || (control)->primitive_yields == 0 || (control)->primitive_live != 0 \
      || (control)->primitive_requeues != (control)->primitive_yields) ++probe_errors; \
} while (0)
";

const GPU_ENTRY: &str = r"
  ++probe_entries;
  probe_bump = tb_bump; probe_live_words = tb_live_words; probe_live_blocks = tb_live_blocks;
  probe_steps = tb_steps; probe_memory = tb_memory; probe_metadata = tb_heap_meta;
  probe_corpus_commit = tb_corpus_storage.committed;
  probe_metadata_commit = tb_metadata_storage.committed;
  probe_readbacks_before = probe_readbacks; probe_downloads_before = probe_downloads;
  probe_imports_before = probe_imports;
";

const SNAPSHOT: &str = r"
    if (state.error == 0 && control.done == 0 && control.primitive_live != 0
        && control.primitive_progress > previous_progress) ++probe_unfinished;
    if (probe_mode != 0 && probe_injected == 0 && state.error == 0) {
      if (probe_mode == 1 && control.done == 0 && control.primitive_live != 0) {
        control.primitive_requeues = control.primitive_yields + 1;
        ++probe_injected;
      } else if (probe_mode == 2 && control.done == 0 && control.primitive_live != 0
          && previous_starts != 0 && previous_progress != 0
          && control.primitive_progress > previous_progress) {
        /* The real device made progress. Freeze the downloaded counters to
         * model a stalled snapshot without changing device execution. */
        state.steps = previous_steps;
        control.primitive_progress = previous_progress;
        control.primitive_requeues = previous_requeues;
        control.primitive_starts = previous_starts;
        control.primitive_yields = previous_yields;
        ++probe_injected;
      } else if (probe_mode == 3 && control.done == 1 && control.primitive_starts != 0) {
        control.primitive_live = 1;
        ++probe_injected;
      }
    }
";

const FAILURE: &str = r#"
  if (probe_mode != 0) {
    ++probe_failures;
    const char *expected = probe_mode == 1 ? "invalid CUDA progress boundary"
      : probe_mode == 2 ? "CUDA task graph made no progress" : "invalid CUDA completion boundary";
    if (strcmp(message, expected) != 0 || probe_injected != 1 || probe_unfinished == 0
        || probe_readbacks != probe_readbacks_before || probe_downloads != probe_downloads_before
        || probe_imports != probe_imports_before || tb_bump != probe_bump
        || tb_live_words != probe_live_words || tb_live_blocks != probe_live_blocks
        || tb_steps != probe_steps || tb_memory != probe_memory || tb_heap_meta != probe_metadata
        || tb_corpus_storage.committed != probe_corpus_commit
        || tb_metadata_storage.committed != probe_metadata_commit) ++probe_errors;
  } else ++probe_errors;
"#;

const DESTROY: &str = r#"{
      int status = tb_cuda.cuCtxDestroy(tb_cuda.context);
      if (status == 0) ++probe_destroyed;
      else ++probe_destroy_errors;
      tb_cuda_cleanup_status(status, "cuCtxDestroy during shutdown");
    }"#;

const MAIN: &str = r"
static bool probe_storage_released(const TBHostStorage *storage) {
  return storage->address == NULL && storage->capacity == 0 && storage->reserved == 0
    && storage->committed == 0 && storage->page_size == 0;
}
static bool probe_released(void) {
  if (!probe_storage_released(&tb_corpus_storage) || !probe_storage_released(&tb_metadata_storage)
      || tb_memory != NULL || tb_heap_meta != NULL || tb_host_current != NULL
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
  for (u32 mode = 1; mode <= 3; ++mode) {
    probe_mode = mode; probe_injected = 0; probe_unfinished = 0;
    int status = checked_main();
    u32 invocations = 2 * mode - 1;
    if (status != 1 || probe_injected != 1 || probe_errors != 0 || probe_failures != mode
        || probe_entries != invocations || probe_completions != mode - 1
        || probe_cpu_replays != 0 || probe_unfinished == 0) return 91;
    if (!probe_released() || probe_destroyed != invocations || probe_destroy_errors != 0)
      return 92;
    if (tb_cuda_info()->compilations != invocations || tb_cuda_info()->allocations != 5 * invocations)
      return 93;

    /* Only the host snapshot fault is disabled. Neither the Bend inputs nor
     * the generated source, compilation options, or device limits change. */
    probe_mode = 0; probe_unfinished = 0;
    status = checked_main();
    ++invocations;
    if (status != 0 || probe_errors != 0 || probe_entries != invocations
        || probe_completions != mode || probe_cpu_replays != 0 || probe_unfinished == 0
        || probe_readbacks != mode || probe_downloads != 2 * mode || probe_imports != mode)
      return 94;
    if (!probe_released() || probe_destroyed != invocations || probe_destroy_errors != 0)
      return 95;
    if (tb_cuda_info()->compilations != invocations || tb_cuda_info()->allocations != 5 * invocations)
      return 96;
    if (tb_live_words != 0 || tb_live_blocks != 0 || tb_tasks != 0
        || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0) return 97;
  }
  return 0;
}
";

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn malformed_stalled_and_pending_completion_snapshots_fail_before_import_and_recover() {
    Fixture::new().run();
}
