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
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-gpu-host-storage-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn run(&self, fail_readback: bool) {
        let path = self.0.join("main.bend");
        fs::write(&path, BEND).unwrap();
        let generated =
            compile_executable_c(&check_executable(&load_executable(path).unwrap()).unwrap())
                .unwrap();
        assert!(generated.contains("#define TB_GPU_ENABLED 1"));
        let source = instrument(generated, fail_readback);
        let executable = executable_c_compiler::compile(
            &self.0,
            &source,
            &["BEND_MAX_ALLOC=1048576", "BEND_CPU_WORKERS=4"],
        );
        let output = executable_c_compiler::bounded(
            Command::new(executable)
                .current_dir(&self.0)
                .env("BEND_GPU", "auto"),
            Duration::from_secs(45),
        );
        fs::write(self.0.join("stdout.log"), &output.stdout).unwrap();
        fs::write(self.0.join("stderr.log"), &output.stderr).unwrap();
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"7\n");
        let expected = if fail_readback {
            b"teamy-bend executable C: host corpus commitment failed\n".as_slice()
        } else {
            b"".as_slice()
        };
        assert_eq!(output.stderr, expected);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::env::var_os("TEAMY_BEND_KEEP_GPU_TESTS").is_some() {
            eprintln!("retained GPU host storage test: {}", self.0.display());
            return;
        }
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn instrument(mut generated: String, fail_readback: bool) -> String {
    // Only the host commitment/download boundaries are instrumented. The
    // generated Bend functions and embedded device program remain unchanged.
    let markers = [
        (
            "static TBOutcome tb_gpu_execute(Env e, Term root) {",
            GPU_ENTRY,
        ),
        (
            "OUTLINE void tb_task_start_cpu(Env e, Term task, TBTaskRun *run) {",
            "\n  if (tb_gpu_marked((Fid)term_aux(task))) ++probe_cpu_replays;",
        ),
        (
            "static bool tb_host_storage_commit(TBHostStorage *storage, size_t bytes) {",
            HOST_COMMIT,
        ),
    ];
    for (marker, insertion) in markers {
        assert_eq!(generated.matches(marker).count(), 1, "{marker}");
        generated = generated.replace(marker, &format!("{marker}{insertion}"));
    }
    let readback = "  tb_heap_commit(state.bump);";
    assert_eq!(generated.matches(readback).count(), 1);
    generated = generated.replace(readback, READBACK_COMMIT);
    for marker in [
        "  tb_gpu_require(tb_cuda_download(TB_GPU_CORPUS, 0, e.mem, used_bytes));",
        "  tb_gpu_require(tb_cuda_download(TB_GPU_METADATA, 0, tb_heap_meta, used_bytes));",
    ] {
        assert_eq!(generated.matches(marker).count(), 1, "{marker}");
        generated = generated.replace(marker, &format!("{BEFORE_DOWNLOAD}\n{marker}"));
    }
    let main = "static int tb_program_main(void)";
    assert_eq!(generated.matches(main).count(), 1);
    generated = generated.replace(main, "#define TB_NO_MAIN 1\nstatic int checked_main(void)");
    let mut source = String::from(PROBE_GLOBALS);
    source.push_str(&generated);
    writeln!(
        source,
        "\n#define PROBE_FAIL_READBACK {}",
        u8::from(fail_readback)
    )
    .unwrap();
    source.push_str(PROBE_MAIN);
    source
}

const BEND: &str = r"import Base
def make(depth: Nat) -> Array<U32>: Array.new(U32, depth, 7)
def pick(pair: Array<U32> & U32) -> U32:
  (array, value) = pair
  value
def main() -> U32: pick(Array.get(U32, make!(12n), 4095))
";

const PROBE_GLOBALS: &str = r"
static unsigned int probe_fail_readback, probe_readback_active, probe_injected;
static unsigned int probe_gpu_entries, probe_gpu_completions, probe_cpu_replays;
static unsigned int probe_commit_calls, probe_downloads, probe_errors, probe_partial_commit;
static unsigned long long probe_corpus_before, probe_metadata_before, probe_bump_before;
static void *probe_corpus_pointer, *probe_metadata_pointer;
#define TB_GPU_OBSERVE(state, control) do { \
  u64 owned = 0; \
  (void)(state); \
  for (u32 i = 0; i < (control)->root_count; ++i) owned += (control)->root_owned[i]; \
  if (owned == 0) ++probe_errors; \
} while (0)
#define TB_GPU_COMPLETE(control, state, info) do { \
  (void)(control); (void)(state); (void)(info); ++probe_gpu_completions; \
} while (0)
";

const GPU_ENTRY: &str = r"
  ++probe_gpu_entries;
  probe_corpus_pointer = tb_memory; probe_metadata_pointer = tb_heap_meta;
  probe_corpus_before = tb_corpus_storage.committed;
  probe_metadata_before = tb_metadata_storage.committed;
  probe_bump_before = tb_bump;
  if (tb_corpus_storage.address != tb_memory || tb_metadata_storage.address != tb_heap_meta
      || probe_corpus_before == 0 || probe_metadata_before == 0
      || tb_corpus_storage.committed >= tb_corpus_storage.capacity
      || tb_metadata_storage.committed >= tb_metadata_storage.capacity) ++probe_errors;
";

const HOST_COMMIT: &str = r"
  if (probe_readback_active != 0) {
    ++probe_commit_calls;
    if (probe_fail_readback != 0 && storage == &tb_metadata_storage
        && bytes > storage->committed) {
      ++probe_injected;
      if (tb_corpus_storage.committed >= bytes
          && tb_corpus_storage.committed > probe_corpus_before
          && tb_metadata_storage.committed == probe_metadata_before
          && tb_bump == probe_bump_before) ++probe_partial_commit;
      else ++probe_errors;
      return false;
    }
  }
";

const READBACK_COMMIT: &str = r"
  if (used_bytes <= probe_corpus_before + 8192u
      || used_bytes <= probe_metadata_before + 8192u
      || tb_corpus_storage.committed != probe_corpus_before
      || tb_metadata_storage.committed != probe_metadata_before
      || tb_bump != probe_bump_before) ++probe_errors;
  probe_readback_active = 1;
  tb_heap_commit(state.bump);
  probe_readback_active = 0;
";

const BEFORE_DOWNLOAD: &str = r"
  ++probe_downloads;
  if (tb_corpus_storage.committed < used_bytes || tb_metadata_storage.committed < used_bytes
      || tb_corpus_storage.committed <= probe_corpus_before
      || tb_metadata_storage.committed <= probe_metadata_before
      || tb_memory != probe_corpus_pointer || tb_heap_meta != probe_metadata_pointer
      || tb_corpus_storage.address != probe_corpus_pointer
      || tb_metadata_storage.address != probe_metadata_pointer) ++probe_errors;
";

const PROBE_MAIN: &str = r"
static bool probe_storage_released(const TBHostStorage *storage) {
  return storage->address == NULL && storage->capacity == 0 && storage->reserved == 0
    && storage->committed == 0 && storage->page_size == 0;
}
static bool probe_resources_released(void) {
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
  probe_fail_readback = PROBE_FAIL_READBACK;
  int status = checked_main();
  if (PROBE_FAIL_READBACK) {
    if (status != 1 || probe_gpu_entries != 1 || probe_gpu_completions != 0
        || probe_injected != 1 || probe_partial_commit != 1 || probe_commit_calls != 2
        || probe_downloads != 0 || probe_cpu_replays != 0 || probe_errors != 0) return 91;
    if (!probe_resources_released()) return 92;
    // The identical program and CUDA source run again in this process. Only
    // the host readback fault is removed, after the first invocation cleaned up.
    probe_fail_readback = 0; probe_readback_active = 0;
    status = checked_main();
  }
  unsigned int invocations = 1u + PROBE_FAIL_READBACK;
  if (status != 0 || probe_gpu_entries != invocations || probe_gpu_completions != 1
      || probe_commit_calls != 2u * invocations || probe_downloads != 2
      || probe_cpu_replays != 0 || probe_errors != 0) return 93;
  if (tb_cuda_info()->compilations != invocations || tb_cuda_info()->allocations != 5u * invocations
      || tb_cuda_info()->launches < 6u * invocations) return 94;
  if (!probe_resources_released()) return 95;
  if (tb_live_words != 0 || tb_live_blocks != 0 || tb_tasks != 0
      || tb_continuations != 0 || tb_frames != 0 || tb_depth != 0) return 96;
  return 0;
}
";

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn gpu_owned_array_grows_host_commitment_before_readback_without_moving_storage() {
    Fixture::new().run(false);
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn gpu_readback_second_commit_failure_aborts_without_replay_and_allows_a_fresh_invocation() {
    Fixture::new().run(true);
}
