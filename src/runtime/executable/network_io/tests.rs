// SPDX-License-Identifier: MPL-2.0
//! Private injected-request regressions for VM ownership and scheduling. Public
//! checked-source and upstream ABI comparisons live in the integration suite.

use super::*;
use crate::kernel::Term;
use crate::kernel::term;
use crate::runtime::Program;
use crate::runtime::gc;
use crate::runtime::host_jobs;
use crate::runtime::runtime_clock::Clock;
use crate::syntax::executable::ForeignDefinition;
use crate::syntax::parse_term;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Write;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;

fn program() -> Program {
    let foreign = [
        ("IO.print", BuiltinForeign::Print, 1),
        ("IO.write", BuiltinForeign::Write, 1),
        ("IO.spawn", BuiltinForeign::Spawn, 2),
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
                name: "captured".into(),
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

fn print(machine: &mut Machine<'_>, label: &str, next: ThunkId) -> ThunkId {
    // Use the same compact host encoding as a network reply. Parsing raw
    // literals without Base layouts would exercise bit-tree expansion here.
    let text = machine
        .host_text(&label.chars().map(u32::from).collect::<Vec<_>>())
        .unwrap();
    request(machine, "IO.print", vec![text], next)
}

fn suspend(machine: &mut Machine<'_>, action: ThunkId) {
    machine.scheduler.spawn(action).unwrap();
    assert_eq!(machine.scheduler.next(), Some(action));
}

fn park_receive(machine: &mut Machine<'_>, socket: NativeSocket, next: ThunkId) {
    let handle = machine.network.insert(Resource::Socket(socket)).unwrap();
    let resource = machine.network.take(handle).unwrap();
    let continuation = constant(machine, next);
    suspend(machine, next);
    assert!(
        machine
            .park_network(Request {
                resource,
                handle: Some(handle),
                continuation,
                operation: Operation::Recv { max: 32 },
            })
            .unwrap()
            .is_none()
    );
}

fn socket_ready(socket: &NativeSocket, interest: host_network::Interest) {
    assert_eq!(
        host_network::poll(
            &[host_network::Registration {
                source: host_network::Source::Socket(socket),
                interest,
            }],
            2_000
        )
        .unwrap(),
        [true]
    );
}

fn connected() -> (NativeSocket, NativeSocket) {
    let listener = host_network::listen(0).unwrap();
    listener.set_recv_buffer_size(1024).unwrap();
    let connection = host_network::connect(
        b"127.0.0.1",
        u32::from(listener.local_addr().unwrap().port()),
    )
    .unwrap();
    if connection.pending {
        socket_ready(&connection.socket, host_network::Interest::Write);
        host_network::finish_connect(&connection.socket).unwrap();
    }
    assert_eq!(
        host_network::poll(
            &[host_network::Registration {
                source: host_network::Source::Listener(&listener),
                interest: host_network::Interest::Read,
            }],
            2_000
        )
        .unwrap(),
        [true]
    );
    let Step::Ready(peer) = host_network::accept(&listener).unwrap() else {
        panic!("ready loopback listener did not accept")
    };
    (connection.socket, peer)
}

fn assert_clean(machine: &Machine<'_>) {
    assert!(machine.network.is_empty());
    assert!(machine.files.is_empty());
    assert!(!machine.jobs.has_pending());
    assert!(machine.scheduler.finished());
}

fn fill_send_window(socket: &NativeSocket) -> usize {
    let bytes = vec![b'x'; 64 * 1024];
    let mut sent = 0;
    // Stop only after real WouldBlock remains non-writable for a bounded poll.
    // The cap prevents a broken fixture from becoming an unbounded host loop.
    for _ in 0..256 {
        match host_network::send(socket, &bytes).unwrap() {
            Step::Ready(count) => {
                assert!(count > 0);
                sent += count;
            }
            Step::Wait => {
                let ready = host_network::poll(
                    &[host_network::Registration {
                        source: host_network::Source::Socket(socket),
                        interest: host_network::Interest::Write,
                    }],
                    20,
                )
                .unwrap();
                if ready == [false] {
                    return sent;
                }
            }
        }
    }
    panic!("bounded loopback prefill did not reach persistent OS backpressure");
}

fn drain_to_eof(socket: &NativeSocket, limit: usize) -> Vec<u8> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut received = Vec::new();
    loop {
        assert!(Instant::now() < deadline, "bounded peer receive deadline");
        match host_network::recv(socket, 64 * 1024, 64 * 1024).unwrap() {
            Step::Ready(bytes) if bytes.is_empty() => return received,
            Step::Ready(bytes) => {
                assert!(
                    received.len() + bytes.len() <= limit,
                    "send duplicated bytes"
                );
                received.extend(bytes);
            }
            Step::Wait => {
                let _ready = host_network::poll(
                    &[host_network::Registration {
                        source: host_network::Source::Socket(socket),
                        interest: host_network::Interest::Read,
                    }],
                    250,
                )
                .unwrap();
            }
        }
    }
}

fn missing_reply() -> host_jobs::Reply {
    host_jobs::Reply::Open(Err(Failure::Io(host_files::Error {
        code: 2,
        message: b"missing".to_vec(),
    })))
}

fn worker(machine: &mut Machine<'_>, next: ThunkId, task: host_jobs::Task) {
    let continuation = constant(machine, next);
    suspend(machine, next);
    machine.jobs.submit(task, continuation, None, 0).unwrap();
}

struct FrozenClock;

impl Clock for FrozenClock {
    fn now(&mut self) -> Result<u64, KernelError> {
        Ok(3)
    }
    fn wait(&mut self, _nanoseconds: u64) -> Result<(), KernelError> {
        Err(KernelError::new("all test waits should already be ready"))
    }
}

#[test]
fn mixed_due_timers_and_network_follow_registration_order_and_keep_closure_roots() {
    let program = program();
    let mut machine = Machine::new(&program);
    machine.gc_mode = gc::Mode::EverySafePoint;
    let mut peers = Vec::new();
    for (index, deadline) in [2, 1, 0].into_iter().enumerate() {
        let end = literal(&mut machine, "Emit{Unit{}}");
        let timer = print(&mut machine, &format!("timer{index}"), end);
        suspend(&mut machine, timer);
        machine.scheduler.sleep(deadline, timer).unwrap();
        if index < 2 {
            let (socket, peer) = connected();
            assert_eq!(host_network::send(&peer, b"x").unwrap(), Step::Ready(1));
            socket_ready(&socket, host_network::Interest::Read);
            let end = literal(&mut machine, "Emit{Unit{}}");
            let next = print(&mut machine, &format!("network{index}"), end);
            park_receive(&mut machine, socket, next);
            peers.push(peer);
        }
    }
    let end = literal(&mut machine, "Emit{Unit{}}");
    let ready = print(&mut machine, "ready", end);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        machine
            .drive_io_with_clock(ready, &mut stdout, &mut stderr, &mut FrozenClock)
            .unwrap(),
        0
    );
    assert_eq!(
        stdout,
        b"ready\ntimer0\nnetwork0\ntimer1\nnetwork1\ntimer2\n"
    );
    assert!(stderr.is_empty());
    assert!(machine.gc.collections > 20);
    assert_clean(&machine);
    drop(peers);
}

struct CancelWriter<'a>(&'a Cell<bool>);

#[test]
fn stale_readiness_reparks_the_same_owned_socket_and_captured_continuation() {
    let program = program();
    let mut machine = Machine::new(&program);
    machine.gc_mode = gc::Mode::EverySafePoint;
    let (socket, peer) = connected();
    let end = literal(&mut machine, "Emit{Unit{}}");
    let captured = print(&mut machine, "captured after retry", end);
    park_receive(&mut machine, socket, captured);
    assert!(machine.network.poll(None, 0).unwrap().is_empty());
    // A readable indication can race with host state. The retry must retain
    // both ownership and the closure environment when recv still would block.
    machine.resume_network(&[0], 0).unwrap();
    assert!(!machine.scheduler.has_ready());
    let garbage = literal(&mut machine, "Unit{}");
    machine.force(garbage).unwrap();
    assert_eq!(host_network::send(&peer, b"x").unwrap(), Step::Ready(1));
    assert_eq!(machine.network.poll(None, 2_000).unwrap(), [1]);
    let start = literal(&mut machine, "Emit{Unit{}}");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        machine
            .drive_io_with_clock(start, &mut stdout, &mut stderr, &mut FrozenClock)
            .unwrap(),
        0
    );
    assert_eq!(stdout, b"captured after retry\n");
    assert!(stderr.is_empty());
    assert!(machine.gc.collections > 5);
    assert_clean(&machine);
}

impl Write for CancelWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        assert_eq!(buf, b"cancel");
        self.0.set(true);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn halt_and_cancellation_close_pending_and_live_network_resources() {
    let program = program();
    for cancel_run in [false, true] {
        let cancelled = Cell::new(false);
        let cancel = || cancelled.get();
        let mut machine = Machine::new(&program);
        machine.cancelled = Some(&cancel);
        machine.gc_mode = gc::Mode::EverySafePoint;
        let (socket, peer) = connected();
        let end = literal(&mut machine, "Emit{Unit{}}");
        let pending = print(&mut machine, "must not run", end);
        park_receive(&mut machine, socket, pending);
        machine
            .network
            .insert(Resource::Listener(host_network::listen(0).unwrap()))
            .unwrap();
        machine
            .network
            .insert(Resource::Socket(host_network::bind(0).unwrap()))
            .unwrap();
        let action = if cancel_run {
            let end = literal(&mut machine, "Emit{Unit{}}");
            let text = literal(&mut machine, "\"cancel\"");
            request(&mut machine, "IO.write", vec![text], end)
        } else {
            literal(&mut machine, "Halt{7, \"halt\"}")
        };
        let mut stderr = Vec::new();
        let result = machine.drive_io(action, &mut CancelWriter(&cancelled), &mut stderr);
        if cancel_run {
            assert!(result.unwrap_err().to_string().contains("cancelled"));
            assert!(stderr.is_empty());
        } else {
            assert_eq!(result.unwrap(), 7);
            assert_eq!(stderr, b"halt\n");
        }
        assert_clean(&machine);
        socket_ready(&peer, host_network::Interest::Read);
        assert_eq!(
            host_network::recv(&peer, 1, 1).unwrap(),
            Step::Ready(Vec::new())
        );
    }
}

struct WorkerWakeClock {
    release: Option<mpsc::Sender<()>>,
    waits: usize,
}

impl Clock for WorkerWakeClock {
    fn now(&mut self) -> Result<u64, KernelError> {
        Ok(0)
    }
    fn wait(&mut self, _nanoseconds: u64) -> Result<(), KernelError> {
        Err(KernelError::new(
            "worker test must use descriptor readiness",
        ))
    }
    fn wait_for_network(
        &mut self,
        network: &network::State,
        work: &host_jobs::State,
        _nanoseconds: u64,
    ) -> Result<Vec<u64>, KernelError> {
        self.waits += 1;
        let release = self
            .release
            .take()
            .expect("worker completion must wake the first poll");
        release.send(()).unwrap();
        let ready = network.poll(work.wake_source(), 2_000)?;
        assert!(
            ready.is_empty(),
            "quiet peer cannot make the network request ready"
        );
        let wake = work
            .wake_source()
            .expect("worker notifier is installed before waiting");
        assert_eq!(
            host_network::poll(
                &[host_network::Registration {
                    source: host_network::Source::Socket(wake),
                    interest: host_network::Interest::Read,
                }],
                0
            )
            .unwrap(),
            [true],
            "worker completion signalled the descriptor poller"
        );
        Ok(ready)
    }
}

#[test]
fn worker_completion_wakes_network_poller_and_halt_discards_quiet_receive() {
    let program = program();
    let mut machine = Machine::new(&program);
    machine.gc_mode = gc::Mode::EverySafePoint;
    let (socket, peer) = connected();
    let end = literal(&mut machine, "Emit{Unit{}}");
    let pending = print(&mut machine, "must not run", end);
    park_receive(&mut machine, socket, pending);
    let (release, gate) = mpsc::channel();
    let halt = literal(&mut machine, "Halt{0, \"worker\"}");
    worker(
        &mut machine,
        halt,
        host_jobs::Task::Test(Box::new(move || {
            gate.recv_timeout(Duration::from_secs(5))
                .expect("bounded worker test gate");
            missing_reply()
        })),
    );
    let start = literal(&mut machine, "Emit{Unit{}}");
    let mut clock = WorkerWakeClock {
        release: Some(release),
        waits: 0,
    };
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        machine
            .drive_io_with_clock(start, &mut stdout, &mut stderr, &mut clock)
            .unwrap(),
        0
    );
    assert_eq!(clock.waits, 1);
    assert!(stdout.is_empty());
    assert_eq!(stderr, b"worker\n");
    assert!(machine.gc.collections > 5);
    assert_clean(&machine);
    socket_ready(&peer, host_network::Interest::Read);
    assert_eq!(
        host_network::recv(&peer, 1, 1).unwrap(),
        Step::Ready(Vec::new())
    );
}

#[test]
fn seventy_backpressured_sends_leave_file_workers_available_and_halt_cleans_up() {
    const REQUESTS: usize = 70;
    const BYTES: usize = 512 * 1024;
    let program = program();
    let mut machine = Machine::new(&program);
    let mut peers = Vec::with_capacity(REQUESTS);
    for _ in 0..REQUESTS {
        let (socket, peer) = connected();
        socket.set_send_buffer_size(1024).unwrap();
        peer.set_recv_buffer_size(1024).unwrap();
        fill_send_window(&socket);
        let handle = machine.network.insert(Resource::Socket(socket)).unwrap();
        let resource = machine.network.take(handle).unwrap();
        let end = literal(&mut machine, "Emit{Unit{}}");
        let continuation = constant(&mut machine, end);
        suspend(&mut machine, end);
        assert!(
            machine
                .advance_network(Request {
                    resource,
                    handle: Some(handle),
                    continuation,
                    operation: Operation::Send {
                        data: vec![b'x'; BYTES],
                        offset: 0
                    },
                })
                .unwrap()
                .is_none(),
            "large send must encounter real OS backpressure"
        );
        peers.push(peer);
    }
    let mut roots = Vec::new();
    machine
        .network
        .visit_roots(|root| {
            roots.push(root);
            Ok(())
        })
        .unwrap();
    assert_eq!(roots.len(), REQUESTS);
    let halt = literal(&mut machine, "Halt{0, \"halt\"}");
    let canary = print(&mut machine, "worker", halt);
    worker(
        &mut machine,
        canary,
        host_jobs::Task::Test(Box::new(missing_reply)),
    );
    machine.jobs.wait(5_000_000_000).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let cancel = || Instant::now() >= deadline;
    machine.cancelled = Some(&cancel);
    machine.gc_mode = gc::Mode::EverySafePoint;
    let start = literal(&mut machine, "Emit{Unit{}}");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        machine.drive_io(start, &mut stdout, &mut stderr).unwrap(),
        0
    );
    assert_eq!(stdout, b"worker\n");
    assert_eq!(stderr, b"halt\n");
    assert!(machine.gc.collections > 5);
    assert_clean(&machine);
    drop(peers);
}

#[test]
fn backpressured_send_resumes_from_its_offset_without_duplicate_or_missing_bytes() {
    let program = program();
    let mut machine = Machine::new(&program);
    machine.gc_mode = gc::Mode::EverySafePoint;
    let (socket, peer) = connected();
    socket.set_send_buffer_size(1024).unwrap();
    peer.set_recv_buffer_size(1024).unwrap();
    let prefilled = fill_send_window(&socket);
    let marker: Vec<_> = b"abcdefghijklmnopqrstuvwxyz0123456789"
        .iter()
        .copied()
        .cycle()
        .take(512 * 1024)
        .collect();
    let mut expected = vec![b'x'; prefilled];
    expected.extend_from_slice(&marker);
    let total = expected.len();
    let handle = machine.network.insert(Resource::Socket(socket)).unwrap();
    let resource = machine.network.take(handle).unwrap();
    let end = literal(&mut machine, "Emit{Unit{}}");
    let done = print(&mut machine, "complete", end);
    let continuation = constant(&mut machine, done);
    suspend(&mut machine, done);
    assert!(
        machine
            .advance_network(Request {
                resource,
                handle: Some(handle),
                continuation,
                operation: Operation::Send {
                    data: marker,
                    offset: 0
                },
            })
            .unwrap()
            .is_none(),
        "the owned send must really park before its peer drains"
    );
    let reader = std::thread::spawn(move || drain_to_eof(&peer, total));
    let deadline = Instant::now() + Duration::from_secs(15);
    let cancel = || Instant::now() >= deadline;
    machine.cancelled = Some(&cancel);
    let start = literal(&mut machine, "Emit{Unit{}}");
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let result = machine.drive_io(start, &mut stdout, &mut stderr);
    let received = reader.join().expect("bounded loopback receiver completed");
    assert_eq!(result.unwrap(), 0);
    assert_eq!(received, expected);
    assert_eq!(stdout, b"complete\n");
    assert!(stderr.is_empty());
    assert!(machine.gc.collections > 5);
    assert_clean(&machine);
}

fn spawn_next(machine: &mut Machine<'_>, child: ThunkId) -> ThunkId {
    let end = literal(machine, "Emit{Unit{}}");
    let erased = machine.allocate(Thunk::Ready(Value::Erased)).unwrap();
    let action = constant(machine, child);
    let action = constant(machine, action);
    request(machine, "IO.spawn", vec![erased, action], end)
}

#[test]
fn completed_workers_run_before_sustained_ready_chain_drains() {
    let program = program();
    let mut machine = Machine::new(&program);
    // This fixture tests worker fairness with 130 retained future tasks. GC
    // reachability is stressed separately above without quadratic fixture work.
    let end = literal(&mut machine, "Emit{Unit{}}");
    let host = print(&mut machine, "host", end);
    worker(
        &mut machine,
        host,
        host_jobs::Task::Test(Box::new(missing_reply)),
    );
    machine.jobs.wait(5_000_000_000).unwrap();
    let end = literal(&mut machine, "Emit{Unit{}}");
    let mut ready = print(&mut machine, "last", end);
    for _ in 0..130 {
        let next = spawn_next(&mut machine, ready);
        ready = print(&mut machine, "ready", next);
    }
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        machine.drive_io(ready, &mut stdout, &mut stderr).unwrap(),
        0
    );
    let text = String::from_utf8(stdout).unwrap();
    let lines: Vec<_> = text.lines().collect();
    assert_eq!(lines.iter().filter(|line| **line == "ready").count(), 130);
    assert_eq!(lines.last(), Some(&"last"));
    let host = lines.iter().position(|line| *line == "host").unwrap();
    assert!(
        host < 65,
        "completed worker must run while ready work continues"
    );
    assert!(stderr.is_empty());
    assert_clean(&machine);
}
