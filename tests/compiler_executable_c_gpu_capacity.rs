// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

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

// The marked identity completes first. The large allocation belongs to an
// ordinary CPU call after that GPU handoff, so this checks the shared logical
// corpus limit rather than only device-side allocation.
const PROGRAM: &str = r"import Base
def identity(x: U32) -> U32: x
def pick(pair: Array<U32> & U32) -> U32:
  (array, value) = pair
  value
def cpu_make() -> U32: pick(Array.get(U32, Array.new(U32, 12n, 7), 4095))
def main() -> U32:
  ready = identity!(0)
  U32.add(ready, cpu_make())
";

struct Fixture {
    directory: PathBuf,
    executable: PathBuf,
}

impl Fixture {
    fn new(default_capacity: u64) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-gpu-capacity-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let path = directory.join("main.bend");
        fs::write(&path, PROGRAM).unwrap();
        let generated =
            compile_executable_c(&check_executable(&load_executable(path).unwrap()).unwrap())
                .unwrap();
        assert!(generated.contains("#define TB_GPU_ENABLED 1"));
        let default = format!("BEND_MAX_ALLOC={default_capacity}");
        let executable = executable_c_compiler::compile(
            &directory,
            &generated,
            &[
                &default,
                "BEND_MAX_GPU_SCRATCH=65536",
                "BEND_MAX_GPU_ALLOC=4194304",
                "BEND_CPU_WORKERS=1",
            ],
        );
        Self {
            directory,
            executable,
        }
    }

    fn run(&self, label: &str, span: &str, success: bool) {
        let mut cache = self.executable.clone().into_os_string();
        cache.push(".gpu");
        let cache = PathBuf::from(cache);
        let mut expected_error = String::new();
        if span != "off" {
            assert!(
                !cache.exists(),
                "this case must start with a cold GPU cache"
            );
            expected_error = format!(
                "bend: compiling the GPU program ({} is missing or stale)\n",
                cache.display()
            );
        }
        if !success {
            expected_error.push_str("teamy-bend executable C: VM allocation budget exhausted\n");
        }
        let output = executable_c_compiler::bounded(
            Command::new(&self.executable)
                .current_dir(&self.directory)
                .args(["--threads", "1", "--gpu", span])
                // Explicit command-line policy must take precedence.
                .env("BEND_GPU", "off"),
            Duration::from_mins(1),
        );
        fs::write(
            self.directory.join(format!("{label}-stdout.log")),
            &output.stdout,
        )
        .unwrap();
        fs::write(
            self.directory.join(format!("{label}-stderr.log")),
            &output.stderr,
        )
        .unwrap();
        assert_eq!(
            output.status.code(),
            Some(i32::from(!success)),
            "{}: {}",
            self.directory.display(),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, if success { b"7\n".as_slice() } else { b"" });
        assert_eq!(output.stderr, expected_error.as_bytes());
        assert_eq!(cache.exists(), span != "off");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::env::var_os("TEAMY_BEND_KEEP_GPU_TESTS").is_some() {
            eprintln!("retained GPU capacity test: {}", self.directory.display());
            return;
        }
        let _removed = fs::remove_dir_all(&self.directory);
    }
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn actual_gpu_arguments_raise_and_lower_the_cpu_array_allocation_limit() {
    let small_default = Fixture::new(8192);
    small_default.run("default-small", "off", false);
    small_default.run("explicit-larger", "0.0625MB", true);

    let large_default = Fixture::new(1_048_576);
    large_default.run("default-large", "off", true);
    // The packed payload alone needs 16 KiB, so a 16 KiB corpus cannot also
    // contain the reserved first word and ordinary live evaluation records.
    large_default.run("explicit-smaller", "0.015625MB", false);
}
