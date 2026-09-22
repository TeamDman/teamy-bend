// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::fs;
use std::path::Path;
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

struct Fixture {
    directory: PathBuf,
    executable: PathBuf,
    working_directory: PathBuf,
}

impl Fixture {
    fn new(definitions: &[&str]) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-generated-cache-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let working_directory = directory.join("different working directory");
        fs::create_dir(&working_directory).unwrap();
        fs::write(directory.join("probe.c"), FOREIGN).unwrap();
        let path = directory.join("main.bend");
        fs::write(&path, PROGRAM).unwrap();
        let checked = check_executable(&load_executable(path).unwrap()).unwrap();
        let generated = compile_executable_c(&checked).unwrap();
        assert!(generated.contains("#define TB_GPU_ENABLED 1"));
        let source = instrument(&generated);
        let mut limits = vec!["BEND_MAX_ALLOC=1048576", "BEND_CPU_WORKERS=4"];
        limits.extend_from_slice(definitions);
        let executable = executable_c_compiler::compile(&directory, &source, &limits);
        Self {
            directory,
            executable,
            working_directory,
        }
    }

    fn run(&self, executable: &Path, arguments: &[&str]) -> Output {
        executable_c_compiler::bounded(
            Command::new(executable)
                .args(arguments)
                .current_dir(&self.working_directory)
                .env("BEND_GPU", "off"),
            Duration::from_secs(45),
        )
    }

    fn relocate(&self) -> PathBuf {
        let relocated = self.directory.join("déplacé λ");
        fs::create_dir(&relocated).unwrap();
        let executable = relocated.join(format!(
            "programme résultat{}",
            std::env::consts::EXE_SUFFIX
        ));
        fs::copy(&self.executable, &executable).unwrap();
        fs::copy(cache_path(&self.executable), cache_path(&executable)).unwrap();
        executable
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::env::var_os("TEAMY_BEND_KEEP_GPU_TESTS").is_some() {
            eprintln!(
                "retained generated GPU cache test: {}",
                self.directory.display()
            );
            return;
        }
        let _removed = fs::remove_dir_all(&self.directory);
    }
}

fn cache_path(executable: &Path) -> PathBuf {
    let mut path = executable.as_os_str().to_os_string();
    path.push(".gpu");
    path.into()
}

fn instrument(generated: &str) -> String {
    let shutdown = "static inline void tb_cuda_shutdown(void) {";
    assert_eq!(generated.matches(shutdown).count(), 1);
    assert_eq!(
        generated.matches("int main(int argc, char **argv)").count(),
        1
    );
    // Keep the real generated argv entry point and all runtime decisions.
    // Report before teardown because prebuild deliberately runs no Bend or
    // foreign initializer. Idempotent later shutdowns produce no extra report.
    format!(
        "{OBSERVATION}\n{}",
        generated.replace(shutdown, &format!("{shutdown}\n{SHUTDOWN_REPORT}"))
    )
}

#[derive(Debug)]
struct Report {
    session: [u64; 7],
    executions: Vec<[u64; 5]>,
    diagnostics: Vec<String>,
}

impl Report {
    fn read(output: &Output) -> Self {
        let mut sessions = Vec::new();
        let mut executions = Vec::new();
        let mut diagnostics = Vec::new();
        for line in String::from_utf8_lossy(&output.stderr).lines() {
            if let Some(values) = line.strip_prefix("CUDA ") {
                sessions.push(numbers(values));
            } else if let Some(values) = line.strip_prefix("GPU ") {
                executions.push(numbers(values));
            } else {
                diagnostics.push(line.to_owned());
            }
        }
        assert_eq!(sessions.len(), 1, "{output:?}");
        Self {
            session: sessions[0],
            executions,
            diagnostics,
        }
    }

    fn executed(&self, compilations: u64, hits: u64, misses: u64, writes: u64) {
        let [compiled, hit, miss, written, allocated, launches, completed] = self.session;
        assert_eq!(
            (compiled, hit, miss, written),
            (compilations, hits, misses, writes)
        );
        assert_eq!((allocated, completed), (5, 2));
        assert!(launches >= 12, "{self:?}");
        assert_eq!(self.executions.len(), 2, "{self:?}");
        for [dispatches, capacity, corpus, metadata, allocations] in &self.executions {
            assert!(*dispatches > 0, "{self:?}");
            assert_eq!(
                (*capacity, *corpus, *metadata),
                (1_572_864, 1_572_864, 1_572_864)
            );
            assert_eq!(*allocations, 5, "{self:?}");
        }
    }
}

fn numbers<const N: usize>(text: &str) -> [u64; N] {
    text.split_whitespace()
        .map(|value| value.parse().unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn executed(output: &Output) -> Report {
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(output.stdout, b"initialized\nmain\n41\n42\n");
    Report::read(output)
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn generated_cuda_cache_survives_warm_processes_and_unicode_executable_relocation() {
    let fixture = Fixture::new(&[]);
    let arguments = ["--gpu", "1.5MB"];
    let cold = executed(&fixture.run(&fixture.executable, &arguments));
    cold.executed(1, 0, 1, 1);
    assert_eq!(cold.diagnostics.len(), 1, "{cold:?}");
    assert!(
        cold.diagnostics[0].contains("is missing or stale"),
        "{cold:?}"
    );
    let cache = fs::read(cache_path(&fixture.executable)).unwrap();
    assert!(!cache.is_empty());
    assert_eq!(fs::read_dir(&fixture.working_directory).unwrap().count(), 0);

    let warm = executed(&fixture.run(&fixture.executable, &arguments));
    warm.executed(0, 1, 0, 0);
    assert!(warm.diagnostics.is_empty(), "{warm:?}");
    assert_eq!(fs::read(cache_path(&fixture.executable)).unwrap(), cache);

    let relocated = fixture.relocate();
    let moved = executed(&fixture.run(&relocated, &arguments));
    moved.executed(0, 1, 0, 0);
    assert!(moved.diagnostics.is_empty(), "{moved:?}");
    assert_eq!(fs::read(cache_path(&relocated)).unwrap(), cache);
    assert_eq!(fs::read_dir(&fixture.working_directory).unwrap().count(), 0);
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn generated_prebuild_skips_execution_and_upfront_allocation_failure_skips_foreign_effects() {
    let fixture = Fixture::new(&["BEND_MAX_GPU_ALLOC=2097152"]);
    let prebuilt = fixture.run(&fixture.executable, &["--gpu-build"]);
    assert_eq!(prebuilt.status.code(), Some(0), "{prebuilt:?}");
    assert!(prebuilt.stdout.is_empty(), "{prebuilt:?}");
    let prebuilt = Report::read(&prebuilt);
    assert_eq!(prebuilt.session, [1, 0, 0, 1, 0, 0, 0]);
    assert!(prebuilt.executions.is_empty(), "{prebuilt:?}");
    assert!(prebuilt.diagnostics.is_empty(), "{prebuilt:?}");
    let cache = fs::read(cache_path(&fixture.executable)).unwrap();
    assert!(!cache.is_empty());
    assert_eq!(fs::read_dir(&fixture.working_directory).unwrap().count(), 0);

    // The first 1.5 MiB allocation fits the explicit 2 MiB device cap. The
    // metadata reservation fails next, before foreign registration or main.
    let failed = fixture.run(&fixture.executable, &["--gpu", "1.5MB"]);
    assert_eq!(failed.status.code(), Some(1), "{failed:?}");
    assert!(failed.stdout.is_empty(), "{failed:?}");
    let failed = Report::read(&failed);
    assert_eq!(failed.session, [0, 1, 0, 0, 1, 0, 0]);
    assert!(failed.executions.is_empty(), "{failed:?}");
    assert_eq!(
        failed.diagnostics,
        ["bend: CUDA device allocation budget exhausted"]
    );
    assert_eq!(fs::read(cache_path(&fixture.executable)).unwrap(), cache);
}

const PROGRAM: &str = r#"import Base
def observe() -> IO(U32): import "probe.c"
def value(x: U32) -> U32: U32.add(x, 1)
def main() -> IO(Unit):
  do IO<Unit>:
    start : U32 <- observe()
    Unit <- IO.print(U32.show(value!(start)))
    IO.print(U32.show(value!(41)))
"#;

const FOREIGN: &str = r#"
static Term observe_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields; (void)work;
  (void)fputs("main\n", stdout);
  return (Term)40;
}
static void __attribute__((constructor)) observe_use(void) {
  (void)fputs("initialized\n", stdout);
  io_eff(CID_OBSERVE, observe_run, 0);
}
"#;

const OBSERVATION: &str = r#"
static unsigned int cache_test_completions;
#define TB_GPU_COMPLETE(control, state, info) do { \
  ++cache_test_completions; \
  (void)fprintf(stderr, "GPU %llu %llu %llu %llu %llu\n", \
    (unsigned long long)(control)->dispatches, \
    (unsigned long long)(state)->capacity * sizeof(Term), \
    (unsigned long long)tb_cuda.buffers[TB_GPU_CORPUS].capacity, \
    (unsigned long long)tb_cuda.buffers[TB_GPU_METADATA].capacity, \
    (unsigned long long)(info)->allocations); \
} while (0)
"#;

const SHUTDOWN_REPORT: &str = r#"
  if (tb_cuda.initialized) {
    (void)fprintf(stderr, "CUDA %llu %llu %llu %llu %llu %llu %u\n",
      (unsigned long long)tb_cuda.info.compilations,
      (unsigned long long)tb_cuda.info.cache_hits,
      (unsigned long long)tb_cuda.info.cache_misses,
      (unsigned long long)tb_cuda.info.cache_writes,
      (unsigned long long)tb_cuda.info.allocations,
      (unsigned long long)tb_cuda.info.launches, cache_test_completions);
  }
"#;
