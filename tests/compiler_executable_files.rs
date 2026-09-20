// SPDX-License-Identifier: MPL-2.0
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::compiler::compile_executable_javascript;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);
const HELPERS: &str = r#"import Base

def text(result: Result<&1, &1, U32 & String, String>) -> String:
  match result:
    case Done{value}: value
    case Fail{(code, message)}: String.append("error:", message)

def status(-A: Type, result: Result<&1, &1, U32 & String, A>) -> String:
  match result:
    case Done{value}: "ok"
    case Fail{(code, message)}: "fail"

def keep_write(pair: File & Result<&1, &1, U32 & String, Unit>) -> IO(File):
  (file, result) = pair
  do IO<File>:
    Unit <- IO.print(status(Unit, result))
    return file

def keep_text(pair: File & Result<&1, &1, U32 & String, String>) -> IO(File):
  (file, result) = pair
  do IO<File>:
    Unit <- IO.print(text(result))
    return file

def byte(value: U32) -> IO(Unit): IO.print(U32.show(value))

def keep_bytes(pair: File & Result<&1, &1, U32 & String, List<&2, U32>>) -> IO(File):
  (file, result) = pair
  do IO<File>:
    bytes : List<&2, U32> <- IO.pass(List<&2, U32>, result)
    Unit <- List.for_each(~&2, ~U32, ~byte, bytes)
    return file
"#;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-file-js-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn run(&self, source: &str, foreign: &str) -> Output {
        fs::write(self.0.join("main.bend"), format!("{HELPERS}{source}")).unwrap();
        fs::write(self.0.join("effect.js"), foreign).unwrap();
        let source = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&source).unwrap();
        let javascript = compile_executable_javascript(&checked).unwrap();
        fs::write(self.0.join("main.cjs"), javascript).unwrap();
        Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
            .arg(self.0.join("main.cjs"))
            .current_dir(&self.0)
            .env("TEAMY_BEND_FILE_ENV_VALUE", "value-é")
            .env("TEAMY_BEND_FILE_ENV_EMPTY", "")
            .env_remove("TEAMY_BEND_FILE_ENV_MISSING")
            .output()
            .expect("file compiler tests require Node.js")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn success(output: &Output, expected: &str) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, expected.as_bytes());
    assert!(output.stderr.is_empty());
}

#[test]
fn environment_distinguishes_empty_missing_and_existing_values() {
    let output = Fixture::new().run(
        r#"def main() -> IO(Unit):
  do IO<Unit>:
    a : Result<&1, &1, U32 & String, String> <- IO.get_env("TEAMY_BEND_FILE_ENV_VALUE")
    Unit <- IO.print(text(a))
    b : Result<&1, &1, U32 & String, String> <- IO.get_env("TEAMY_BEND_FILE_ENV_EMPTY")
    Unit <- IO.print(String.append("empty:", text(b)))
    c : Result<&1, &1, U32 & String, String> <- IO.get_env("TEAMY_BEND_FILE_ENV_MISSING")
    Unit <- IO.print(text(c))
    d : Result<&1, &1, U32 & String, String> <- IO.get_env("TEAMY_BEND_FILE_ENV_VALUE\0tail")
    IO.print(text(d))
"#,
        "",
    );
    let nul_result = if cfg!(windows) {
        "value-é"
    } else {
        "error:No such file or directory"
    };
    success(
        &output,
        &format!("value-é\nempty:\nerror:No such file or directory\n{nul_result}\n"),
    );
}

#[test]
fn file_write_append_read_cursor_and_eof_match_native_javascript_values() {
    let fixture = Fixture::new();
    let output = fixture.run(
        r#"def write(path: String, mode: String, data: String) -> IO(Unit):
  do IO<Unit>:
    file : File <- IO.try(File, File.open(path, mode))
    pair : File & Result<&1, &1, U32 & String, Unit> <- File.write(file, data)
    file : File <- keep_write(pair)
    File.close(file)

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- write("file-é.txt", "w", "abcdef")
    Unit <- write("file-é.txt", "a", "!")
    file : File <- IO.try(File, File.open("file-é.txt", "r"))
    pair : File & Result<&1, &1, U32 & String, String> <- File.read(file, 2)
    file : File <- keep_text(pair)
    pair : File & Result<&1, &1, U32 & String, String> <- File.read(file, 0)
    file : File <- keep_text(pair)
    pair : File & Result<&1, &1, U32 & String, String> <- File.read(file, 20)
    file : File <- keep_text(pair)
    pair : File & Result<&1, &1, U32 & String, String> <- File.read(file, 1)
    file : File <- keep_text(pair)
    File.close(file)
"#,
        "",
    );
    success(&output, "ok\nok\nab\n\ncdef!\n\n");
    assert_eq!(fs::read(fixture.0.join("file-é.txt")).unwrap(), b"abcdef!");
}

#[test]
fn file_read_preserves_javascript_textdecoder_and_raw_byte_semantics() {
    let fixture = Fixture::new();
    fs::write(
        fixture.0.join("bytes"),
        [0xef, 0xbb, 0xbf, b'X', 0xff, 0, 0xc3],
    )
    .unwrap();
    let output = fixture.run(
        r#"def main() -> IO(Unit):
  do IO<Unit>:
    file : File <- IO.try(File, File.open("bytes", "r"))
    pair : File & Result<&1, &1, U32 & String, String> <- File.read(file, 7)
    file : File <- keep_text(pair)
    Unit <- File.close(file)
    file : File <- IO.try(File, File.open("bytes", "r"))
    pair : File & Result<&1, &1, U32 & String, List<&2, U32>> <- File.read_bytes(file, 7)
    file : File <- keep_bytes(pair)
    File.close(file)
"#,
        "",
    );
    success(&output, "X�\0�\n239\n187\n191\n88\n255\n0\n195\n");
}

#[test]
fn failures_retain_the_file_and_close_discards_host_failures() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("input"), b"still-readable").unwrap();
    let output = fixture.run(
        r#"def bad() -> IO(File): import "effect.js"

def main() -> IO(Unit):
  do IO<Unit>:
    file : File <- IO.try(File, File.open("input", "r"))
    pair : File & Result<&1, &1, U32 & String, Unit> <- File.write(file, "no")
    file : File <- keep_write(pair)
    pair : File & Result<&1, &1, U32 & String, String> <- File.read(file, 30)
    file : File <- keep_text(pair)
    Unit <- File.close(file)
    invalid : File <- bad()
    Unit <- File.close(invalid)
    IO.print("closed")
"#,
        "function bad() { return -1; }",
    );
    success(&output, "fail\nstill-readable\nclosed\n");
}

#[test]
fn file_mode_validation_and_foreign_system_error_adapter_remain_observable() {
    let fixture = Fixture::new();
    let output = fixture.run(
        r#"def touch() -> IO(Unit): import "effect.js"

def show(result: Result<&1, &1, U32 & String, File>) -> String:
  match result:
    case Done{file}: "unexpected"
    case Fail{(code, message)}: message

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- touch()
    result : Result<&1, &1, U32 & String, File> <- File.open("absent", "r+")
    Unit <- IO.print(show(result))
    result : Result<&1, &1, U32 & String, File> <- File.open("absent\0tail", "r+")
    Unit <- IO.print(show(result))
    result : Result<&1, &1, U32 & String, String> <- IO.get_env("TEAMY_BEND_FILE_ENV_MISSING")
    IO.print(text(result))
"#,
        "function touch() { globalThis.BEND_SYS = {strerror: code => 'errno-' + code}; return {$:'Unit'}; }\nfunction $tbFileOpen() { throw new Error('shadowed builtin'); }",
    );
    let nul_code = if cfg!(target_os = "macos") { 92 } else { 84 };
    success(
        &output,
        &format!("errno-22\nerrno-{nul_code}\nerror:errno-2\n"),
    );
}

#[test]
fn file_read_budget_refuses_before_a_host_allocation() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("input"), b"x").unwrap();
    let output = fixture.run(
        r#"def main() -> IO(Unit):
  do IO<Unit>:
    file : File <- IO.try(File, File.open("input", "r"))
    pair : File & Result<&1, &1, U32 & String, String> <- File.read(file, 8388609)
    IO.print("unreachable")
"#,
        "",
    );
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("IO byte buffer budget exhausted"));
}

#[test]
fn file_write_retries_short_progress_and_rejects_a_zero_progress_host() {
    let source = r#"def touch() -> IO(Unit): import "effect.js"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- touch()
    file : File <- IO.try(File, File.open("output", "w"))
    pair : File & Result<&1, &1, U32 & String, Unit> <- File.write(file, "é😀")
    file : File <- keep_write(pair)
    File.close(file)
"#;
    let fixture = Fixture::new();
    let output = fixture.run(source,
        "function touch() { const fs = require('fs'); const write = fs.writeSync; fs.writeSync = (fd,b,at,len,pos) => fd > 2 ? write(fd,b,at,Math.min(1,len),pos) : write(fd,b,at,len,pos); return {$:'Unit'}; }");
    success(&output, "ok\n");
    assert_eq!(
        fs::read(fixture.0.join("output")).unwrap(),
        "é😀".as_bytes()
    );
    let fixture = Fixture::new();
    let output = fixture.run(source,
        "function touch() { const fs = require('fs'); const write = fs.writeSync; fs.writeSync = (fd,...args) => fd > 2 ? 0 : write(fd,...args); return {$:'Unit'}; }");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("short write on a file"));
}
