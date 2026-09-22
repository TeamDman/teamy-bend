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

struct Fixture {
    directory: PathBuf,
    executable: PathBuf,
}

impl Fixture {
    fn new(marked: bool) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-c-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        fs::write(directory.join("probe.c"), PROBE).unwrap();
        let path = directory.join("main.bend");
        fs::write(
            &path,
            if marked {
                PROGRAM.to_owned()
            } else {
                PROGRAM.replace("value!(41)", "value(41)")
            },
        )
        .unwrap();
        let checked = check_executable(&load_executable(path).unwrap()).unwrap();
        let generated = compile_executable_c(&checked).unwrap();
        assert_eq!(generated.contains("#define TB_GPU_ENABLED 1"), marked);
        assert_eq!(
            generated.matches("int main(int argc, char **argv)").count(),
            1
        );
        // Exercise the real generated argv entry point. The marked fixture
        // retains the entire CUDA adapter but fails before loading libraries,
        // covering both ordinary initialization and cache prebuild without
        // requiring CUDA hardware.
        let generated = if marked {
            unavailable_cuda(&generated)
        } else {
            generated
        };
        let executable = executable_c_compiler::compile(
            &directory,
            &generated,
            &["BEND_CPU_WORKERS=4", "BEND_MAX_ALLOC=1048576"],
        );
        Self {
            directory,
            executable,
        }
    }

    fn run(&self, arguments: &[&str], policy: &str) -> Output {
        executable_c_compiler::bounded(
            Command::new(&self.executable)
                .current_dir(&self.directory)
                .args(arguments)
                .env("BEND_GPU", policy),
            Duration::from_secs(30),
        )
    }

    fn cache_path(&self) -> PathBuf {
        let mut path = self.executable.clone().into_os_string();
        path.push(".gpu");
        path.into()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.directory);
    }
}

fn unavailable_cuda(generated: &str) -> String {
    let signature = "static bool tb_cuda_libraries(void) {";
    assert_eq!(generated.matches(signature).count(), 1);
    format!(
        "static volatile int cli_cuda_unavailable = 1;\n{}",
        generated.replace(
            signature,
            &format!(
                "{signature}\n  (void)printf(\"cuda-attempt span=%llu\\n\", (unsigned long long)tb_cli_gpu_span);\n  if (cli_cuda_unavailable) return tb_cuda_message(TB_CUDA_ERROR_UNAVAILABLE, \"test CUDA unavailable\");"
            )
        )
    )
}

fn success(output: &Output, expected: &str) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected.as_bytes());
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
}

fn help(output: &Output) {
    assert!(output.status.success(), "{:?}", output.stderr);
    assert!(output.stderr.is_empty(), "{:?}", output.stderr);
    let text = String::from_utf8_lossy(&output.stdout);
    for option in ["--help", "--threads", "--gpu", "--gpu-build"] {
        assert!(text.contains(option), "missing {option}: {text}");
    }
    assert!(!text.contains("initialized"), "{text}");
    assert!(!text.contains("cuda-attempt"), "{text}");
}

const PROGRAM: &str = r#"import Base
def workers() -> IO(U32): import "probe.c"
def value(x: U32) -> U32: U32.add(x, 1)
def main() -> IO(Unit):
  do IO<Unit>:
    count : U32 <- workers()
    Unit <- IO.print(U32.show(count))
    IO.print(U32.show(value!(41)))
"#;

const PROBE: &str = r#"
static Term workers_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields; (void)work;
  return (Term)tb_cpu.desired;
}
static void __attribute__((constructor)) workers_use(void) {
  (void)fputs("initialized\n", stdout);
  io_eff(CID_WORKERS, workers_run, 0);
}
"#;

#[test]
fn help_and_prebuild_skip_bend_and_foreign_initializers_without_device_source() {
    let fixture = Fixture::new(false);
    success(&fixture.run(&[], "off"), "initialized\n4\n42\n");
    help(&fixture.run(&["--help"], "on"));
    success(&fixture.run(&["--gpu-build"], "on"), "");
    assert!(!fixture.cache_path().exists());
}

#[test]
fn help_and_unavailable_device_prebuild_skip_bend_and_foreign_initializers() {
    let fixture = Fixture::new(true);
    help(&fixture.run(&["--help"], "on"));
    success(
        &fixture.run(&["--gpu-build"], "auto"),
        "cuda-attempt span=0\n",
    );
    assert!(!fixture.cache_path().exists());
}

#[test]
fn thread_arguments_select_the_initialized_pool_and_clamp_at_128() {
    let fixture = Fixture::new(false);
    for (argument, expected) in [
        ("1", 1),
        ("4", 4),
        ("128", 128),
        ("129", 128),
        ("1024", 128),
        ("18446744073709551615", 128),
        ("18446744073709551616", 128),
    ] {
        success(
            &fixture.run(&["--threads", argument], "off"),
            &format!("initialized\n{expected}\n42\n"),
        );
    }
    success(
        &fixture.run(&["--gpu", "on", "--threads", "2"], "off"),
        "initialized\n2\n42\n",
    );
    success(
        &fixture.run(&["--gpu", "1.5MB"], "on"),
        "initialized\n4\n42\n",
    );
}

#[test]
fn invalid_arguments_fail_before_registration_main_or_cuda_initialization() {
    let fixture = Fixture::new(true);
    let invalid: &[&[&str]] = &[
        &["--unknown"],
        &["unexpected"],
        &["--threads"],
        &["--threads", "0"],
        &["--threads", "-1"],
        &["--threads", "1.5"],
        &["--gpu"],
        &["--gpu", "auto"],
        &["--gpu", "0"],
        &["--gpu", "0MB"],
        &["--gpu", "-1GB"],
        &["--gpu", "0.001MB"],
        &["--gpu", "NaNMB"],
        &["--gpu", "infGB"],
        &["--gpu", "1e309GB"],
        &["--gpu", "18446744073709551616GB"],
        &["--gpu", "8388608MB"],
        &["--gpu", "1GBjunk"],
    ];
    for arguments in invalid {
        let output = fixture.run(arguments, "on");
        assert_eq!(output.status.code(), Some(1), "{arguments:?}");
        assert!(
            output.stdout.is_empty(),
            "{arguments:?}: {:?}",
            output.stdout
        );
        assert!(!output.stderr.is_empty(), "{arguments:?}");
    }
}

#[test]
fn gpu_cli_policy_overrides_the_environment_in_both_directions() {
    let fixture = Fixture::new(true);
    for policy in ["on", "invalid-environment-policy"] {
        success(
            &fixture.run(&["--gpu", "off"], policy),
            "initialized\n4\n42\n",
        );
    }
    let output = fixture.run(&["--gpu", "on"], "off");
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"cuda-attempt span=0\n");
    assert_eq!(
        output.stderr,
        b"bend: --gpu on, but this binary found no GPU device\n"
    );
}

#[test]
fn gpu_spans_use_binary_units_and_round_fractional_sizes_to_pages() {
    let fixture = Fixture::new(true);
    for (argument, bytes) in [
        ("1MB", 1_048_576_u64),
        ("1.5MB", 1_572_864),
        ("0.5GB", 536_870_912),
        ("0.02MB", 16_384),
        ("0.015625MB", 16_384),
    ] {
        let output = fixture.run(&["--gpu", argument], "off");
        assert_eq!(output.status.code(), Some(1), "{argument}");
        assert_eq!(
            output.stdout,
            format!("cuda-attempt span={bytes}\n").as_bytes(),
            "{argument}"
        );
        assert_eq!(
            output.stderr, b"bend: --gpu on, but this binary found no GPU device\n",
            "{argument}"
        );
    }
}
