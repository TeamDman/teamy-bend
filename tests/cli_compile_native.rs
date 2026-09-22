// SPDX-License-Identifier: MPL-2.0
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

#[cfg(windows)]
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

static NEXT: AtomicUsize = AtomicUsize::new(0);
const PURE: &str = "import Base\ndef answer() -> Nat: 3n\ndef main() -> Nat: 7n\n";

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "teamy-bend-native-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("main.bend"), source).unwrap();
        Self(root)
    }

    fn binary(&self) -> PathBuf {
        self.0
            .join(format!("native output{}", std::env::consts::EXE_SUFFIX))
    }

    fn command(&self, output: &Path) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
        command
            .current_dir(&self.0)
            .args([
                "--output-format",
                "json",
                "compile",
                "main.bend",
                "--target",
                "native",
                "--output",
            ])
            .arg(output)
            .env("BEND_GPU", "off");
        command
    }

    fn success(&self, arguments: &[&str]) -> Output {
        let output = self
            .command(&self.binary())
            .args(arguments)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("native"));
        self.assert_no_staging();
        output
    }

    fn assert_no_staging(&self) {
        assert!(!fs::read_dir(&self.0).unwrap().any(|entry| {
            entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(".teamy-bend-build-")
        }));
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn assert_nat_output(output: &Output, expected: u32) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut value = String::from(r#"{"constructor":"Zero","fields":[]}"#);
    for _ in 0..expected {
        value = format!(r#"{{"constructor":"Succ","fields":[{value}]}}"#);
    }
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        format!(r#"{{"value":{value}}}"#)
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn native_strict_output_is_a_runnable_binary_and_retains_entry_selection() {
    let fixture = Fixture::new(PURE);
    fixture.success(&["--entry", "answer"]);
    let output = Command::new(fixture.binary()).output().unwrap();
    assert_nat_output(&output, 3);
    let mut cache = fixture.binary().into_os_string();
    cache.push(".gpu");
    assert!(!PathBuf::from(cache).exists());
}

#[test]
fn executable_native_prebuild_never_runs_a_main_effect() {
    let fixture = Fixture::new(
        r#"import Base
def plus_one(x: U32) -> U32: U32.add(x, 1)
def stamp(x: U32) -> IO(U32): import "effect.c"
def main() -> IO(Unit):
  do IO<Unit>:
    value : U32 <- stamp(plus_one!(6))
    IO.print(U32.show(value))
"#,
    );
    let marker = fixture.0.join("main-was-run.txt");
    let marker_literal = facet_json::to_string(&marker.to_string_lossy().as_ref()).unwrap();
    fs::write(
        fixture.0.join("effect.c"),
        format!(
            r#"
static Term stamp_run(Env e, Term *fields, IoWork *work) {{
  (void)e; (void)work;
  FILE *file = fopen({marker_literal}, "wb");
  if (file == NULL) err_fail("cannot create main marker");
  if (fputs("ran", file) < 0 || fclose(file) != 0) err_fail("cannot write main marker");
  return fields[0];
}}
static void __attribute__((constructor)) stamp_use(void) {{ io_eff(CID_STAMP, stamp_run, 0); }}
"#
        ),
    )
    .unwrap();
    fixture.success(&["--executable"]);
    assert!(
        !marker.exists(),
        "native compilation/prebuild must not execute main"
    );
    let output = Command::new(fixture.binary())
        .current_dir(&fixture.0)
        .env("BEND_GPU", "off")
        .output()
        .unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"7\n");
    assert_eq!(fs::read_to_string(marker).unwrap(), "ran");
}

#[test]
fn native_outputs_require_force_and_bad_compilation_preserves_the_previous_file() {
    let fixture = Fixture::new(PURE);
    fs::write(fixture.binary(), "previous output").unwrap();
    let refused = fixture.command(&fixture.binary()).output().unwrap();
    assert_eq!(refused.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("--force"));
    let missing = fixture
        .command(&fixture.binary())
        .arg("--force")
        .env("TEAMY_BEND_CC", fixture.0.join("missing-compiler"))
        .output()
        .unwrap();
    assert_eq!(missing.status.code(), Some(1));
    assert_eq!(
        fs::read_to_string(fixture.binary()).unwrap(),
        "previous output"
    );
    fixture.assert_no_staging();
    fixture.success(&["--force"]);
    assert_nat_output(&Command::new(fixture.binary()).output().unwrap(), 7);
}

#[test]
fn native_c_compiler_errors_do_not_replace_existing_output() {
    let fixture = Fixture::new(
        "import Base\ndef effect() -> IO(U32): import \"effect.c\"\ndef main() -> IO(U32): effect()\n",
    );
    fs::write(
        fixture.0.join("effect.c"),
        "#error deliberate native compile failure\n",
    )
    .unwrap();
    fs::write(fixture.binary(), "previous output").unwrap();
    let output = fixture
        .command(&fixture.binary())
        .args(["--executable", "--force"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("deliberate native compile failure"));
    assert_eq!(
        fs::read_to_string(fixture.binary()).unwrap(),
        "previous output"
    );
    fixture.assert_no_staging();
}

#[test]
fn force_cannot_replace_bend_inputs_imports_or_their_hard_links() {
    let fixture =
        Fixture::new("import Base\nimport value.bend as V\ndef main() -> Nat: V.answer()\n");
    let module = fixture.0.join("value.bend");
    fs::write(&module, "import Base\ndef answer() -> Nat: 7n\n").unwrap();
    let alias = fixture.0.join("linked-output");
    fs::hard_link(&module, &alias).unwrap();
    for path in [fixture.0.join("main.bend"), module.clone(), alias] {
        let original = fs::read(&path).unwrap();
        let output = fixture.command(&path).arg("--force").output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("source input"));
        assert_eq!(fs::read(&path).unwrap(), original);
    }
    fixture.assert_no_staging();
}

#[test]
fn native_output_and_gpu_sidecar_cannot_replace_foreign_source() {
    let fixture = Fixture::new(
        "import Base\ndef value(x: U32) -> U32: x\ndef effect(x: U32) -> IO(U32): import \"effect.c\"\ndef main() -> IO(U32): effect(value!(7))\n",
    );
    let foreign = fixture.0.join("effect.c");
    let source = "/* no top-level side effect */\n";
    fs::write(&foreign, source).unwrap();
    let output = fixture
        .command(&foreign)
        .args(["--executable", "--force"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("source input"));
    let mut sidecar = fixture.binary().into_os_string();
    sidecar.push(".gpu");
    fs::hard_link(&foreign, PathBuf::from(sidecar)).unwrap();
    let output = fixture
        .command(&fixture.binary())
        .args(["--executable", "--force"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("source input"));
    assert_eq!(fs::read_to_string(foreign).unwrap(), source);
    assert!(!fixture.binary().exists());
    fixture.assert_no_staging();
}

#[test]
fn c_source_output_does_not_require_a_native_toolchain() {
    let fixture = Fixture::new(PURE);
    let output = Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
        .current_dir(&fixture.0)
        .args([
            "compile",
            "main.bend",
            "--target",
            "c",
            "--output",
            "output.c",
        ])
        .env("TEAMY_BEND_CC", fixture.0.join("missing-compiler"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        fs::read_to_string(fixture.0.join("output.c"))
            .unwrap()
            .contains("int main(")
    );
}

#[cfg(windows)]
const CANCELLATION_HELPER: &str = r#"
#define _CRT_SECURE_NO_WARNINGS
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
int main(int argc, char **argv) {
  if (argc == 2 && strcmp(argv[1], "--worker") == 0) {
    const char *path = getenv("TEAMY_BEND_NATIVE_CHILD_PID");
    if (path == NULL) return 2;
    FILE *file = fopen(path, "wb");
    if (file == NULL) return 3;
    if (fprintf(file, "%lu", (unsigned long)GetCurrentProcessId()) < 0) return 4;
    if (fclose(file) != 0) return 5;
    Sleep(15000);
    return 0;
  }
  // Ordinary descendants start after the compiler has been assigned its job.
  Sleep(200);
  char module[MAX_PATH], command[MAX_PATH + 32];
  DWORD length = GetModuleFileNameA(NULL, module, MAX_PATH);
  if (length == 0 || length >= MAX_PATH) return 6;
  int count = snprintf(command, sizeof(command), "\"%s\" --worker", module);
  if (count < 0 || (size_t)count >= sizeof(command)) return 7;
  STARTUPINFOA startup = {0};
  PROCESS_INFORMATION process = {0};
  startup.cb = (DWORD)sizeof(startup);
  startup.dwFlags = STARTF_USESTDHANDLES;
  startup.hStdInput = GetStdHandle(STD_INPUT_HANDLE);
  startup.hStdOutput = GetStdHandle(STD_OUTPUT_HANDLE);
  startup.hStdError = GetStdHandle(STD_ERROR_HANDLE);
  if (!CreateProcessA(module, command, NULL, NULL, TRUE, CREATE_NO_WINDOW,
                      NULL, NULL, &startup, &process)) return 8;
  if (!CloseHandle(process.hThread)) return 9;
  if (!CloseHandle(process.hProcess)) return 10;
  Sleep(15000);
  return 0;
}
"#;

#[cfg(windows)]
#[test]
fn cancellation_terminates_the_compiler_child_and_preserves_previous_output() {
    use std::time::Duration;
    use std::time::Instant;
    use windows::Win32::Foundation::WAIT_OBJECT_0;
    use windows::Win32::Foundation::WAIT_TIMEOUT;
    use windows::Win32::System::Threading::OpenProcess;
    use windows::Win32::System::Threading::PROCESS_SYNCHRONIZE;
    use windows::Win32::System::Threading::WaitForSingleObject;
    use windows::core::Owned;

    let fixture = Fixture::new(PURE);
    let helper_directory = fixture.0.join("helper");
    fs::create_dir(&helper_directory).unwrap();
    let helper = executable_c_compiler::compile(&helper_directory, CANCELLATION_HELPER, &[]);
    fs::write(fixture.binary(), "previous output").unwrap();
    let pid_file = fixture.0.join("child-pid.txt");
    let mut command = Command::new(env!("CARGO_BIN_EXE_teamy-bend"));
    command
        .current_dir(&fixture.0)
        .args([
            "--stop-after-duration",
            "3s",
            "compile",
            "main.bend",
            "--target",
            "native",
            "--force",
            "--output",
        ])
        .arg(fixture.binary())
        .env("TEAMY_BEND_CC", helper)
        .env("TEAMY_BEND_NATIVE_CHILD_PID", &pid_file)
        .env("BEND_GPU", "off");
    let execution = std::thread::spawn(move || {
        executable_c_compiler::bounded(&mut command, Duration::from_secs(20))
    });
    let deadline = Instant::now() + Duration::from_secs(5);
    let pid = loop {
        if let Ok(text) = fs::read_to_string(&pid_file)
            && let Ok(pid) = text.parse::<u32>()
        {
            break pid;
        }
        if Instant::now() >= deadline {
            let output = execution.join().unwrap();
            panic!(
                "compiler child did not start: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    // SAFETY: The helper reported its own PID. Open a synchronization-only
    // handle while it is alive, retaining the exact process despite PID reuse.
    let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) }.unwrap();
    // SAFETY: Successful OpenProcess transferred this unique owned handle.
    let process = unsafe { Owned::new(handle) };
    // SAFETY: The live owned process handle supports this bounded wait.
    assert_eq!(unsafe { WaitForSingleObject(*process, 0) }, WAIT_TIMEOUT);
    let output = execution.join().unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cancel"));
    // SAFETY: The live owned process handle identifies the same helper child.
    let child_status = unsafe { WaitForSingleObject(*process, 2000) };
    assert_eq!(child_status, WAIT_OBJECT_0);
    assert_eq!(
        fs::read_to_string(fixture.binary()).unwrap(),
        "previous output"
    );
    fixture.assert_no_staging();
}

#[cfg(unix)]
#[test]
fn native_output_symlinks_never_replace_their_target() {
    let fixture = Fixture::new(PURE);
    let source = fixture.0.join("main.bend");
    std::os::unix::fs::symlink(&source, fixture.binary()).unwrap();
    let output = fixture
        .command(&fixture.binary())
        .arg("--force")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("symlink"));
    assert_eq!(fs::read_to_string(source).unwrap(), PURE);
}
