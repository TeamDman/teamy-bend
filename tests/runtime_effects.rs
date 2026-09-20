// SPDX-License-Identifier: MPL-2.0
use std::cell::Cell;
use std::cell::RefCell;
use std::io::Write;
use std::io::{self};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::kernel::ExecutableBook;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-effects-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir(&path).expect("create effect fixture directory");
        std::fs::write(path.join("main.bend"), source).expect("write effect fixture");
        Self(path)
    }

    fn checked(&self) -> ExecutableBook {
        let source = load_executable(self.0.join("main.bend")).expect("load execution fixture");
        check_executable(&source).expect("type check execution fixture")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(source: &str) -> (u32, Vec<u8>, Vec<u8>) {
    let fixture = Fixture::new(source);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = fixture
        .checked()
        .run_main(&mut stdout, &mut stderr, &|| false)
        .expect("execute fixture");
    (code, stdout, stderr)
}

#[test]
fn successful_io_discards_payload_even_when_it_is_a_type_or_function() {
    for body in [
        "def main() -> IO(U32): IO.pure(U32, 7)",
        "def main() -> IO(Type): IO.pure(Type, Nat)",
        "def main() -> IO(Nat -> Nat): IO.pure(Nat -> Nat, x => x)",
    ] {
        assert_eq!(run(&format!("import Base\n{body}\n")), (0, vec![], vec![]));
    }
}

#[test]
fn bind_orders_print_write_and_computed_text() {
    let (code, stdout, stderr) = run(r#"import Base
def main() -> IO(U32):
  do IO<U32>:
    a : Unit <- IO.print("hello")
    b : Unit <- IO.write("num=")
    c : Unit <- IO.print(U32.show((40 + 2 : U32)))
    return 7
"#);
    assert_eq!(code, 0);
    assert_eq!(stdout, b"hello\nnum=42\n");
    assert!(stderr.is_empty());
}

#[test]
fn console_preserves_utf8_non_bmp_and_embedded_nul() {
    let (code, stdout, stderr) = run(r#"import Base
def main() -> IO(Unit): IO.print("a\0è❁🙂z")
"#);
    assert_eq!(code, 0);
    assert_eq!(stdout, "a\0è❁🙂z\n".as_bytes());
    assert!(stderr.is_empty());
}

#[test]
fn halt_reports_u32_status_and_stops_later_effects() {
    let (code, stdout, stderr) = run(r#"import Base
def main() -> IO(Unit):
  do IO<Unit>:
    a : Unit <- IO.write("before")
    b : Unit <- IO.die(Unit, 4294967295, "halt\0è❁🙂")
    c : Unit <- IO.print("after")
    return Unit{}
"#);
    assert_eq!(code, u32::MAX);
    assert_eq!(stdout, b"before");
    assert_eq!(stderr, "halt\0è❁🙂\n".as_bytes());
}

#[test]
fn unused_effects_are_inert_and_pure_io_operations_remain_matchable() {
    let (code, stdout, stderr) = run(r#"import Base
def choose(flag: Bool) -> IO(Unit):
  match flag:
    case False{}: IO.pure(Unit, Unit{})
    case True{}: IO.print("UNUSED")
def decode(op: IO.OP<String>) -> String:
  match op:
    case Emit{v}: v
    case Halt{c, m}: m
def main() -> IO(Unit):
  do IO<Unit>:
    a : Unit <- choose(False{})
    b : Unit <- IO.print(decode(IO.pure(String, "pure")(String, x => Emit{x})))
    return Unit{}
"#);
    assert_eq!(code, 0);
    assert_eq!(stdout, b"pure\n");
    assert!(stderr.is_empty());
}

#[test]
fn matching_a_foreign_request_fails_before_performing_the_effect() {
    for arms in [
        "    case Emit{v}: IO.pure(Unit, Unit{})\n    case Halt{c, m}: IO.print(m)",
        "    case Emit{v}: IO.pure(Unit, Unit{})\n    case rest: IO.print(\"FALLBACK\")",
    ] {
        let fixture = Fixture::new(&format!(
            "import Base\ndef intercept(op: IO.OP<Unit>) -> IO(Unit):\n  match op:\n{arms}\ndef main() -> IO(Unit): intercept(IO.print(\"SECRET\")(Unit, x => Emit{{x}}))\n"
        ));
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let error = fixture
            .checked()
            .run_main(&mut stdout, &mut stderr, &|| false)
            .unwrap_err();
        assert!(
            error.to_string().contains("foreign effect request"),
            "{error}"
        );
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }
}

#[test]
fn unsupported_foreign_contracts_do_not_receive_builtin_privileges() {
    let fixture = Fixture::new(
        r#"import Base
def print(text: String) -> IO(Unit):
  import "./print.js"
def main() -> IO(Unit): print("SECRET")
"#,
    );
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let error = fixture
        .checked()
        .run_main(&mut stdout, &mut stderr, &|| false)
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("does not support the foreign implementation of print")
    );
    assert!(stdout.is_empty());
    assert!(stderr.is_empty());
}

#[test]
fn console_flushes_stdout_before_stderr_and_halt() {
    type Events = Rc<RefCell<Vec<(&'static str, Vec<u8>)>>>;
    struct Recorder {
        stream: &'static str,
        events: Events,
    }
    impl Write for Recorder {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.events.borrow_mut().push((self.stream, buf.to_vec()));
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            self.events.borrow_mut().push((self.stream, vec![]));
            Ok(())
        }
    }
    let fixture = Fixture::new(
        r#"import Base
def main() -> IO(Unit):
  do IO<Unit>:
    a : Unit <- IO.write("out")
    b : Unit <- IO.print_err("err")
    c : Unit <- IO.die(Unit, 3, "halt")
    return Unit{}
"#,
    );
    let events = Rc::new(RefCell::new(Vec::new()));
    let mut stdout = Recorder {
        stream: "out",
        events: Rc::clone(&events),
    };
    let mut stderr = Recorder {
        stream: "err",
        events: Rc::clone(&events),
    };
    assert_eq!(
        fixture
            .checked()
            .run_main(&mut stdout, &mut stderr, &|| false)
            .unwrap(),
        3
    );
    assert_eq!(
        *events.borrow(),
        vec![
            ("out", b"out".to_vec()),
            ("out", vec![]),
            ("err", b"err".to_vec()),
            ("err", b"\n".to_vec()),
            ("err", vec![]),
            ("out", vec![]),
            ("err", b"halt".to_vec()),
            ("err", b"\n".to_vec()),
            ("err", vec![]),
        ]
    );
}

#[test]
fn cancellation_is_checked_before_execution_and_between_partial_writes() {
    struct CancellingWriter<'a> {
        cancelled: &'a Cell<bool>,
        bytes: Vec<u8>,
    }
    impl Write for CancellingWriter<'_> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.bytes.push(buf[0]);
            self.cancelled.set(true);
            Ok(1)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let fixture = Fixture::new("import Base\ndef main() -> IO(Unit): IO.print(\"hello\")\n");
    let book = fixture.checked();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let error = book
        .run_main(&mut stdout, &mut stderr, &|| true)
        .unwrap_err();
    assert!(error.to_string().contains("cancelled"));
    assert!(stdout.is_empty());
    let cancelled = Cell::new(false);
    let mut writer = CancellingWriter {
        cancelled: &cancelled,
        bytes: vec![],
    };
    let error = book
        .run_main(&mut writer, &mut stderr, &|| cancelled.get())
        .unwrap_err();
    assert!(error.to_string().contains("cancelled"));
    assert_eq!(writer.bytes, b"h");
    assert!(stderr.is_empty());
    let ticks = Cell::new(0);
    let error = book
        .run_main(&mut stdout, &mut stderr, &|| {
            ticks.set(ticks.get() + 1);
            ticks.get() > 20
        })
        .unwrap_err();
    assert!(error.to_string().contains("cancelled"));
    assert!(stdout.is_empty());
}

#[test]
fn partial_and_interrupted_writes_preserve_the_complete_byte_stream() {
    struct PartialWriter {
        interrupted: bool,
        bytes: Vec<u8>,
    }
    impl Write for PartialWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::ErrorKind::Interrupted.into());
            }
            let count = 2.min(buf.len());
            self.bytes.extend_from_slice(&buf[..count]);
            Ok(count)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let fixture = Fixture::new("import Base\ndef main() -> IO(Unit): IO.print(\"è🙂\\0\")\n");
    let mut stdout = PartialWriter {
        interrupted: false,
        bytes: vec![],
    };
    let mut stderr = Vec::new();
    assert_eq!(
        fixture
            .checked()
            .run_main(&mut stdout, &mut stderr, &|| false)
            .unwrap(),
        0
    );
    assert_eq!(stdout.bytes, "è🙂\0\n".as_bytes());
    assert!(stderr.is_empty());
}

#[test]
fn writer_zero_errors_and_flush_errors_are_propagated() {
    struct FailingWriter {
        mode: u8,
    }
    impl Write for FailingWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            match self.mode {
                0 => Ok(0),
                1 => Err(io::Error::other("write sentinel")),
                _ => Ok(buf.len()),
            }
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("flush sentinel"))
        }
    }
    let fixture = Fixture::new("import Base\ndef main() -> IO(Unit): IO.print(\"hello\")\n");
    let book = fixture.checked();
    for (mode, expected) in [
        (0, "zero-length write"),
        (1, "write sentinel"),
        (2, "flush sentinel"),
    ] {
        let error = book
            .run_main(&mut FailingWriter { mode }, &mut Vec::new(), &|| false)
            .unwrap_err();
        assert!(error.to_string().contains(expected), "{error}");
    }
}
