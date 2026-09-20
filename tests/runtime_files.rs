// SPDX-License-Identifier: MPL-2.0
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

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
            "teamy-bend-native-files-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).unwrap();
        std::fs::write(directory.join("main.bend"), format!("{HELPERS}{source}")).unwrap();
        Self(directory)
    }

    fn run(&self) -> Output {
        // Never change the test process's environment or current directory.
        Command::new(env!("CARGO_BIN_EXE_teamy-bend"))
            .args(["run", "main.bend"])
            .current_dir(&self.0)
            .env("TEAMY_BEND_FILE_TEST_VALUE", "Olá 日本 🙂")
            .env("TEAMY_BEND_FILE_TEST_EMPTY", "")
            .env_remove("TEAMY_BEND_FILE_TEST_MISSING")
            .env_remove("toString")
            .env_remove("hasOwnProperty")
            .output()
            .expect("execute native file fixture")
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
