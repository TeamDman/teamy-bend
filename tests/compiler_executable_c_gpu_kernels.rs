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
const KERNELS: [&str; 5] = [
    "tb_device_initialize",
    "tb_device_tables_initialize",
    "tb_device_tasks_begin",
    "tb_device_tasks_commit",
    "tb_device_tasks_step",
];

struct Fixture {
    directory: PathBuf,
    executable: PathBuf,
    cache: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-gpu-kernels-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("probe.c"), FOREIGN).unwrap();
        let path = directory.join("main.bend");
        fs::write(&path, PROGRAM).unwrap();
        let checked = check_executable(&load_executable(path).unwrap()).unwrap();
        let generated = compile_executable_c(&checked).unwrap();
        let entry = "int main(int argc, char **argv)";
        assert_eq!(generated.matches(entry).count(), 1);
        let mut source =
            generated.replace(entry, "static int kernel_test_main(int argc, char **argv)");
        source.push_str(PROBE);
        let executable = executable_c_compiler::compile(
            &directory,
            &source,
            &["BEND_MAX_ALLOC=1048576", "BEND_CPU_WORKERS=4"],
        );
        let mut cache = executable.as_os_str().to_os_string();
        cache.push(".gpu");
        Self {
            directory,
            executable,
            cache: cache.into(),
        }
    }

    fn run(&self, arguments: &[&str]) -> Output {
        executable_c_compiler::bounded(
            Command::new(&self.executable)
                .args(arguments)
                .current_dir(&self.directory)
                .env("BEND_GPU", "off"),
            Duration::from_secs(45),
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::env::var_os("TEAMY_BEND_KEEP_GPU_TESTS").is_some() {
            eprintln!(
                "retained GPU kernel contract test: {}",
                self.directory.display()
            );
            return;
        }
        let _removed = fs::remove_dir_all(&self.directory);
    }
}

#[derive(Debug)]
struct Report {
    counters: [u64; 8],
    diagnostics: Vec<String>,
}

impl Report {
    fn read(output: &Output) -> Self {
        let mut reports = Vec::new();
        let mut diagnostics = Vec::new();
        for line in String::from_utf8_lossy(&output.stderr).lines() {
            if let Some(values) = line.strip_prefix("CONTRACT ") {
                reports.push(
                    values
                        .split_whitespace()
                        .map(|value| value.parse::<u64>().unwrap())
                        .collect::<Vec<_>>()
                        .try_into()
                        .unwrap(),
                );
            } else {
                diagnostics.push(line.to_owned());
            }
        }
        assert_eq!(reports.len(), 1, "{output:?}");
        Self {
            counters: reports[0],
            diagnostics,
        }
    }
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn driver_loadable_cache_missing_each_required_kernel_fails_before_foreign_effects() {
    let fixture = Fixture::new();
    let prebuilt = fixture.run(&["--gpu-build"]);
    assert_eq!(prebuilt.status.code(), Some(0), "{prebuilt:?}");
    assert!(prebuilt.stdout.is_empty(), "{prebuilt:?}");
    assert_eq!(Report::read(&prebuilt).counters, [1, 0, 0, 0, 1, 0, 0, 1]);
    let original = fs::read(&fixture.cache).unwrap();
    for missing in KERNELS {
        let seeded = fixture.run(&["--test-wrong-module", missing]);
        assert_eq!(seeded.status.code(), Some(0), "{seeded:?}");
        assert!(seeded.stdout.is_empty(), "{seeded:?}");
        let seeded = Report::read(&seeded);
        assert_eq!(seeded.counters, [1, 0, 0, 0, 1, 0, 0, 1]);
        assert!(seeded.diagnostics.is_empty(), "{seeded:?}");
        let mut wrong = fs::read(&fixture.cache).unwrap();
        assert_eq!(&wrong[..16], &original[..16]);
        assert_ne!(&wrong[56..88], &original[56..88]);
        // Keep the valid cubin payload, payload checksum, lengths and format.
        // Give it the requested Bend source identity so validation reaches
        // the actual driver-loaded module rather than rejecting its header.
        wrong[24..56].copy_from_slice(&original[24..56]);
        fs::write(&fixture.cache, &wrong).unwrap();
        fs::write(
            fixture.directory.join(format!("missing-{missing}.gpu")),
            &wrong,
        )
        .unwrap();

        let failed = fixture.run(&["--gpu", "on"]);
        assert_eq!(failed.status.code(), Some(1), "{failed:?}");
        assert!(
            failed.stdout.is_empty(),
            "foreign/main effects before rejection: {failed:?}"
        );
        let failed = Report::read(&failed);
        assert_eq!(failed.counters, [0, 0, 0, 1, 0, 0, 0, 1]);
        assert_eq!(failed.diagnostics.len(), 1, "{failed:?}");
        assert!(failed.diagnostics[0].contains(missing), "{failed:?}");
        assert!(
            failed.diagnostics[0].contains("required CUDA kernel"),
            "{failed:?}"
        );
        assert_eq!(fs::read(&fixture.cache).unwrap(), wrong);
    }

    fs::write(&fixture.cache, &original).unwrap();
    let warmed = fixture.run(&["--gpu", "on"]);
    assert_eq!(warmed.status.code(), Some(0), "{warmed:?}");
    assert_eq!(warmed.stdout, b"initialized\nmain\n42\n");
    let warmed = Report::read(&warmed);
    let [
        compiled,
        hits,
        misses,
        rejected,
        writes,
        allocations,
        launches,
        clean,
    ] = warmed.counters;
    assert_eq!((compiled, hits, misses, rejected, writes), (0, 1, 0, 0, 0));
    assert_eq!((allocations, clean), (5, 1));
    assert!(launches >= 6, "{warmed:?}");
    assert!(warmed.diagnostics.is_empty(), "{warmed:?}");
    assert_eq!(fs::read(&fixture.cache).unwrap(), original);
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn fresh_module_contract_failure_preserves_the_previously_published_cache() {
    let fixture = Fixture::new();
    let prebuilt = fixture.run(&["--gpu-build"]);
    assert_eq!(prebuilt.status.code(), Some(0), "{prebuilt:?}");
    let original = fs::read(&fixture.cache).unwrap();
    let rejected = fixture.run(&["--test-prebuild-contract"]);
    assert_eq!(rejected.status.code(), Some(0), "{rejected:?}");
    assert!(rejected.stdout.is_empty(), "{rejected:?}");
    let rejected = Report::read(&rejected);
    assert_eq!(rejected.counters, [1, 0, 0, 0, 0, 0, 0, 1]);
    assert_eq!(rejected.diagnostics.len(), 1, "{rejected:?}");
    assert!(rejected.diagnostics[0].contains("needed"), "{rejected:?}");
    assert_eq!(fs::read(&fixture.cache).unwrap(), original);
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn an_initialized_cuda_module_revalidates_a_changed_caller_kernel_contract() {
    let fixture = Fixture::new();
    let rejected = fixture.run(&["--test-existing-contract"]);
    assert_eq!(rejected.status.code(), Some(0), "{rejected:?}");
    assert!(rejected.stdout.is_empty(), "{rejected:?}");
    let rejected = Report::read(&rejected);
    assert_eq!(rejected.counters, [1, 0, 0, 0, 0, 0, 0, 1]);
    assert_eq!(rejected.diagnostics.len(), 1, "{rejected:?}");
    assert!(rejected.diagnostics[0].contains("needed"), "{rejected:?}");
    assert!(!fixture.cache.exists());
}

const PROGRAM: &str = r#"import Base
def observe() -> IO(U32): import "probe.c"
def value(x: U32) -> U32: U32.add(x, 1)
def main() -> IO(Unit):
  do IO<Unit>:
    start : U32 <- observe()
    IO.print(U32.show(value!(start)))
"#;

const FOREIGN: &str = r#"
static Term observe_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields; (void)work;
  (void)fputs("main\n", stdout);
  return (Term)41;
}
static void __attribute__((constructor)) observe_use(void) {
  (void)fputs("initialized\n", stdout);
  io_eff(CID_OBSERVE, observe_run, 0);
}
"#;

const PROBE: &str = r#"
static int kernel_test_wrong_module(const char *missing) {
  static const char *const names[] = {
    "tb_device_initialize", "tb_device_tables_initialize", "tb_device_tasks_begin",
    "tb_device_tasks_commit", "tb_device_tasks_step"
  };
  char source[1024]; size_t used = 0;
  source[0] = 0;
  for (size_t i = 0; i < sizeof(names) / sizeof(names[0]); ++i) {
    if (strcmp(names[i], missing) == 0) continue;
    int wrote = snprintf(source + used, sizeof(source) - used,
      "extern \"C\" __global__ void %s(void) {}\n", names[i]);
    if (wrote < 0 || (size_t)wrote >= sizeof(source) - used) return 90;
    used += (size_t)wrote;
  }
  char *path = tb_cli_cache_path();
  bool configured = path != NULL && tb_cuda_set_cache_path(path); free(path);
  if (!configured || !tb_cuda_build_cache(source) || !tb_cuda.initialized) return 91;
  /* This generic adapter load succeeded. Confirm the selected omission while
   * its context is current; the other four entrypoints must be real functions. */
  if (!tb_cuda_push()) return 92;
  for (size_t i = 0; i < sizeof(names) / sizeof(names[0]); ++i) {
    void *function = NULL;
    int status = tb_cuda.cuModuleGetFunction(&function, tb_cuda.module, names[i]);
    if ((strcmp(names[i], missing) == 0) != (status != 0)) return 93;
  }
  if (!tb_cuda_pop(true)) return 94;
  tb_cuda_shutdown(); return 0;
}
static int kernel_test_contract(bool existing) {
  static const char source[] = "extern \"C\" __global__ void probe(void) {}";
  static const char *const correct[] = {"probe"};
  static const char *const missing[] = {"needed"};
  if (existing) {
    if (!tb_cuda_initialize(source)
        || !tb_cuda_initialize_kernels(source, false, correct, 1)) return 96;
  } else {
    char *path = tb_cli_cache_path();
    bool configured = path != NULL && tb_cuda_set_cache_path(path); free(path);
    if (!configured) return 97;
  }
  if (tb_cuda_initialize_kernels(source, !existing, missing, 1)
      || tb_cuda_error_class() != TB_CUDA_ERROR_RUNTIME) return 98;
  (void)fprintf(stderr, "%s\n", tb_cuda_error());
  /* A rejected new contract does not consume or silently replace an existing
   * caller-owned module. Its original contract still validates without NVRTC. */
  if (existing && !tb_cuda_initialize_kernels(source, false, correct, 1)) return 99;
  tb_cuda_shutdown(); return 0;
}
int main(int argc, char **argv) {
  int status;
  if (argc == 3 && strcmp(argv[1], "--test-wrong-module") == 0)
    status = kernel_test_wrong_module(argv[2]);
  else if (argc == 2 && strcmp(argv[1], "--test-prebuild-contract") == 0)
    status = kernel_test_contract(false);
  else if (argc == 2 && strcmp(argv[1], "--test-existing-contract") == 0)
    status = kernel_test_contract(true);
  else status = kernel_test_main(argc, argv);
  bool clean = !tb_cuda.initialized && tb_cuda.context == NULL && tb_cuda.module == NULL
    && tb_cuda.stream == NULL && tb_cuda.source == NULL && tb_cuda.cache_path == NULL
    && tb_cuda.driver == NULL && tb_cuda.compiler == NULL && tb_gpu_status == 0
    && tb_memory == NULL && tb_heap_meta == NULL && tb_host_current == NULL
    && tb_failure_guard == NULL && tb_live_words == 0 && tb_live_blocks == 0
    && tb_tasks == 0 && tb_continuations == 0 && tb_frames == 0 && tb_depth == 0;
  for (unsigned int i = 0; i < TB_CUDA_BUFFERS; ++i)
    if (tb_cuda.buffers[i].address != 0 || tb_cuda.buffers[i].capacity != 0) clean = false;
  (void)fprintf(stderr, "CONTRACT %llu %llu %llu %llu %llu %llu %llu %u\n",
    (unsigned long long)tb_cuda.info.compilations, (unsigned long long)tb_cuda.info.cache_hits,
    (unsigned long long)tb_cuda.info.cache_misses, (unsigned long long)tb_cuda.info.cache_rejections,
    (unsigned long long)tb_cuda.info.cache_writes, (unsigned long long)tb_cuda.info.allocations,
    (unsigned long long)tb_cuda.info.launches, (unsigned int)clean);
  return clean ? status : 95;
}
"#;
