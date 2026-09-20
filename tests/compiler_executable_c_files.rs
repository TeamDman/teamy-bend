// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

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
const HELPERS: &str = r#"import Base
def text_result(r: Result<&1, &1, (U32 & String), String>) -> IO(Unit):
  match r:
    case Done{value}: IO.print(String.append("text:", value))
    case Fail{error}:
      (code, message) = error
      IO.print(String.append("error:", U32.show(code)))

def keep_write(pair: File & Result<&1, &1, (U32 & String), Unit>) -> IO(File):
  (file, result) = pair
  match result:
    case Done{value}: IO.pure(File, file)
    case Fail{error}:
      (code, message) = error
      do IO<File>:
        Unit <- IO.print(String.append("error:", U32.show(code)))
        return file

def keep_text(pair: File & Result<&1, &1, (U32 & String), String>) -> IO(File):
  (file, result) = pair
  do IO<File>:
    Unit <- text_result(result)
    return file

def byte_line(xs: List<&2, U32>) -> String:
  match xs:
    case Nil{}: ""
    case x <> rest: U32.show(x) ++ "," ++ byte_line(rest)

def keep_bytes(pair: File & Result<&1, &1, (U32 & String), List<&2, U32>>) -> IO(File):
  (file, result) = pair
  match result:
    case Done{bytes}:
      do IO<File>:
        Unit <- IO.print(String.append("bytes:", byte_line(bytes)))
        return file
    case Fail{error}:
      (code, message) = error
      do IO<File>:
        Unit <- IO.print(String.append("error:", U32.show(code)))
        return file
"#;

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-c-files-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).unwrap();
        std::fs::write(directory.join("main.bend"), format!("{HELPERS}{source}")).unwrap();
        Self(directory)
    }

    fn run(&self) -> Output {
        self.run_with_definitions(&[])
    }

    fn run_with_definitions(&self, definitions: &[&str]) -> Output {
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        let source = compile_executable_c(&checked).unwrap();
        let executable = executable_c_compiler::compile(&self.0, &source, definitions);
        executable_c_compiler::bounded(
            Command::new(executable)
                .current_dir(&self.0)
                .env("TEAMY_BEND_FILE_TEST_VALUE", "Olá 日本 🙂")
                .env("TEAMY_BEND_FILE_TEST_EMPTY", "")
                .env_remove("TEAMY_BEND_FILE_TEST_MISSING")
                .env_remove("toString")
                .env_remove("hasOwnProperty"),
            Duration::from_secs(20),
        )
    }

    fn success(&self, stdout: &str) {
        let result = self.run();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stdout, stdout.as_bytes());
        assert!(result.stderr.is_empty());
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(&self.0);
    }
}

const ENVIRONMENT: &str = r#"def main() -> IO(Unit):
  do IO<Unit>:
    a : Result<&1, &1, (U32 & String), String> <- IO.get_env("TEAMY_BEND_FILE_TEST_VALUE")
    Unit <- text_result(a)
    a : Result<&1, &1, (U32 & String), String> <- IO.get_env("TEAMY_BEND_FILE_TEST_EMPTY")
    Unit <- text_result(a)
    a : Result<&1, &1, (U32 & String), String> <- IO.get_env("TEAMY_BEND_FILE_TEST_MISSING")
    Unit <- text_result(a)
    a : Result<&1, &1, (U32 & String), String> <- IO.get_env("toString")
    Unit <- text_result(a)
    a : Result<&1, &1, (U32 & String), String> <- IO.get_env("hasOwnProperty")
    Unit <- text_result(a)
    a : Result<&1, &1, (U32 & String), String> <- IO.get_env("TEAMY_BEND_FILE_TEST_VALUE\0tail")
    text_result(a)
"#;

#[test]
fn environment_distinguishes_empty_missing_unicode_and_embedded_nul() {
    Fixture::new(ENVIRONMENT)
        .success("text:Olá 日本 🙂\ntext:\nerror:2\nerror:2\nerror:2\nerror:2\n");
}

const ENVIRONMENT_HALT: &str = r#"def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print_err("diagnostic")
    Unit <- IO.print("before")
    value : String <- IO.try(String, IO.get_env("TEAMY_BEND_FILE_TEST_MISSING"))
    IO.print("BAD")
"#;

#[test]
fn failed_environment_try_halts_without_running_the_continuation() {
    let result = Fixture::new(ENVIRONMENT_HALT).run();
    assert_eq!(result.status.code(), Some(2));
    assert_eq!(result.stdout, b"before\n");
    assert_eq!(result.stderr, b"diagnostic\nNo such file or directory\n");
}

const ROUNDTRIP: &str = r#"def main() -> IO(Unit):
  do IO<Unit>:
    f : File <- IO.try(File, File.open("roundtrip-é.tmp", "w"))
    pair : File & Result<&1, &1, (U32 & String), Unit> <- File.write(f, "Olá 日本 🙂\0\r\n")
    f : File <- keep_write(pair)
    Unit <- File.close(f)
    f : File <- IO.try(File, File.open("roundtrip-é.tmp", "r"))
    pair : File & Result<&1, &1, (U32 & String), String> <- File.read(f, 1024)
    f : File <- keep_text(pair)
    pair : File & Result<&1, &1, (U32 & String), String> <- File.read(f, 1024)
    f : File <- keep_text(pair)
    File.close(f)
"#;

#[test]
fn unicode_paths_and_text_preserve_nul_and_crlf_bytes_and_eof() {
    let fixture = Fixture::new(ROUNDTRIP);
    fixture.success("text:Olá 日本 🙂\0\r\n\ntext:\n");
    assert_eq!(
        std::fs::read(fixture.0.join("roundtrip-é.tmp")).unwrap(),
        "Olá 日本 🙂\0\r\n".as_bytes()
    );
}

const APPEND: &str = r#"def main() -> IO(Unit):
  do IO<Unit>:
    f : File <- IO.try(File, File.open("append.tmp", "w"))
    pair : File & Result<&1, &1, (U32 & String), Unit> <- File.write(f, "old")
    f : File <- keep_write(pair)
    Unit <- File.close(f)
    f : File <- IO.try(File, File.open("append.tmp", "w"))
    pair : File & Result<&1, &1, (U32 & String), Unit> <- File.write(f, "new")
    f : File <- keep_write(pair)
    Unit <- File.close(f)
    f : File <- IO.try(File, File.open("append.tmp", "a"))
    pair : File & Result<&1, &1, (U32 & String), Unit> <- File.write(f, "!")
    f : File <- keep_write(pair)
    Unit <- File.close(f)
    f : File <- IO.try(File, File.open("created-by-append.tmp", "a"))
    File.close(f)
"#;

#[test]
fn write_truncates_append_preserves_and_both_create() {
    let fixture = Fixture::new(APPEND);
    fixture.success("");
    assert_eq!(
        std::fs::read(fixture.0.join("append.tmp")).unwrap(),
        b"new!"
    );
    assert!(fixture.0.join("created-by-append.tmp").is_file());
}

const BYTE_READS: &str = r#"def main() -> IO(Unit):
  do IO<Unit>:
    f : File <- IO.try(File, File.open("bytes.tmp", "r"))
    pair : File & Result<&1, &1, (U32 & String), List<&2, U32>> <- File.read_bytes(f, 0)
    f : File <- keep_bytes(pair)
    pair : File & Result<&1, &1, (U32 & String), List<&2, U32>> <- File.read_bytes(f, 3)
    f : File <- keep_bytes(pair)
    pair : File & Result<&1, &1, (U32 & String), List<&2, U32>> <- File.read_bytes(f, 8)
    f : File <- keep_bytes(pair)
    pair : File & Result<&1, &1, (U32 & String), List<&2, U32>> <- File.read_bytes(f, 8)
    f : File <- keep_bytes(pair)
    File.close(f)
"#;

#[test]
fn byte_reads_preserve_octets_and_file_position_across_zero_short_and_eof_reads() {
    let fixture = Fixture::new(BYTE_READS);
    std::fs::write(fixture.0.join("bytes.tmp"), [0, 127, 128, 195, 161, 255]).unwrap();
    fixture.success("bytes:\nbytes:0,127,128,\nbytes:195,161,255,\nbytes:\n");
}

const FAILURE_KEEPS_HANDLE: &str = r#"def main() -> IO(Unit):
  do IO<Unit>:
    f : File <- IO.try(File, File.open("kept.tmp", "w"))
    pair : File & Result<&1, &1, (U32 & String), String> <- File.read(f, 8)
    f : File <- keep_text(pair)
    pair : File & Result<&1, &1, (U32 & String), List<&2, U32>> <- File.read_bytes(f, 8)
    f : File <- keep_bytes(pair)
    pair : File & Result<&1, &1, (U32 & String), Unit> <- File.write(f, "kept")
    f : File <- keep_write(pair)
    Unit <- File.close(f)
    f : File <- IO.try(File, File.open("kept.tmp", "r"))
    pair : File & Result<&1, &1, (U32 & String), Unit> <- File.write(f, "BAD")
    f : File <- keep_write(pair)
    pair : File & Result<&1, &1, (U32 & String), Unit> <- File.write(f, "")
    f : File <- keep_write(pair)
    pair : File & Result<&1, &1, (U32 & String), String> <- File.read(f, 8)
    f : File <- keep_text(pair)
    File.close(f)
"#;

#[test]
fn failed_reads_and_writes_return_the_same_live_handle() {
    let fixture = Fixture::new(FAILURE_KEEPS_HANDLE);
    fixture.success("error:9\nerror:9\nerror:9\ntext:kept\n");
    assert_eq!(std::fs::read(fixture.0.join("kept.tmp")).unwrap(), b"kept");
}

const MODES_AND_NUL: &str = r#"def report(result: Result<&1, &1, (U32 & String), File>) -> IO(Unit):
  match result:
    case Done{file}:
      do IO<Unit>:
        Unit <- File.close(file)
        IO.print("BAD opened")
    case Fail{error}:
      (code, message) = error
      IO.print(U32.show(code))

def probe(path: String, mode: String) -> IO(Unit):
  IO.bind(Result<&1, &1, (U32 & String), File>, Unit, File.open(path, mode), report)

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- probe("missing/file.tmp", "r")
    Unit <- probe("bad-mode.tmp", "r+")
    Unit <- probe("bad-mode.tmp", "w+")
    Unit <- probe("bad-mode.tmp", "a+")
    Unit <- probe("bad-mode.tmp", "")
    Unit <- probe("bad-mode.tmp", "W")
    Unit <- probe("bad-mode.tmp", "w\0tail")
    Unit <- probe("nul-path.tmp\0tail", "w")
    probe("nul-path.tmp\0tail", "bad")
"#;

#[test]
fn invalid_modes_never_open_and_nul_path_errors_precede_mode_errors() {
    let fixture = Fixture::new(MODES_AND_NUL);
    let ilseq = if cfg!(windows) {
        42
    } else if cfg!(target_os = "macos") {
        92
    } else {
        84
    };
    fixture.success(&format!("2\n22\n22\n22\n22\n22\n22\n{ilseq}\n{ilseq}\n"));
    assert!(!fixture.0.join("bad-mode.tmp").exists());
    assert!(!fixture.0.join("nul-path.tmp").exists());
}

const MALFORMED_TEXT: &str = r#"def code_line(text: String) -> String:
  match text:
    case SNil{}: ""
    case SCon{Chr{code}, rest}: U32.show(code) ++ "," ++ code_line(rest)

def show_codes(pair: File & Result<&1, &1, (U32 & String), String>) -> IO(File):
  (file, result) = pair
  do IO<File>:
    text : String <- IO.pass(String, result)
    Unit <- IO.print(code_line(text))
    return file

def main() -> IO(Unit):
  do IO<Unit>:
    f : File <- IO.try(File, File.open("malformed.tmp", "r"))
    pair : File & Result<&1, &1, (U32 & String), String> <- File.read(f, 1024)
    f : File <- show_codes(pair)
    Unit <- File.close(f)
    f : File <- IO.try(File, File.open("split.tmp", "r"))
    pair : File & Result<&1, &1, (U32 & String), String> <- File.read(f, 1)
    f : File <- show_codes(pair)
    pair : File & Result<&1, &1, (U32 & String), String> <- File.read(f, 1)
    f : File <- show_codes(pair)
    File.close(f)
"#;

#[test]
fn text_reads_follow_native_io_str_even_for_non_scalar_and_split_sequences() {
    let fixture = Fixture::new(MALFORMED_TEXT);
    std::fs::write(
        fixture.0.join("malformed.tmp"),
        [
            0x80, 0xc0, 0x80, 0xed, 0xa0, 0x80, 0xf4, 0x90, 0x80, 0x80, 0xff, 0xc3,
        ],
    )
    .unwrap();
    std::fs::write(fixture.0.join("split.tmp"), [0xc3, 0xa9]).unwrap();
    // Upstream C io_str accepts overlong, surrogate and out-of-range code points.
    // Observe raw Char codes without conflating decoding with console encoding.
    fixture.success("128,0,55296,1114112,255,195,\n195,\n169,\n");
}

const RAW_CHARACTER_WRITE: &str = r#"def main() -> IO(Unit):
  do IO<Unit>:
    f : File <- IO.try(File, File.open("raw-characters.tmp", "w"))
    pair : File & Result<&1, &1, (U32 & String), Unit> <- File.write(f, SCon{Chr{55296}, SCon{Chr{1114112}, SCon{Chr{4294967295}, SNil{}}}})
    f : File <- keep_write(pair)
    File.close(f)
"#;

#[test]
fn file_writes_encode_raw_character_codes_like_native_io_utf8() {
    let fixture = Fixture::new(RAW_CHARACTER_WRITE);
    fixture.success("");
    assert_eq!(
        std::fs::read(fixture.0.join("raw-characters.tmp")).unwrap(),
        [
            0xed, 0xa0, 0x80, 0xf4, 0x90, 0x80, 0x80, 0xff, 0xbf, 0xbf, 0xbf
        ]
    );
}

const WORKER_YIELD: &str = r#"def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, IO.print("child"))
    f : File <- IO.try(File, File.open("worker.tmp", "w"))
    Unit <- IO.print("opened")
    File.close(f)
"#;

#[test]
fn file_open_parks_until_ready_tasks_have_run() {
    Fixture::new(WORKER_YIELD).success("child\nopened\n");
}

const ZERO_READS: &str = r#"def main() -> IO(Unit):
  do IO<Unit>:
    f : File <- IO.try(File, File.open("zero.tmp", "w"))
    pair : File & Result<&1, &1, (U32 & String), String> <- File.read(f, 0)
    f : File <- keep_text(pair)
    pair : File & Result<&1, &1, (U32 & String), List<&2, U32>> <- File.read_bytes(f, 0)
    f : File <- keep_bytes(pair)
    pair : File & Result<&1, &1, (U32 & String), Unit> <- File.write(f, "x")
    f : File <- keep_write(pair)
    Unit <- File.close(f)
    f : File <- IO.try(File, File.open("zero.tmp", "r"))
    pair : File & Result<&1, &1, (U32 & String), String> <- File.read(f, 0)
    f : File <- keep_text(pair)
    pair : File & Result<&1, &1, (U32 & String), String> <- File.read(f, 1)
    f : File <- keep_text(pair)
    File.close(f)
"#;

#[test]
fn zero_reads_preserve_the_cursor_and_the_host_access_check() {
    Fixture::new(ZERO_READS).success(if cfg!(windows) {
        "text:\nbytes:\ntext:\ntext:x\n"
    } else {
        "error:9\nerror:9\ntext:\ntext:x\n"
    });
}

#[test]
fn read_limits_fail_before_running_a_continuation_or_allocating_an_unbounded_buffer() {
    for operation in ["read", "read_bytes"] {
        for count in ["8388609", "4294967295"] {
            let result_type = if operation == "read" {
                "String"
            } else {
                "List<&2, U32>"
            };
            let source = format!(
                "def main() -> IO(Unit):\n  do IO<Unit>:\n    f : File <- IO.try(File, File.open(\"input\", \"r\"))\n    pair : File & Result<&1, &1, (U32 & String), {result_type}> <- File.{operation}(f, {count})\n    IO.print(\"BAD\")\n"
            );
            let fixture = Fixture::new(&source);
            std::fs::write(fixture.0.join("input"), b"x").unwrap();
            let output = fixture.run();
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8_lossy(&output.stderr).contains("budget"));
        }
    }
}

const READ_LIMIT_BOUNDARY: &str = r#"def main() -> IO(Unit):
  do IO<Unit>:
    file : File <- IO.try(File, File.open("input", "r"))
    pair : File & Result<&1, &1, (U32 & String), String> <- File.read(file, 8388608)
    file : File <- keep_text(pair)
    Unit <- File.close(file)
    file : File <- IO.try(File, File.open("input", "r"))
    pair : File & Result<&1, &1, (U32 & String), List<&2, U32>> <- File.read_bytes(file, 8388608)
    file : File <- keep_bytes(pair)
    File.close(file)
"#;

#[test]
fn a_read_at_the_buffer_limit_accepts_a_short_host_result() {
    let fixture = Fixture::new(READ_LIMIT_BOUNDARY);
    std::fs::write(fixture.0.join("input"), b"x").unwrap();
    fixture.success("text:x\nbytes:120,\n");
}

const CHILD_AFTER_MAIN: &str = r#"def child() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.sleep(0)
    f : File <- IO.try(File, File.open("child.tmp", "w"))
    pair : File & Result<&1, &1, (U32 & String), Unit> <- File.write(f, "child-value")
    f : File <- keep_write(pair)
    Unit <- File.close(f)
    IO.print("child-complete")

def main() -> IO(Unit): IO.spawn(Unit, child())
"#;

#[test]
fn a_timer_and_file_worker_in_a_child_remain_live_after_main_returns() {
    let fixture = Fixture::new(CHILD_AFTER_MAIN);
    fixture.success("child-complete\n");
    assert_eq!(
        std::fs::read(fixture.0.join("child.tmp")).unwrap(),
        b"child-value"
    );
}

const ERROR_TEXT: &str = r#"def report(result: Result<&1, &1, (U32 & String), File>) -> IO(Unit):
  match result:
    case Done{file}: File.close(file)
    case Fail{error}:
      (code, message) = error
      IO.print(U32.show(code) ++ ":" ++ message)

def main() -> IO(Unit):
  do IO<Unit>:
    result : Result<&1, &1, (U32 & String), File> <- File.open("missing", "r")
    Unit <- report(result)
    result : Result<&1, &1, (U32 & String), File> <- File.open("missing", "r+")
    report(result)
"#;

#[test]
fn ordinary_open_errors_report_native_errno_and_error_text() {
    Fixture::new(ERROR_TEXT).success("2:No such file or directory\n22:Invalid argument\n");
}

const REMEMBER_FILE: &str = r#"def remember(file: File) -> IO(File): import "effect.c"
def main() -> IO(Unit):
  do IO<Unit>:
    file : File <- IO.try(File, File.open("owned.tmp", "w"))
    file : File <- remember(file)
"#;

const OBSERVE_CLOSE: &str = r"
static int remembered_fd = -1;
#ifdef _WIN32
static HANDLE remembered_handle = INVALID_HANDLE_VALUE;
#endif
static Term remember_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)work;
  remembered_fd = (int)io_hand_v(fields[0]);
#ifdef _WIN32
  remembered_handle = (HANDLE)_get_osfhandle(remembered_fd);
  if (remembered_handle == INVALID_HANDLE_VALUE) abort();
#endif
  return fields[0];
}
static void observe_closed(void) {
  if (remembered_fd < 0) abort();
#ifdef _WIN32
  BY_HANDLE_FILE_INFORMATION info;
  if (GetFileInformationByHandle(remembered_handle, &info) != 0 || GetLastError() != ERROR_INVALID_HANDLE) abort();
#else
  errno = 0;
  if (fcntl(remembered_fd, F_GETFD) != -1 || errno != EBADF) abort();
#endif
}
static void __attribute__((constructor)) remember_use(void) {
  io_eff(CID_REMEMBER, remember_run, 0);
  if (atexit(observe_closed) != 0) abort();
}
";

#[test]
fn completed_file_opens_close_on_normal_exit_and_halt_before_process_teardown() {
    for (ending, code, stderr) in [
        ("IO.pure(Unit, Unit{})", 0, ""),
        ("IO.die(Unit, 7, \"halt\")", 7, "halt\n"),
    ] {
        let fixture = Fixture::new(&format!("{REMEMBER_FILE}    {ending}\n"));
        std::fs::write(fixture.0.join("effect.c"), OBSERVE_CLOSE).unwrap();
        let output = fixture.run();
        assert_eq!(output.status.code(), Some(code));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, stderr.as_bytes());
    }
}

const INVALID_FILE: &str = r#"def invalid() -> IO(File): import "effect.c"
def main() -> IO(Unit):
  do IO<Unit>:
    file : File <- invalid()
    pair : File & Result<&1, &1, (U32 & String), String> <- File.read(file, 1)
    file : File <- keep_text(pair)
    pair : File & Result<&1, &1, (U32 & String), Unit> <- File.write(file, "nonempty")
    file : File <- keep_write(pair)
    pair : File & Result<&1, &1, (U32 & String), Unit> <- File.write(file, "")
    file : File <- keep_write(pair)
    Unit <- File.close(file)
    IO.print("closed")
"#;

const INVALID_FILE_C: &str = r"
static Term invalid_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields; (void)work;
  return io_hand(TEST_INVALID_DESCRIPTOR);
}
static void __attribute__((constructor)) invalid_use(void) {
  io_eff(CID_INVALID, invalid_run, 0);
}
";

#[test]
fn foreign_invalid_descriptor_preserves_errno_empty_write_and_discarded_close_errors() {
    // UINT32_MAX tests descriptor range validation; INT_MAX reaches the CRT's
    // invalid-parameter handler because it is representable as a signed int.
    for definition in [
        "TEST_INVALID_DESCRIPTOR=UINT32_MAX",
        "TEST_INVALID_DESCRIPTOR=INT_MAX",
    ] {
        let fixture = Fixture::new(INVALID_FILE);
        std::fs::write(fixture.0.join("effect.c"), INVALID_FILE_C).unwrap();
        let output = fixture.run_with_definitions(&[definition]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(output.stdout, b"error:9\nerror:9\nclosed\n");
        assert!(output.stderr.is_empty());
    }
}

#[cfg(windows)]
const SURROGATE_ENVIRONMENT: &str = r#"def prepare() -> IO(Unit): import "effect.c"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- prepare()
    value : Result<&1, &1, (U32 & String), String> <- IO.get_env("TEAMY_BEND_FILE_TEST_SURROGATE")
    text_result(value)
"#;

#[cfg(windows)]
const SURROGATE_ENVIRONMENT_C: &str = r#"
static Term prepare_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields; (void)work;
  return term_pak(CID_UNIT, 0);
}
static void __attribute__((constructor)) prepare_use(void) {
  if (!SetEnvironmentVariableW(L"TEAMY_BEND_FILE_TEST_SURROGATE", L"\xd800")) abort();
  io_eff(CID_PREPARE, prepare_run, 0);
}
"#;

#[cfg(windows)]
#[test]
fn an_unpaired_utf16_environment_value_fails_with_eilseq_instead_of_replacement_text() {
    let fixture = Fixture::new(SURROGATE_ENVIRONMENT);
    std::fs::write(fixture.0.join("effect.c"), SURROGATE_ENVIRONMENT_C).unwrap();
    fixture.success("error:42\n");
}

#[cfg(windows)]
const INFLIGHT_FILE: &str = r#"def pipe_name() -> IO(String): import "effect.c"
def remember(file: File) -> IO(File): import "effect.c"
def child(file: File) -> IO(Unit):
  do IO<Unit>:
    pair : File & Result<&1, &1, (U32 & String), String> <- File.read(file, 1)
    file : File <- keep_text(pair)
    Unit <- File.close(file)
    IO.print("BAD resumed after Halt")

def main() -> IO(Unit):
  do IO<Unit>:
    path : String <- pipe_name()
    file : File <- IO.try(File, File.open(path, "r"))
    file : File <- remember(file)
    Unit <- IO.spawn(Unit, child(file))
    Unit <- IO.sleep(0)
    IO.die(Unit, 7, "halt")
"#;

#[cfg(windows)]
const INFLIGHT_FILE_C: &str = r#"
static HANDLE server_pipe = INVALID_HANDLE_VALUE;
static HANDLE client_pipe = INVALID_HANDLE_VALUE;
static Term pipe_name_run(Env e, Term *fields, IoWork *work) {
  char name[128]; int length; (void)fields; (void)work;
  length = snprintf(name, sizeof(name), "\\\\.\\pipe\\teamy-bend-c-file-%lu", (unsigned long)GetCurrentProcessId());
  if (length <= 0 || (size_t)length >= sizeof(name)) abort();
  server_pipe = CreateNamedPipeA(name, PIPE_ACCESS_OUTBOUND, PIPE_TYPE_BYTE | PIPE_WAIT, 1, 4096, 4096, 0, NULL);
  if (server_pipe == INVALID_HANDLE_VALUE) abort();
  return io_str(e, name, (u64)length);
}
static Term remember_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)work;
  client_pipe = (HANDLE)_get_osfhandle((int)io_hand_v(fields[0]));
  if (client_pipe == INVALID_HANDLE_VALUE) abort();
  return fields[0];
}
static void observe_inflight_close(void) {
  DWORD flags, written = 0;
  if (client_pipe == INVALID_HANDLE_VALUE || !GetHandleInformation(client_pipe, &flags)) abort();
  if (!WriteFile(server_pipe, "x", 1, &written, NULL) || written != 1) abort();
  for (unsigned attempt = 0; attempt < 5000; ++attempt) {
    if (!GetHandleInformation(client_pipe, &flags)) {
      if (GetLastError() != ERROR_INVALID_HANDLE) abort();
      (void)CloseHandle(server_pipe);
      return;
    }
    Sleep(1);
  }
  abort();
}
static void __attribute__((constructor)) file_probe_use(void) {
  io_eff(CID_PIPE_NAME, pipe_name_run, 0);
  io_eff(CID_REMEMBER, remember_run, 0);
  if (atexit(observe_inflight_close) != 0) abort();
}
"#;

#[cfg(windows)]
#[test]
fn halt_retains_an_inflight_file_read_until_its_worker_finishes_and_then_closes_it() {
    let fixture = Fixture::new(INFLIGHT_FILE);
    std::fs::write(fixture.0.join("effect.c"), INFLIGHT_FILE_C).unwrap();
    let output = fixture.run();
    assert_eq!(
        output.status.code(),
        Some(7),
        "file worker/Halt fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"halt\n");
}

#[cfg(windows)]
const QUEUED_FILE: &str = r#"def pipe_name() -> IO(String): import "effect.c"
def remember(file: File) -> IO(File): import "effect.c"
def remember_queued(file: File) -> IO(File): import "effect.c"
def blocked(file: File) -> IO(Unit):
  do IO<Unit>:
    pair : File & Result<&1, &1, (U32 & String), String> <- File.read(file, 1)
    IO.print("BAD resumed read after Halt")

def queued(file: File) -> IO(Unit):
  do IO<Unit>:
    pair : File & Result<&1, &1, (U32 & String), Unit> <- File.write(file, "BAD queued write")
    IO.print("BAD resumed write after Halt")

def main() -> IO(Unit):
  do IO<Unit>:
    path : String <- pipe_name()
    pipe : File <- IO.try(File, File.open(path, "r"))
    pipe : File <- remember(pipe)
    file : File <- IO.try(File, File.open("queued.tmp", "w"))
    file : File <- remember_queued(file)
    Unit <- IO.spawn(Unit, blocked(pipe))
    Unit <- IO.spawn(Unit, queued(file))
    Unit <- IO.sleep(0)
    IO.die(Unit, 7, "halt")
"#;

#[cfg(windows)]
const QUEUED_FILE_C: &str = r"
static HANDLE queued_file = INVALID_HANDLE_VALUE;
static Term remember_queued_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)work;
  queued_file = (HANDLE)_get_osfhandle((int)io_hand_v(fields[0]));
  if (queued_file == INVALID_HANDLE_VALUE) abort();
  return fields[0];
}
static void observe_queued_close(void) {
  DWORD flags;
  if (queued_file == INVALID_HANDLE_VALUE) abort();
  if (GetHandleInformation(queued_file, &flags) || GetLastError() != ERROR_INVALID_HANDLE) abort();
}
static void __attribute__((constructor)) queued_probe_use(void) {
  io_eff(CID_REMEMBER_QUEUED, remember_queued_run, 0);
  /* Registered last, so this runs before the observer releases the pipe read. */
  if (atexit(observe_queued_close) != 0) abort();
}
";

#[cfg(windows)]
#[test]
fn halt_closes_queued_file_jobs_before_the_active_worker_releases_its_own_handle() {
    let fixture = Fixture::new(QUEUED_FILE);
    std::fs::write(
        fixture.0.join("effect.c"),
        format!("{INFLIGHT_FILE_C}{QUEUED_FILE_C}"),
    )
    .unwrap();
    let output = fixture.run_with_definitions(&["BEND_MAX_WORKERS=1"]);
    assert_eq!(
        output.status.code(),
        Some(7),
        "queued file/Halt fixture failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"halt\n");
    assert!(
        std::fs::read(fixture.0.join("queued.tmp"))
            .unwrap()
            .is_empty()
    );
}
