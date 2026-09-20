// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::kernel::Declaration;
use crate::kernel::check_executable;
use crate::runtime::gc;
use crate::syntax::load_executable;
use std::cell::Cell;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static NEXT: AtomicUsize = AtomicUsize::new(0);

#[derive(Default)]
struct TestClock {
    time: u64,
    cancel_on_wait: Option<Rc<Cell<bool>>>,
}

impl Clock for TestClock {
    fn now(&mut self) -> Result<u64, KernelError> {
        Ok(self.time)
    }

    fn wait(&mut self, nanoseconds: u64) -> Result<(), KernelError> {
        self.time += nanoseconds;
        if let Some(cancelled) = &self.cancel_on_wait {
            cancelled.set(true);
        }
        Ok(())
    }
}

fn program(source: &str) -> Program {
    let directory = std::env::temp_dir().join(format!(
        "teamy-bend-channel-roots-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&directory).unwrap();
    let main = directory.join("main.bend");
    std::fs::write(&main, format!("import Base\n{source}")).unwrap();
    let loaded = load_executable(&main).unwrap();
    // These are the same checked declarations and sealed loader metadata used
    // by the public path. The direct Machine only enables forced collection
    // and a deterministic clock; no fabricated unchecked requests are needed.
    check_executable(&loaded).unwrap();
    std::fs::remove_dir_all(&directory).unwrap();
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

fn assert_clean(machine: &Machine<'_>) {
    assert!(machine.scheduler.finished());
    let mut roots = Vec::new();
    machine
        .channels
        .visit_roots(|id| {
            roots.push(id);
            Ok(())
        })
        .unwrap();
    assert!(roots.is_empty(), "terminal driver retained channel roots");
}

fn collected(source: &str, expected: &str) {
    let program = program(source);
    let mut machine = Machine::new(&program);
    machine.gc_mode = gc::Mode::EverySafePoint;
    let main = machine.reference("main").unwrap();
    let first = machine.io_action(main).unwrap();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        machine
            .drive_io_with_clock(first, &mut stdout, &mut stderr, &mut TestClock::default())
            .unwrap(),
        0
    );
    assert_eq!(stdout, expected.as_bytes());
    assert!(stderr.is_empty());
    assert!(machine.gc.collections > 20);
    assert_clean(&machine);
}

#[test]
fn buffered_payloads_and_sender_receiver_continuations_survive_every_safe_point() {
    collected(
        r#"def show(value: Maybe<&1, String>) -> String:
  match value:
    case None{}: "none"
    case Some{x}: x

def send(c: Chan(String), value: String, tag: String) -> IO(Unit):
  do IO<Unit>:
    ok : Bool <- Chan.send(String, c, value)
    IO.print(String.append(tag, Bool.show(ok)))

def receive(c: Chan(String), tag: String) -> IO(Unit):
  do IO<Unit>:
    value : Maybe<&1, String> <- Chan.recv(String, c)
    IO.print(String.append(tag, show(value)))

def use(a: Chan(String), b: Chan(String)) -> IO(Unit):
  +buffer = a
  +rendezvous = b
  do IO<Unit>:
    ok : Bool <- Chan.send(String, buffer, String.append("buf", "fer"))
    Unit <- IO.spawn(Unit, send(buffer, String.append("que", "ued"), "sender"))
    Unit <- IO.spawn(Unit, receive(rendezvous, String.append("rece", "iver")))
    Unit <- IO.sleep(0)
    first : Maybe<&1, String> <- Chan.recv(String, buffer)
    Unit <- IO.print(show(first))
    second : Maybe<&1, String> <- Chan.recv(String, buffer)
    Unit <- IO.print(show(second))
    ok : Bool <- Chan.send(String, rendezvous, String.append("deli", "vered"))
    IO.print("main")

def main() -> IO(Unit):
  do IO<Unit>:
    a : Chan(String) <- Chan.new(String, 1)
    b : Chan(String) <- Chan.new(String, 0)
    use(a, b)
"#,
        "buffer\nqueued\nmain\nsenderTrue\nreceiverdelivered\n",
    );
}

#[test]
fn close_preserves_buffer_and_all_pending_wakes_during_forced_collection() {
    collected(
        r#"def show(value: Maybe<&1, String>) -> String:
  match value:
    case None{}: "none"
    case Some{x}: x

def send(c: Chan(String), tag: String) -> IO(Unit):
  do IO<Unit>:
    ok : Bool <- Chan.send(String, c, String.append("dis", "carded"))
    IO.print(String.append(tag, Bool.show(ok)))

def receive(c: Chan(String), tag: String) -> IO(Unit):
  do IO<Unit>:
    value : Maybe<&1, String> <- Chan.recv(String, c)
    IO.print(String.append(tag, show(value)))

def use(a: Chan(String), b: Chan(String)) -> IO(Unit):
  +buffer = a
  +empty = b
  do IO<Unit>:
    ok : Bool <- Chan.send(String, buffer, String.append("buf", "fer"))
    Unit <- IO.spawn(Unit, send(buffer, "a"))
    Unit <- IO.spawn(Unit, send(buffer, "b"))
    Unit <- IO.spawn(Unit, receive(empty, "c"))
    Unit <- IO.spawn(Unit, receive(empty, "d"))
    Unit <- IO.sleep(0)
    Unit <- Chan.close(String, buffer)
    Unit <- Chan.close(String, empty)
    Unit <- Chan.close(String, buffer)
    first : Maybe<&1, String> <- Chan.recv(String, buffer)
    IO.print(show(first))

def main() -> IO(Unit):
  do IO<Unit>:
    a : Chan(String) <- Chan.new(String, 1)
    b : Chan(String) <- Chan.new(String, 0)
    use(a, b)
"#,
        "buffer\naFalse\nbFalse\ncnone\ndnone\n",
    );
}

#[test]
fn nested_handles_and_closure_payloads_survive_collection_and_join_close() {
    collected(
        r#"def apply(f: Unit -> String) -> IO(Unit): IO.print(f(Unit{}))

def transfer(c: Chan(String)) -> IO(Unit):
  +ch = c
  do IO<Unit>:
    ok : Bool <- Chan.send(String, ch, String.append("pay", "load"))
    carrier : Chan(Chan(String)) <- IO.fork(Chan(String), IO.pure(Chan(String), ch))
    received : Chan(String) <- IO.join(Chan(String), carrier)
    n : String <- IO.join(String, received)
    f : Chan(Unit -> String) <- IO.fork(Unit -> String, IO.pure(Unit -> String, x => n))
    fun : (Unit -> String) <- IO.join(Unit -> String, f)
    apply(fun)

def main() -> IO(Unit): IO.bind(Chan(String), Unit, Chan.new(String, 1), transfer)
"#,
        "payload\n",
    );
}

#[test]
fn halt_and_cancellation_discard_all_channel_roots() {
    for cancel in [false, true] {
        let terminal = if cancel {
            "IO.sleep(4294967295)"
        } else {
            "IO.die(Unit, 7, \"stop\")"
        };
        let program = program(&format!(
            r#"def send(c: Chan(String)) -> IO(Unit):
  do IO<Unit>:
    ok : Bool <- Chan.send(String, c, String.append("pending", "payload"))
    IO.print("BAD sender")

def receive(c: Chan(String)) -> IO(Unit):
  do IO<Unit>:
    got : Maybe<&1, String> <- Chan.recv(String, c)
    IO.print("BAD receiver")

def use(a: Chan(String), b: Chan(String)) -> IO(Unit):
  +buffer = a
  do IO<Unit>:
    ok : Bool <- Chan.send(String, buffer, "buffer")
    Unit <- IO.spawn(Unit, send(buffer))
    Unit <- IO.spawn(Unit, receive(b))
    Unit <- IO.sleep(0)
    {terminal}

def main() -> IO(Unit):
  do IO<Unit>:
    a : Chan(String) <- Chan.new(String, 1)
    b : Chan(String) <- Chan.new(String, 0)
    use(a, b)
"#
        ));
        let mut machine = Machine::new(&program);
        machine.gc_mode = gc::Mode::EverySafePoint;
        let main = machine.reference("main").unwrap();
        let first = machine.io_action(main).unwrap();
        let cancelled = Rc::new(Cell::new(false));
        let cancellation = || cancelled.get();
        machine.cancelled = Some(&cancellation);
        let mut clock = TestClock {
            cancel_on_wait: cancel.then(|| Rc::clone(&cancelled)),
            ..TestClock::default()
        };
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = machine.drive_io_with_clock(first, &mut stdout, &mut stderr, &mut clock);
        if cancel {
            let error = result.unwrap_err();
            assert!(error.to_string().contains("cancelled"), "{error}");
            assert!(stderr.is_empty());
        } else {
            assert_eq!(result.unwrap(), 7);
            assert_eq!(stderr, b"stop\n");
        }
        assert!(stdout.is_empty());
        assert!(machine.gc.collections > 20);
        assert_clean(&machine);
    }
}

#[test]
fn deadlock_error_drops_pending_payloads_and_continuations() {
    let program = program(
        r#"def send(c: Chan(String)) -> IO(Unit):
  do IO<Unit>:
    ok : Bool <- Chan.send(String, c, String.append("pending", "payload"))
    IO.print("BAD")

def main() -> IO(Unit): IO.bind(Chan(String), Unit, Chan.new(String, 0), send)
"#,
    );
    let mut machine = Machine::new(&program);
    machine.gc_mode = gc::Mode::EverySafePoint;
    let main = machine.reference("main").unwrap();
    let first = machine.io_action(main).unwrap();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let error = machine
        .drive_io_with_clock(first, &mut stdout, &mut stderr, &mut TestClock::default())
        .unwrap_err();
    assert!(error.to_string().contains("deadlock"), "{error}");
    assert!(stdout.is_empty() && stderr.is_empty());
    assert!(machine.gc.collections > 20);
    assert_clean(&machine);
}
