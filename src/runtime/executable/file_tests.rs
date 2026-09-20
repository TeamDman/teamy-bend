// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::kernel::Declaration;
use crate::kernel::Term;
use crate::kernel::check_executable;
use crate::kernel::term;
use crate::runtime::gc;
use crate::runtime::host_files;
use crate::runtime::host_jobs;
use crate::syntax::load_executable;
use crate::syntax::parse_term;
use std::cell::Cell;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-file-roots-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn program(&self, source: &str) -> Program {
        let main = self.0.join("main.bend");
        let file = self.0.join("data.tmp").to_string_lossy().replace('\\', "/");
        std::fs::write(
            &main,
            format!(
                "import Base\n{}",
                source.replace("FILE_PATH", &format!("{file:?}"))
            ),
        )
        .unwrap();
        let loaded = load_executable(&main).unwrap();
        check_executable(&loaded).unwrap();
        let mut definitions = BTreeMap::new();
        let mut datatypes = BTreeMap::new();
        for declaration in loaded.book.declarations {
            match declaration {
                Declaration::Def(definition) => {
                    definitions.insert(definition.name.clone(), definition);
                }
                Declaration::Adt(datatype) => {
                    datatypes.insert(datatype.name.clone(), datatype);
                }
            }
        }
        Program::from_executable(
            &Rc::new(definitions),
            &Rc::new(datatypes),
            &loaded.foreign,
            &loaded.numeric,
            &loaded.base_names,
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(&self.0);
    }
}

fn assert_clean(machine: &Machine<'_>) {
    assert!(machine.files.is_empty());
    assert!(!machine.jobs.has_pending());
    assert!(machine.scheduler.finished());
}

#[test]
fn file_waits_preserve_captured_continuations_and_returned_handles_during_gc() {
    let fixture = Fixture::new();
    let program = fixture.program(
        r#"def keep(pair: File & Result<&1, &1, U32 & String, Unit>) -> IO(File):
  (file, result) = pair
  do IO<File>:
    Unit <- IO.pass(Unit, result)
    return file

def saved(message: String) -> IO(Unit):
  +text = message
  do IO<Unit>:
    Unit <- IO.spawn(Unit, IO.print("child"))
    file : File <- IO.try(File, File.open(FILE_PATH, "w"))
    pair : File & Result<&1, &1, U32 & String, Unit> <- File.write(file, text)
    file : File <- keep(pair)
    Unit <- File.close(file)
    IO.print(text)

def main() -> IO(Unit): saved(String.append("captured", " value"))
"#,
    );
    for mode in [gc::Mode::Disabled, gc::Mode::EverySafePoint] {
        let mut machine = Machine::new(&program);
        machine.gc_mode = mode;
        let main = machine.reference("main").unwrap();
        let action = machine.io_action(main).unwrap();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(
            machine.drive_io(action, &mut stdout, &mut stderr).unwrap(),
            0
        );
        assert_eq!(stdout, b"child\ncaptured value\n");
        assert!(stderr.is_empty());
        assert_eq!(
            std::fs::read(fixture.0.join("data.tmp")).unwrap(),
            b"captured value"
        );
        if matches!(mode, gc::Mode::EverySafePoint) {
            assert!(machine.gc.collections > 20);
        }
        assert_clean(&machine);
    }
}

#[test]
fn halt_and_cancellation_discard_live_file_handles() {
    struct Writer<'a>(&'a Cell<bool>);
    impl Write for Writer<'_> {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            assert_eq!(buf, b"cancel");
            self.0.set(true);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let fixture = Fixture::new();
    for terminal in ["IO.die(Unit, 7, \"halt\")", "IO.write(\"cancel\")"] {
        let program = fixture.program(&format!("def main() -> IO(Unit):\n  do IO<Unit>:\n    file : File <- IO.try(File, File.open(FILE_PATH, \"w\"))\n    {terminal}\n"));
        let cancelled = Cell::new(false);
        let cancel = || cancelled.get();
        let mut machine = Machine::new(&program);
        machine.cancelled = Some(&cancel);
        machine.gc_mode = gc::Mode::EverySafePoint;
        let main = machine.reference("main").unwrap();
        let action = machine.io_action(main).unwrap();
        let mut stderr = Vec::new();
        let result = machine.drive_io(action, &mut Writer(&cancelled), &mut stderr);
        if terminal.contains("cancel") {
            assert!(result.unwrap_err().to_string().contains("cancelled"));
            assert!(stderr.is_empty());
        } else {
            assert_eq!(result.unwrap(), 7);
            assert_eq!(stderr, b"halt\n");
        }
        assert_clean(&machine);
    }
}

fn print(machine: &mut Machine<'_>, label: &str) -> ThunkId {
    let text = machine
        .expression(parse_term(&format!("{label:?}")).unwrap(), 0)
        .unwrap();
    let continuation = machine
        .allocate(Thunk::Ready(Value::EmitContinuation))
        .unwrap();
    machine
        .allocate(Thunk::Ready(Value::Request {
            name: "IO.print".into(),
            arguments: vec![text],
            continuation,
        }))
        .unwrap()
}

#[test]
fn ready_tasks_precede_host_completions_which_precede_due_timers_and_survive_gc() {
    let fixture = Fixture::new();
    let program = fixture.program("def main() -> IO(Unit): IO.print(\"unused\")\n");
    let mut machine = Machine::new(&program);
    machine.gc_mode = gc::Mode::EverySafePoint;
    let host = print(&mut machine, "host");
    let environment = machine.environment(0, vec![(1, host)]).unwrap();
    let continuation = machine
        .allocate(Thunk::Ready(Value::Closure {
            binder: 0,
            body: term(Term::Var {
                id: 1,
                name: "next".into(),
            }),
            environment,
        }))
        .unwrap();
    machine.scheduler.spawn(host).unwrap();
    assert_eq!(machine.scheduler.next(), Some(host));
    machine
        .jobs
        .submit(
            host_jobs::Task::Test(Box::new(|| {
                host_jobs::Reply::Open(Err(host_files::Failure::Io(host_files::Error {
                    code: 2,
                    message: b"missing".to_vec(),
                })))
            })),
            continuation,
            None,
            0,
        )
        .unwrap();
    machine.jobs.wait(5_000_000_000).unwrap();
    let timer = print(&mut machine, "timer");
    machine.scheduler.spawn(timer).unwrap();
    assert_eq!(machine.scheduler.next(), Some(timer));
    machine.scheduler.sleep(0, timer).unwrap();
    let ready = print(&mut machine, "ready");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        machine.drive_io(ready, &mut stdout, &mut stderr).unwrap(),
        0
    );
    assert_eq!(stdout, b"ready\nhost\ntimer\n");
    assert!(stderr.is_empty());
    assert!(machine.gc.collections > 10);
    assert_clean(&machine);
}
