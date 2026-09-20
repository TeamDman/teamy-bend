// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::kernel::Term;
use crate::kernel::term;
use crate::runtime::gc;
use crate::syntax::parse_term;
use std::cell::Cell;
use std::cell::RefCell;

#[derive(Default)]
struct TestClock {
    time: Rc<Cell<u64>>,
    waits: Vec<u64>,
    oversleep: u64,
    cancel_on_wait: Option<Rc<Cell<bool>>>,
    fail_wait: bool,
}

impl Clock for TestClock {
    fn now(&mut self) -> Result<u64, KernelError> {
        Ok(self.time.get())
    }

    fn wait(&mut self, nanoseconds: u64) -> Result<(), KernelError> {
        self.waits.push(nanoseconds);
        if self.fail_wait {
            return Err(KernelError::new("test clock wait failure"));
        }
        self.time
            .set(self.time.get() + nanoseconds + self.oversleep);
        if let Some(cancelled) = &self.cancel_on_wait {
            cancelled.set(true);
        }
        Ok(())
    }
}

fn program() -> Program {
    let foreign = [
        ("IO.print", BuiltinForeign::Print, 1),
        ("IO.spawn", BuiltinForeign::Spawn, 2),
        ("IO.sleep", BuiltinForeign::Sleep, 1),
    ]
    .into_iter()
    .map(|(name, builtin, arity)| {
        (
            name.into(),
            ForeignDefinition {
                imports: vec![],
                local_symbol: name.into(),
                declared_arity: arity,
                parameters: (0..arity).map(|i| format!("p{i}")).collect(),
                builtin: Some(builtin),
            },
        )
    })
    .collect();
    Program::from_executable(
        &Rc::new(BTreeMap::new()),
        &Rc::new(BTreeMap::new()),
        &foreign,
        &BTreeMap::new(),
        &BTreeSet::new(),
    )
}

fn program_with_nat() -> Program {
    let source = crate::syntax::parse(include_str!("../../syntax/base.bend")).unwrap();
    let mut definitions = BTreeMap::new();
    let mut datatypes = BTreeMap::new();
    for declaration in source.declarations {
        match declaration {
            crate::kernel::Declaration::Def(definition) => {
                definitions.insert(definition.name.clone(), definition);
            }
            crate::kernel::Declaration::Adt(datatype) => {
                datatypes.insert(datatype.name.clone(), datatype);
            }
        }
    }
    let origins = definitions
        .keys()
        .chain(datatypes.keys())
        .cloned()
        .collect();
    let mut foreign = program().foreign.as_ref().clone();
    foreign.insert(
        "IO.now".into(),
        ForeignDefinition {
            imports: vec![],
            local_symbol: "IO.now".into(),
            declared_arity: 0,
            parameters: vec![],
            builtin: Some(BuiltinForeign::Now),
        },
    );
    Program::from_executable(
        &Rc::new(definitions),
        &Rc::new(datatypes),
        &foreign,
        &BTreeMap::new(),
        &origins,
    )
}

fn literal(machine: &mut Machine<'_>, source: &str) -> ThunkId {
    machine.expression(parse_term(source).unwrap(), 0).unwrap()
}

fn constant(machine: &mut Machine<'_>, next: ThunkId) -> ThunkId {
    let environment = machine.environment(0, vec![(1, next)]).unwrap();
    machine
        .allocate(Thunk::Ready(Value::Closure {
            binder: 0,
            body: term(Term::Var {
                id: 1,
                name: "next".into(),
            }),
            environment,
        }))
        .unwrap()
}

fn request(
    machine: &mut Machine<'_>,
    name: &str,
    arguments: Vec<ThunkId>,
    next: ThunkId,
) -> ThunkId {
    let continuation = constant(machine, next);
    machine
        .allocate(Thunk::Ready(Value::Request {
            name: name.into(),
            arguments,
            continuation,
        }))
        .unwrap()
}

fn print(machine: &mut Machine<'_>, text: &str, next: ThunkId) -> ThunkId {
    let text = literal(machine, &format!("{text:?}"));
    request(machine, "IO.print", vec![text], next)
}

fn sleep(machine: &mut Machine<'_>, milliseconds: u32, next: ThunkId) -> ThunkId {
    let delay = literal(machine, &milliseconds.to_string());
    request(machine, "IO.sleep", vec![delay], next)
}

fn spawn(machine: &mut Machine<'_>, child: ThunkId, next: ThunkId) -> ThunkId {
    let erased = machine.allocate(Thunk::Ready(Value::Erased)).unwrap();
    let action = constant(machine, child);
    let action = constant(machine, action);
    request(machine, "IO.spawn", vec![erased, action], next)
}

fn timed_pair(machine: &mut Machine<'_>) -> ThunkId {
    let end = literal(machine, "Emit{Unit{}}");
    let a = print(machine, "a", end);
    let a = sleep(machine, 30, a);
    let b = print(machine, "b", end);
    let b = sleep(machine, 10, b);
    let next = spawn(machine, b, end);
    spawn(machine, a, next)
}

#[test]
fn now_driver_preserves_full_clock_origin_and_floors_submilliseconds_across_runs() {
    let program = program_with_nat();
    let origin_millis = (1_u64 << 40) + 123;
    let mut clock = TestClock::default();
    for (milliseconds, fraction) in [(origin_millis, 999_999), (origin_millis + 37, 500_000)] {
        clock.time.set(milliseconds * NANOS_PER_MILLI + fraction);
        let mut machine = Machine::new(&program);
        machine.gc_mode = gc::Mode::EverySafePoint;
        // The returned clock value goes through the real request continuation,
        // checked Base Nat.show, and console driver; no direct nat() injection.
        let continuation = literal(
            &mut machine,
            "n => IO.print(Nat.show(n))(Unit, x => Emit{x})",
        );
        let first = machine
            .allocate(Thunk::Ready(Value::Request {
                name: "IO.now".into(),
                arguments: vec![],
                continuation,
            }))
            .unwrap();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        assert_eq!(
            machine
                .drive_io_with_clock(first, &mut stdout, &mut stderr, &mut clock)
                .unwrap(),
            0
        );
        assert_eq!(stdout, format!("{milliseconds}\n").as_bytes());
        assert!(stderr.is_empty());
        assert!(machine.gc.collections > 0);
    }
    assert!(clock.waits.is_empty());
}

#[test]
fn buffered_stdout_is_flushed_before_the_host_wait() {
    type Events = Rc<RefCell<Vec<&'static str>>>;
    struct Writer(Events);
    impl Write for Writer {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.borrow_mut().push("write");
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.0.borrow_mut().push("flush");
            Ok(())
        }
    }
    struct FlushedClock {
        events: Events,
        clock: TestClock,
    }
    impl Clock for FlushedClock {
        fn now(&mut self) -> Result<u64, KernelError> {
            self.clock.now()
        }
        fn wait(&mut self, nanoseconds: u64) -> Result<(), KernelError> {
            self.events.borrow_mut().push("wait");
            self.clock.wait(nanoseconds)
        }
    }
    let program = program();
    let mut machine = Machine::new(&program);
    let end = literal(&mut machine, "Emit{Unit{}}");
    let parked = sleep(&mut machine, 1, end);
    let first = print(&mut machine, "before", parked);
    let events = Events::default();
    let mut stdout = Writer(Rc::clone(&events));
    let mut stderr = Vec::new();
    let mut clock = FlushedClock {
        events: Rc::clone(&events),
        clock: TestClock::default(),
    };
    assert_eq!(
        machine
            .drive_io_with_clock(first, &mut stdout, &mut stderr, &mut clock)
            .unwrap(),
        0
    );
    let events = events.borrow();
    let wait = events.iter().position(|event| *event == "wait").unwrap();
    let written = events.iter().rposition(|event| *event == "write").unwrap();
    assert!(written < wait);
    assert!(events[written + 1..wait].contains(&"flush"));
    assert!(stderr.is_empty());
}

#[test]
fn clock_driver_waits_for_earliest_timer_and_keeps_collected_continuations() {
    let program = program();
    let mut machine = Machine::new(&program);
    machine.gc_mode = gc::Mode::EverySafePoint;
    let first = timed_pair(&mut machine);
    let mut clock = TestClock::default();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        machine
            .drive_io_with_clock(first, &mut stdout, &mut stderr, &mut clock)
            .unwrap(),
        0
    );
    assert_eq!(stdout, b"b\na\n");
    assert!(stderr.is_empty());
    assert_eq!(clock.waits, [10 * NANOS_PER_MILLI, 20 * NANOS_PER_MILLI]);
    assert!(machine.gc.collections > 5);
    assert!(machine.scheduler.finished());
}

#[test]
fn clock_oversleep_promotes_overdue_timers_in_registration_order() {
    let program = program();
    let mut machine = Machine::new(&program);
    machine.gc_mode = gc::Mode::EverySafePoint;
    let first = timed_pair(&mut machine);
    let mut clock = TestClock {
        oversleep: 50 * NANOS_PER_MILLI,
        ..TestClock::default()
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        machine
            .drive_io_with_clock(first, &mut stdout, &mut stderr, &mut clock)
            .unwrap(),
        0
    );
    assert_eq!(stdout, b"a\nb\n");
    assert!(stderr.is_empty());
    assert_eq!(clock.waits, [10 * NANOS_PER_MILLI]);
    assert!(machine.gc.collections > 5);
}

#[test]
fn runnable_work_precedes_already_overdue_timers_under_forced_collection() {
    struct BusyWriter {
        time: Rc<Cell<u64>>,
        output: Vec<u8>,
    }
    impl Write for BusyWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.output.extend_from_slice(buf);
            // The runnable computation consumes enough CPU to make both
            // parked timers overdue without voluntarily yielding.
            self.time.set(50 * NANOS_PER_MILLI);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let program = program();
    let mut machine = Machine::new(&program);
    machine.gc_mode = gc::Mode::EverySafePoint;
    let end = literal(&mut machine, "Emit{Unit{}}");
    let a = print(&mut machine, "a", end);
    let a = sleep(&mut machine, 30, a);
    let b = print(&mut machine, "b", end);
    let b = sleep(&mut machine, 10, b);
    let busy = print(&mut machine, "busy", end);
    let next = spawn(&mut machine, busy, end);
    let next = spawn(&mut machine, b, next);
    let first = spawn(&mut machine, a, next);
    let mut clock = TestClock::default();
    let mut stdout = BusyWriter {
        time: Rc::clone(&clock.time),
        output: Vec::new(),
    };
    let mut stderr = Vec::new();
    assert_eq!(
        machine
            .drive_io_with_clock(first, &mut stdout, &mut stderr, &mut clock)
            .unwrap(),
        0
    );
    assert_eq!(stdout.output, b"busy\na\nb\n");
    assert!(stderr.is_empty());
    assert!(clock.waits.is_empty());
    assert!(machine.gc.collections > 5);
}

#[test]
fn long_waits_are_bounded_and_cancellation_drops_parked_continuations() {
    let program = program();
    let mut machine = Machine::new(&program);
    let end = literal(&mut machine, "Emit{Unit{}}");
    let after = print(&mut machine, "AFTER", end);
    let first = sleep(&mut machine, u32::MAX, after);
    let cancelled = Rc::new(Cell::new(false));
    let cancellation = || cancelled.get();
    machine.cancelled = Some(&cancellation);
    let mut clock = TestClock {
        cancel_on_wait: Some(Rc::clone(&cancelled)),
        ..TestClock::default()
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let error = machine
        .drive_io_with_clock(first, &mut stdout, &mut stderr, &mut clock)
        .unwrap_err();
    assert!(error.to_string().contains("cancelled"), "{error}");
    assert!(stdout.is_empty() && stderr.is_empty());
    assert_eq!(clock.waits, [MAX_WAIT_NANOS]);
    assert!(machine.scheduler.finished());
}

#[test]
fn idle_polling_does_not_consume_the_programs_evaluation_budget() {
    let program = program();
    let mut machine = Machine::new(&program);
    let end = literal(&mut machine, "Emit{Unit{}}");
    let first = sleep(&mut machine, 1_000_000, end);
    machine.remaining = 2_000;
    let mut clock = TestClock::default();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        machine
            .drive_io_with_clock(first, &mut stdout, &mut stderr, &mut clock)
            .unwrap(),
        0
    );
    assert!(stdout.is_empty() && stderr.is_empty());
    assert!(clock.waits.len() > 2_000);
    assert!(clock.waits.iter().all(|wait| *wait <= MAX_WAIT_NANOS));
    assert_eq!(clock.waits.iter().sum::<u64>(), 1_000_000 * NANOS_PER_MILLI);
    assert!(machine.remaining > 0);
}

#[test]
fn timer_deadline_overflow_fails_before_the_continuation_or_wait() {
    let program = program();
    let mut machine = Machine::new(&program);
    let end = literal(&mut machine, "Emit{Unit{}}");
    let after = print(&mut machine, "AFTER", end);
    let first = sleep(&mut machine, 1, after);
    let mut clock = TestClock::default();
    clock.time.set(u64::MAX);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let error = machine
        .drive_io_with_clock(first, &mut stdout, &mut stderr, &mut clock)
        .unwrap_err();
    assert!(error.to_string().contains("deadline overflow"), "{error}");
    assert!(stdout.is_empty() && stderr.is_empty());
    assert!(clock.waits.is_empty());
    assert!(machine.scheduler.finished());
}

#[test]
fn failed_host_wait_drops_all_pending_tasks_without_resuming_them() {
    let program = program();
    let mut machine = Machine::new(&program);
    let first = timed_pair(&mut machine);
    let mut clock = TestClock {
        fail_wait: true,
        ..TestClock::default()
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let error = machine
        .drive_io_with_clock(first, &mut stdout, &mut stderr, &mut clock)
        .unwrap_err();
    assert!(
        error.to_string().contains("test clock wait failure"),
        "{error}"
    );
    assert!(stdout.is_empty() && stderr.is_empty());
    assert!(machine.scheduler.finished());
}
