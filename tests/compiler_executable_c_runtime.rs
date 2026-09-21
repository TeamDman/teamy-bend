// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::fs;
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

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-c-runtime-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn run(&self, bend: &str, foreign: &str) -> Output {
        self.run_transformed(bend, foreign, str::to_owned)
    }

    fn run_transformed(
        &self,
        bend: &str,
        foreign: &str,
        transform: impl FnOnce(&str) -> String,
    ) -> Output {
        fs::write(self.0.join("main.bend"), bend).unwrap();
        fs::write(self.0.join("effect.c"), foreign).unwrap();
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        let generated = compile_executable_c(&checked).unwrap();
        let generated = transform(&generated);
        let executable = executable_c_compiler::compile(&self.0, &generated, &[]);
        executable_c_compiler::bounded(
            Command::new(executable).current_dir(&self.0),
            Duration::from_secs(20),
        )
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

const WORKER_BEND: &str = r#"import Base
def background() -> IO(U32): import "effect.c"
def main() -> IO(Unit):
  do IO<Unit>:
    answer : U32 <- background()
    IO.print(U32.show(answer))
"#;

const WORKER_C: &str = r#"
#ifdef _WIN32
static DWORD vm_thread;
static bool on_vm_thread(void) { return GetCurrentThreadId() == vm_thread; }
#else
static pthread_t vm_thread;
static bool on_vm_thread(void) { return pthread_equal(pthread_self(), vm_thread) != 0; }
#endif
static void background_call(IoWork *work) {
  if (on_vm_thread()) err_fail("worker ran on the VM thread");
  work->data[0] = 41;
}
static Term background_resume(Env e, IoWork *work) {
  Term answer; (void)e;
  if (!on_vm_thread()) err_fail("packing ran outside the VM thread");
  answer = (unsigned char)work->data[0] + 1;
  free(work->data); work->data = NULL;
  return answer;
}
static Term background_pack(Env e, IoWork *work) {
  IoAct *action = (IoAct *)work; (void)e;
  if (!on_vm_thread()) err_fail("packing ran outside the VM thread");
  (void)io_wait_on(work, 0, POLLIN, background_resume);
  action->time = 1;
  return IO_PARK;
}
static Term background_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields;
#ifdef _WIN32
  vm_thread = GetCurrentThreadId();
#else
  vm_thread = pthread_self();
#endif
  work->data = io_mem(malloc(16));
  return io_work(work, background_call, background_pack);
}
static void __attribute__((constructor)) background_use(void) {
  io_eff(CID_BACKGROUND, background_run, 0);
}
"#;

#[test]
fn worker_syscalls_run_off_vm_and_packing_can_repark_on_vm() {
    success(&Fixture::new().run(WORKER_BEND, WORKER_C), "42\n");
}

const SNAPSHOT_BEND: &str = r#"import Base
def background() -> IO(Unit): import "effect.c"
def child() -> IO(Unit):
  do IO<Unit>:
    Unit <- background()
    IO.print("worker")
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, child())
    Unit <- IO.sleep(0)
    IO.print("timer")
"#;

const SNAPSHOT_C: &str = r#"
static TBMutex snapshot_gate = TB_MUTEX_INIT;
static bool snapshot_release;
static bool snapshot_observed;
static void snapshot_pause(void) {
#ifdef _WIN32
  Sleep(1);
#else
  struct timespec duration = {0, 1000000};
  (void)nanosleep(&duration, NULL);
#endif
}
static void background_call(IoWork *work) {
  (void)work;
  for (unsigned attempt = 0; attempt < 5000; ++attempt) {
    bool ready;
    tb_lock(&snapshot_gate); ready = snapshot_release; tb_unlock(&snapshot_gate);
    if (ready) return;
    snapshot_pause();
  }
  err_fail("poll snapshot did not release its worker");
}
static Term background_pack(Env e, IoWork *work) {
  (void)e; (void)work; return term_pak(CID_UNIT, 0);
}
static Term background_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields; return io_work(work, background_call, background_pack);
}
#ifdef _WIN32
static int tb_test_poll(WSAPOLLFD *rows, ULONG count, int timeout) {
  int result = WSAPoll(rows, count, timeout);
#else
static int tb_test_poll(struct pollfd *rows, nfds_t count, int timeout) {
  int result = poll(rows, count, timeout);
#endif
  if (!snapshot_observed) {
    snapshot_observed = true;
    if (result != 0) err_fail("worker was ready before the poll snapshot");
    for (size_t index = 0; index < (size_t)count; ++index)
      if (rows[index].revents != 0) err_fail("unexpected poll snapshot readiness");
    tb_lock(&snapshot_gate); snapshot_release = true; tb_unlock(&snapshot_gate);
    for (unsigned attempt = 0; attempt < 5000; ++attempt) {
      bool done;
      tb_lock(&tb_io->host->mutex); done = tb_io->active == 0; tb_unlock(&tb_io->host->mutex);
      if (done) return result;
      snapshot_pause();
    }
    err_fail("worker did not complete after the poll snapshot");
  }
  return result;
}
static void __attribute__((constructor)) background_use(void) {
  io_eff(CID_BACKGROUND, background_run, 0);
}
"#;

fn snapshot_poll_seam(source: &str) -> String {
    let prototype = "#ifdef _WIN32\n\
        static int tb_test_poll(WSAPOLLFD *, ULONG, int);\n\
        #else\nstatic int tb_test_poll(struct pollfd *, nfds_t, int);\n#endif\n\
        OUTLINE void io_wait(Env e) {";
    assert_eq!(source.matches("OUTLINE void io_wait(Env e) {").count(), 1);
    let source = source.replacen("OUTLINE void io_wait(Env e) {", prototype, 1);
    let original = if cfg!(windows) {
        "WSAPoll(polls, poll_count, timeout)"
    } else {
        "poll(polls, poll_count, timeout)"
    };
    assert_eq!(source.matches(original).count(), 1);
    source.replacen(original, "tb_test_poll(polls, poll_count, timeout)", 1)
}

#[test]
fn workers_completing_after_the_poll_snapshot_wait_for_a_subsequent_wake() {
    // The checked Bend program, worker thread, wake socket and poll are real.
    // Only the poll call is wrapped: after the OS returns a timer-only snapshot,
    // it releases the worker and waits for completion before returning that
    // unchanged snapshot to io_wait. This makes the boundary deterministic.
    success(
        &Fixture::new().run_transformed(SNAPSHOT_BEND, SNAPSHOT_C, snapshot_poll_seam),
        "timer\nworker\n",
    );
    // Verify the fixture distinguishes the old policy that inspected all
    // completed workers, even when the snapshot contained no worker wake.
    success(
        &Fixture::new().run_transformed(SNAPSHOT_BEND, SNAPSHOT_C, |source| {
            let source = snapshot_poll_seam(source);
            let original = "if (worker_ready) io_take(e);";
            assert_eq!(source.matches(original).count(), 1);
            source.replacen(
                original,
                "(void)worker_ready; if (io_busy != 0) io_take(e);",
                1,
            )
        }),
        "worker\ntimer\n",
    );
}

const NOTIFICATION_C: &str = r#"
static unsigned notification_attempts;
static intptr_t tb_test_notification_send(intptr_t descriptor, const char *data,
    u32 size, const struct sockaddr_in *address) {
  if (notification_attempts++ == 0) {
#ifdef _WIN32
    WSASetLastError(TEST_INTERRUPTED ? WSAEINTR : WSAENOBUFS);
#else
    errno = TEST_INTERRUPTED ? EINTR : EIO;
#endif
    return -1;
  }
  return tb_net_send_system(descriptor, data, size, address);
}
static void background_call(IoWork *work) { (void)work; }
static Term background_pack(Env e, IoWork *work) {
  (void)e; (void)work;
  if (notification_attempts != 2) err_fail("interrupted notification was not retried");
  return 42;
}
static Term background_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields; return io_work(work, background_call, background_pack);
}
static void __attribute__((constructor)) background_use(void) {
  io_eff(CID_BACKGROUND, background_run, 0);
}
"#;

fn notification_send_seam(source: &str) -> String {
    let declaration = "static void tb_network_notify_locked(TBHost *host) {";
    let prototype = "static intptr_t tb_test_notification_send(intptr_t, const char *, \
        u32, const struct sockaddr_in *);\n";
    assert_eq!(source.matches(declaration).count(), 1);
    let source = source.replacen(declaration, &format!("{prototype}{declaration}"), 1);
    let call = "tb_net_send_system(host->wake_write, \"w\", 1, NULL)";
    assert_eq!(source.matches(call).count(), 1);
    source.replacen(
        call,
        "tb_test_notification_send(host->wake_write, \"w\", 1, NULL)",
        1,
    )
}

#[test]
fn notification_send_retries_interruption_and_other_failures_stop_the_invocation() {
    // Inject only the first notifier send result. The checked program, real
    // worker, completion queue and descriptor poll remain production code.
    for interrupted in [false, true] {
        let foreign = format!(
            "#define TEST_INTERRUPTED {}\n{NOTIFICATION_C}",
            u8::from(interrupted)
        );
        let output = Fixture::new().run_transformed(WORKER_BEND, &foreign, notification_send_seam);
        if interrupted {
            success(&output, "42\n");
        } else {
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert_eq!(
                output.stderr,
                b"teamy-bend executable C: worker notification send failed\n"
            );
        }
    }
}

const COLLECTION_C: &str = r#"
static TBMutex release_gate = TB_MUTEX_INIT;
static bool released;
static void pause_briefly(void) {
#ifdef _WIN32
  Sleep(1);
#else
  struct timespec duration = {0, 1000000};
  (void)nanosleep(&duration, NULL);
#endif
}
static void work_call(IoWork *work) {
  (void)work;
  for (unsigned attempt = 0; attempt < 5000; ++attempt) {
    bool ready;
    tb_lock(&release_gate); ready = released; tb_unlock(&release_gate);
    if (ready) return;
    pause_briefly();
  }
  err_fail("worker release deadline exceeded");
}
static Term work_pack(Env e, IoWork *work) {
  (void)e; (void)work; return term_pak(CID_UNIT, 0);
}
static Term work_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields; return io_work(work, work_call, work_pack);
}
static Term release_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields; (void)work;
  tb_lock(&release_gate); released = true; tb_unlock(&release_gate);
  for (unsigned attempt = 0; attempt < 5000; ++attempt) {
    bool done;
    tb_lock(&tb_io->host->mutex); done = tb_io->active == 0; tb_unlock(&tb_io->host->mutex);
    if (done) return term_pak(CID_UNIT, 0);
    pause_briefly();
  }
  err_fail("worker completion deadline exceeded");
}
static void __attribute__((constructor)) collection_use(void) {
  io_eff(CID_WORK, work_run, 0);
  io_eff(CID_RELEASE, release_run, 0);
}
"#;

#[test]
fn c_worker_collection_waits_for_the_current_activation_to_yield() {
    let mut source = String::from(
        r#"import Base
def work() -> IO(Unit): import "effect.c"
def release() -> IO(Unit): import "effect.c"
def background() -> IO(Unit):
  do IO<Unit>:
    Unit <- work()
    IO.print("worker")
def burst() -> IO(Unit):
  do IO<Unit>:
"#,
    );
    for _ in 0..19 {
        source.push_str("    Unit <- IO.spawn(Unit, IO.print(\"child\"))\n");
    }
    source.push_str(
        r#"    IO.spawn(Unit, IO.print("child"))
def producer() -> IO(Unit):
  do IO<Unit>:
    Unit <- release()
    Unit <- burst()
    Unit <- burst()
    Unit <- burst()
    Unit <- burst()
    IO.print("producer")
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, background())
    IO.spawn(Unit, producer())
"#,
    );
    let expected = format!("producer\n{}worker\n", "child\n".repeat(80));
    success(&Fixture::new().run(&source, COLLECTION_C), &expected);
}

#[test]
fn malformed_native_registrations_fail_before_running_the_program() {
    let source = r#"import Base
def request() -> IO(Unit): import "effect.c"
def main() -> IO(Unit): request()
"#;
    let handlers = r"
static Term first_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields; (void)work; return term_pak(CID_UNIT, 0);
}
static Term second_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields; (void)work; return term_pak(CID_UNIT, 0);
}
";
    for (registration, diagnostic) in [
        ("", "unregistered foreign effect"),
        (
            "io_eff(BEND_CID_COUNT, first_run, 0);",
            "invalid foreign effect registration",
        ),
        (
            "io_eff(CID_REQUEST, first_run, 4);",
            "invalid foreign effect registration",
        ),
        (
            "io_eff(CID_REQUEST, first_run, 0); io_eff(CID_REQUEST, second_run, 0);",
            "duplicate foreign effect registration",
        ),
    ] {
        let foreign = format!(
            "{handlers}\nstatic void __attribute__((constructor)) register_request(void) {{\n\
             (void)first_run; (void)second_run; {registration}\n}}\n"
        );
        let output = Fixture::new().run(source, &foreign);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8_lossy(&output.stderr),
            format!("teamy-bend executable C: {diagnostic}\n")
        );
    }
}

const ORDER_BEND: &str = r#"import Base
def waiting(value: U32) -> IO(U32): import "effect.c"
def child(value: U32) -> IO(Unit):
  do IO<Unit>:
    answer : U32 <- waiting(value)
    IO.print(U32.show(answer))
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, child(1))
    Unit <- IO.spawn(Unit, child(2))
    Unit <- IO.spawn(Unit, child(3))
    IO.sleep(0)
"#;

const ORDER_C: &str = r#"
static IoAct *old_wait;
static Term waiting_more(Env e, IoWork *work) {
  IoAct *action = (IoAct *)work; (void)e;
  if (work->hand != 42) err_fail("readiness wait changed work.hand");
  if (work->made == 2) return 2;
  if (work->size++ == 0) {
    (void)io_wait_on(work, 0, POLLIN, waiting_more);
    action->time = 1;
    return IO_PARK;
  }
  old_wait->time = 1;
  return (Term)work->made;
}
static Term waiting_run(Env e, Term *fields, IoWork *work) {
  IoAct *action = (IoAct *)work; (void)e;
  work->made = (intptr_t)fields[0]; work->hand = 42;
  (void)io_wait_on(work, 0, POLLIN, waiting_more);
  action->time = fields[0] == 2 ? UINT64_MAX : 1;
  if (fields[0] == 2) old_wait = action;
  return IO_PARK;
}
static void __attribute__((constructor)) waiting_use(void) {
  io_eff(CID_WAITING, waiting_run, 0);
}
"#;

#[test]
fn c_callbacks_repark_during_the_original_queue_walk() {
    // Upstream C invokes each due callback as it visits the parked queue. The
    // first callback reparks before the still-pending second wait, so its next
    // wake can make that second wait ready during the same queue walk. The JS
    // driver instead partitions every due wait before invoking callbacks.
    success(&Fixture::new().run(ORDER_BEND, ORDER_C), "1\n2\n3\n");
}

const HALT_BEND: &str = r#"import Base
def blocked() -> IO(Unit): import "effect.c"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, blocked())
    Unit <- IO.sleep(0)
    IO.die(Unit, 7, "halt")
"#;

const HALT_C: &str = r#"
static TBMutex finished_gate = TB_MUTEX_INIT;
static bool finished;
static void pause_briefly(void) {
#ifdef _WIN32
  Sleep(1);
#else
  struct timespec duration = {0, 1000000};
  (void)nanosleep(&duration, NULL);
#endif
}
static void blocked_call(IoWork *work) {
  bool stopped = false;
  while (!stopped) {
    tb_lock(&tb_host_current->mutex);
    stopped = tb_host_current->stopped;
    tb_unlock(&tb_host_current->mutex);
    if (!stopped) pause_briefly();
  }
  if (memcmp(work->data, "owned after Halt", 16) != 0) abort();
  free(work->data); work->data = NULL;
  tb_lock(&finished_gate); finished = true; tb_unlock(&finished_gate);
}
static Term blocked_pack(Env e, IoWork *work) {
  (void)e; (void)work;
  err_fail("cancelled work was packed into a released VM");
}
static Term blocked_run(Env e, Term *fields, IoWork *work) {
  (void)e; (void)fields;
  work->data = io_mem(malloc(16));
  memcpy(work->data, "owned after Halt", 16);
  return io_work(work, blocked_call, blocked_pack);
}
static void observe_worker_after_halt(void) {
  for (unsigned attempt = 0; attempt < 5000; ++attempt) {
    bool done;
    tb_lock(&finished_gate); done = finished; tb_unlock(&finished_gate);
    if (done) return;
    pause_briefly();
  }
  abort();
}
static void __attribute__((constructor)) blocked_use(void) {
  io_eff(CID_BLOCKED, blocked_run, 0);
  if (atexit(observe_worker_after_halt) != 0) err_fail("atexit registration failed");
}
"#;

#[test]
fn halt_keeps_active_worker_buffers_alive_and_never_packs_into_the_released_vm() {
    let output = Fixture::new().run(HALT_BEND, HALT_C);
    assert_eq!(output.status.code(), Some(7));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"halt\n");
}

#[test]
fn runnable_tasks_precede_zero_timers_and_children_outlive_the_parent() {
    let source = r#"import Base
def late(text: String) -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.sleep(0)
    Unit <- IO.sleep(0)
    IO.print(text)
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, late("a"))
    Unit <- IO.spawn(Unit, late("b"))
    Unit <- IO.spawn(Unit, IO.print("ready"))
    IO.print("main")
"#;
    success(&Fixture::new().run(source, ""), "main\nready\na\nb\n");
}

#[test]
fn buffered_channels_drain_after_close_and_old_copies_remain_closed() {
    let source = r#"import Base
def maybe(value: Maybe<&1, U32>) -> String:
  match value:
    case None{}: "none"
    case Some{x}: U32.show(x)
def flag(value: Bool) -> String:
  match value:
    case False{}: "false"
    case True{}: "true"
def use(channel: Chan(U32)) -> IO(Unit):
  +saved = channel
  do IO<Unit>:
    ok : Bool <- Chan.send(U32, saved, 1)
    ok : Bool <- Chan.send(U32, saved, 2)
    Unit <- Chan.close(U32, saved)
    ok : Bool <- Chan.send(U32, saved, 3)
    Unit <- IO.print(flag(ok))
    one : Maybe<&1, U32> <- Chan.recv(U32, saved)
    Unit <- IO.print(maybe(one))
    two : Maybe<&1, U32> <- Chan.recv(U32, saved)
    Unit <- IO.print(maybe(two))
    empty : Maybe<&1, U32> <- Chan.recv(U32, saved)
    Unit <- IO.print(maybe(empty))
    Chan.close(U32, saved)
def main() -> IO(Unit):
  IO.bind(Chan(U32), Unit, Chan.new(U32, 2), use)
"#;
    success(&Fixture::new().run(source, ""), "false\n1\n2\nnone\n");
}
