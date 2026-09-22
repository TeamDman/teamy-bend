// SPDX-License-Identifier: MPL-2.0
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use std::time::Instant;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-native-gpu-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn run(&self, command: &mut Command, name: &str) -> Output {
        let mut child = command
            .env("BEND_GPU", "on")
            .env(
                "TEAMY_BEND_NATIVE_MAIN_MARKER",
                self.0.join("main-was-run.txt"),
            )
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let stdout = thread::spawn(move || read_output(stdout));
        let stderr = thread::spawn(move || read_output(stderr));
        let deadline = Instant::now() + Duration::from_mins(1);
        let (status, timed_out) = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break (status, false);
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                break (child.wait().unwrap(), true);
            }
            thread::sleep(Duration::from_millis(10));
        };
        let output = Output {
            status,
            stdout: stdout.join().unwrap(),
            stderr: stderr.join().unwrap(),
        };
        fs::write(self.0.join(format!("{name}.stdout")), &output.stdout).unwrap();
        fs::write(self.0.join(format!("{name}.stderr")), &output.stderr).unwrap();
        fs::write(
            self.0.join(format!("{name}.status")),
            format!("status={status}; timed_out={timed_out}\n"),
        )
        .unwrap();
        assert!(!timed_out, "{name} exceeded its 60-second deadline");
        assert!(
            output.status.success(),
            "{name}: {}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::env::var_os("TEAMY_BEND_KEEP_GPU_TESTS").is_some() {
            eprintln!("retained native GPU CLI test: {}", self.0.display());
            return;
        }
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn read_output(mut stream: impl Read) -> Vec<u8> {
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes).unwrap();
    bytes
}

fn sidecar(executable: &Path) -> PathBuf {
    let mut path = executable.as_os_str().to_owned();
    path.push(".gpu");
    PathBuf::from(path)
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn native_cli_installs_a_prebuilt_gpu_sidecar_that_survives_relocation() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("main.bend"), BEND).unwrap();
    fs::write(fixture.0.join("effect.c"), EFFECT).unwrap();
    let executable = fixture
        .0
        .join(format!("native output{}", std::env::consts::EXE_SUFFIX));
    fixture.run(
        Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .current_dir(&fixture.0)
            .args([
                "--output-format",
                "json",
                "--stop-after-duration",
                "45s",
                "compile",
                "main.bend",
                "--executable",
                "--target",
                "native",
                "--output",
            ])
            .arg(&executable),
        "build",
    );
    let marker = fixture.0.join("main-was-run.txt");
    assert!(
        !marker.exists(),
        "native compilation/prebuild must skip main"
    );
    assert!(executable.is_file());
    let cache = sidecar(&executable);
    let original_cache = fs::read(&cache).expect("native build installed executable.gpu");
    assert!(!original_cache.is_empty());
    let warm = fixture.run(
        Command::new(&executable).current_dir(&fixture.0),
        "installed",
    );
    assert_eq!(warm.stdout, b"7\n");
    // This checks public diagnostics and installed bytes. Adapter-level tests
    // independently verify the NVRTC compilation and cache-hit counters.
    assert!(warm.stderr.is_empty());
    assert_eq!(fs::read(&marker).unwrap(), b"ran");
    assert_eq!(fs::read(&cache).unwrap(), original_cache);

    let relocated_directory = fixture.0.join("déplacé λ");
    let working_directory = fixture.0.join("different working directory");
    fs::create_dir(&relocated_directory).unwrap();
    fs::create_dir(&working_directory).unwrap();
    let relocated =
        relocated_directory.join(format!("programme é{}", std::env::consts::EXE_SUFFIX));
    fs::rename(&executable, &relocated).unwrap();
    fs::rename(&cache, sidecar(&relocated)).unwrap();
    let warm = fixture.run(
        Command::new(&relocated).current_dir(&working_directory),
        "relocated",
    );
    assert_eq!(warm.stdout, b"7\n");
    assert!(warm.stderr.is_empty());
    assert_eq!(fs::read(&marker).unwrap(), b"ranran");
    assert_eq!(fs::read(sidecar(&relocated)).unwrap(), original_cache);
}

const BEND: &str = r#"import Base
def plus_one(x: U32) -> U32: U32.add(x, 1)
def stamp(x: U32) -> IO(U32): import "effect.c"
def main() -> IO(Unit):
  do IO<Unit>:
    value : U32 <- stamp(plus_one!(6))
    IO.print(U32.show(value))
"#;

const EFFECT: &str = r#"
static Term stamp_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)work;
  const char *path = getenv("TEAMY_BEND_NATIVE_MAIN_MARKER");
  if (path == NULL) err_fail("main marker path is missing");
  FILE *file = fopen(path, "ab");
  if (file == NULL) err_fail("cannot create main marker");
  if (fputs("ran", file) < 0 || fclose(file) != 0) err_fail("cannot write main marker");
  return fields[0];
}
static void __attribute__((constructor)) stamp_use(void) { io_eff(CID_STAMP, stamp_run, 0); }
"#;
