// SPDX-License-Identifier: MPL-2.0
use std::cell::Cell;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;
use teamy_bend::kernel::ExecutableBook;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-native-scheduler-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("main.bend"), source).unwrap();
        Self(path)
    }

    fn checked(&self) -> ExecutableBook {
        check_executable(&load_executable(self.0.join("main.bend")).unwrap()).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = std::fs::remove_dir_all(&self.0);
    }
}

fn run(source: &str) -> (u32, Vec<u8>, Vec<u8>) {
    let fixture = Fixture::new(source);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = fixture
        .checked()
        .run_main(&mut stdout, &mut stderr, &|| false)
        .unwrap();
    (code, stdout, stderr)
}

// Upstream's native io_step runs synchronous requests until Emit or a park.
// io_spawn only appends to the FIFO ready queue; it does not yield the parent.
const SPAWN_ORDER: &str = r#"import Base
def first() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print("first starts")
    Unit <- IO.spawn(Unit, IO.print("grandchild"))
    IO.print("first ends")

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, first())
    Unit <- IO.print("main continues")
    Unit <- IO.spawn(Unit, IO.print("second"))
    IO.print("main ends")
"#;

#[test]
fn spawn_continues_parent_and_queues_children_in_fifo_order() {
    assert_eq!(
        run(SPAWN_ORDER),
        (
            0,
            b"main continues\nmain ends\nfirst starts\nfirst ends\nsecond\ngrandchild\n".to_vec(),
            vec![],
        )
    );
}

// Even a zero-duration sleep parks. Ready tasks drain before timer promotion.
const ZERO_TIMERS: &str = r#"import Base
def late(tag: String) -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.sleep(0)
    Unit <- IO.sleep(0)
    IO.print(tag)

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, late("a"))
    Unit <- IO.spawn(Unit, late("b"))
    Unit <- IO.spawn(Unit, IO.print("ready"))
    IO.print("main")
"#;

#[test]
fn children_outlive_main_and_ready_tasks_precede_zero_timers() {
    assert_eq!(
        run(ZERO_TIMERS),
        (0, b"main\nready\na\nb\n".to_vec(), vec![])
    );
}

const PAYLOADS: &str = r#"import Base
def typed() -> IO(Type):
  do IO<Type>:
    Unit <- IO.sleep(0)
    Unit <- IO.print("type")
    return Nat

def function() -> IO(Nat -> Nat):
  do IO<Nat -> Nat>:
    Unit <- IO.sleep(0)
    Unit <- IO.print("function")
    return n => n

def main() -> IO(U32):
  do IO<U32>:
    Unit <- IO.spawn(Type, typed())
    Unit <- IO.spawn(Nat -> Nat, function())
    return 99
"#;

#[test]
fn completed_tasks_discard_non_data_payloads_and_main_payload_is_not_exit_code() {
    assert_eq!(run(PAYLOADS), (0, b"type\nfunction\n".to_vec(), vec![]));
}

const TIMER_ORDER: &str = r#"import Base
def ping(tag: String) -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.sleep(1)
    IO.print(tag)

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, ping("a"))
    Unit <- IO.spawn(Unit, ping("b"))
    Unit <- IO.spawn(Unit, ping("c"))
    Unit <- IO.spawn(Unit, ping("d"))
    IO.pure(Unit, Unit{})
"#;

#[test]
fn equal_duration_timers_preserve_registration_order() {
    assert_eq!(run(TIMER_ORDER), (0, b"a\nb\nc\nd\n".to_vec(), vec![]));
}

const MAIN_HALT: &str = r#"import Base
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, IO.print("CHILD"))
    IO.die(Unit, 7, "stop")
"#;

const CHILD_HALT: &str = r#"import Base
def late() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.sleep(100)
    IO.print("TIMER")

def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.spawn(Unit, late())
    Unit <- IO.spawn(Unit, IO.die(Unit, 7, "stop"))
    Unit <- IO.spawn(Unit, IO.print("QUEUED"))
    Unit <- IO.sleep(100)
    IO.print("MAIN")
"#;

#[test]
fn halt_in_any_task_cancels_ready_tasks_and_parked_timers() {
    for source in [MAIN_HALT, CHILD_HALT] {
        assert_eq!(run(source), (7, vec![], b"stop\n".to_vec()));
    }
}

#[test]
fn child_halt_preserves_the_existing_full_u32_status_contract() {
    // This is our public run_main contract, not a claim about a host process's
    // narrowed exit code or upstream C's signed io_step status representation.
    assert_eq!(
        run(&CHILD_HALT.replace("7, \"stop\"", "4294967295, \"stop\"")),
        (u32::MAX, vec![], b"stop\n".to_vec())
    );
}

const CLOCK: &str = r#"import Base
def elapsed(early: Bool) -> String:
  match early:
    case False{}: "ok"
    case True{}: "early"

def self_check(c: Cmp) -> String:
  match c:
    case EQ{}: "zero"
    case LT{}: "bad"
    case GT{}: "bad"

def measured(t: Nat) -> IO(Unit):
  +before = t
  do IO<Unit>:
    Unit <- IO.print(self_check(Nat.cmp(Nat.sub(before, before), 0n)))
    Unit <- IO.sleep(15)
    after : Nat <- IO.now()
    IO.print(elapsed(Nat.is_lt(Nat.sub(after, before), 14n)))

def main() -> IO(Unit):
  do IO<Unit>:
    before : Nat <- IO.now()
    measured(before)
"#;

#[test]
fn monotonic_now_supports_nat_arithmetic_and_observes_sleep_lower_bound() {
    // No assertion about absolute clock origin: upstream native and JS use
    // different monotonic clocks. Avoid expanding host uptime into unary nodes.
    assert_eq!(run(CLOCK), (0, b"zero\nok\n".to_vec(), vec![]));
}

const MATCH_CLOCK: &str = r"import Base
def roundtrip(t: Nat) -> Bool:
  match t:
    case Zero{}: Nat.is_eq(0n, 0n)
    case Succ{+p}: Nat.is_eq(Nat.sub(Succ{p}, p), 1n)

def main() -> IO(Unit):
  do IO<Unit>:
    t : Nat <- IO.now()
    IO.print(Bool.show(roundtrip(t)))
";

#[test]
fn clock_values_remain_matchable_and_reconstructable_as_ordinary_nat() {
    assert_eq!(run(MATCH_CLOCK), (0, b"True\n".to_vec(), vec![]));
}

#[test]
fn cancellation_interrupts_a_parked_maximum_duration_timer() {
    struct StartTimer<'a>(&'a Cell<Option<Instant>>);

    impl Write for StartTimer<'_> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.set(Some(Instant::now()));
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let fixture = Fixture::new(
        "import Base\ndef main() -> IO(Unit):\n  do IO<Unit>:\n    Unit <- IO.write(\"start\")\n    Unit <- IO.sleep(4294967295)\n    IO.print(\"AFTER\")\n",
    );
    let book = fixture.checked();
    let started = Cell::new(None);
    let mut stderr = Vec::new();
    let error = book
        .run_main(&mut StartTimer(&started), &mut stderr, &|| {
            started
                .get()
                .is_some_and(|time| time.elapsed() >= Duration::from_millis(30))
        })
        .unwrap_err();
    assert!(error.to_string().contains("cancelled"), "{error}");
    assert!(stderr.is_empty());
    assert!(
        started.get().unwrap().elapsed() < Duration::from_secs(3),
        "a pending timer must not hide cancellation until its deadline"
    );
}
