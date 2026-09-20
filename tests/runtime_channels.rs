// SPDX-License-Identifier: MPL-2.0
use std::cell::Cell;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::kernel::ExecutableBook;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);
const HELPERS: &str = r#"import Base
def maybe(value: Maybe<&1, U32>) -> String:
  match value:
    case None{}: "none"
    case Some{x}: U32.show(x)

def flag(value: Bool) -> String:
  match value:
    case False{}: "false"
    case True{}: "true"
"#;

struct Fixture(PathBuf);

impl Fixture {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-native-channels-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("main.bend"), format!("{HELPERS}{source}")).unwrap();
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

fn success(source: &str, expected: &str) {
    assert_eq!(run(source), (0, expected.as_bytes().to_vec(), vec![]));
}

#[test]
fn independent_buffers_drain_after_close_and_stale_handles_cannot_reopen() {
    success(
        r"def use_fresh(old: Chan(U32), fresh: Chan(U32)) -> IO(Unit):
  +stale = old
  +new = fresh
  do IO<Unit>:
    ok : Bool <- Chan.send(U32, new, 42)
    Unit <- Chan.close(U32, stale)
    ok : Bool <- Chan.send(U32, stale, 99)
    Unit <- IO.print(flag(ok))
    a : Maybe<&1, U32> <- Chan.recv(U32, stale)
    Unit <- IO.print(maybe(a))
    a : Maybe<&1, U32> <- Chan.recv(U32, new)
    IO.print(maybe(a))

def use(a: Chan(U32), b: Chan(U32)) -> IO(Unit):
  +x = a
  +y = b
  do IO<Unit>:
    ok : Bool <- Chan.send(U32, x, 1)
    ok : Bool <- Chan.send(U32, x, 2)
    ok : Bool <- Chan.send(U32, y, 9)
    Unit <- Chan.close(U32, x)
    ok : Bool <- Chan.send(U32, x, 3)
    Unit <- IO.print(flag(ok))
    a : Maybe<&1, U32> <- Chan.recv(U32, x)
    Unit <- IO.print(maybe(a))
    b : Maybe<&1, U32> <- Chan.recv(U32, y)
    Unit <- IO.print(maybe(b))
    a : Maybe<&1, U32> <- Chan.recv(U32, x)
    Unit <- IO.print(maybe(a))
    fresh : Chan(U32) <- Chan.new(U32, 1)
    use_fresh(x, fresh)

def main() -> IO(Unit):
  do IO<Unit>:
    a : Chan(U32) <- Chan.new(U32, 2)
    b : Chan(U32) <- Chan.new(U32, 1)
    use(a, b)
",
        "false\n1\n9\n2\nfalse\nnone\n42\n",
    );
}

#[test]
fn blocked_senders_resume_fifo_and_receives_refill_the_buffer() {
    success(
        r#"def send(c: Chan(U32), +n: U32) -> IO(Unit):
  do IO<Unit>:
    ok : Bool <- Chan.send(U32, c, n)
    IO.print(String.append("sent", U32.show(n)))

def start(c: Chan(U32)) -> IO(Unit):
  +ch = c
  do IO<Unit>:
    ok : Bool <- Chan.send(U32, ch, 0)
    Unit <- IO.spawn(Unit, send(ch, 1))
    Unit <- IO.spawn(Unit, send(ch, 2))
    Unit <- IO.sleep(0)
    a : Maybe<&1, U32> <- Chan.recv(U32, ch)
    Unit <- IO.print(maybe(a))
    b : Maybe<&1, U32> <- Chan.recv(U32, ch)
    Unit <- IO.print(maybe(b))
    c : Maybe<&1, U32> <- Chan.recv(U32, ch)
    IO.print(maybe(c))

def main() -> IO(Unit): IO.bind(Chan(U32), Unit, Chan.new(U32, 1), start)
"#,
        "0\n1\n2\nsent1\nsent2\n",
    );
}

#[test]
fn rendezvous_receivers_resume_fifo_after_the_sender_finishes() {
    success(
        r#"def receive(c: Chan(U32), tag: String) -> IO(Unit):
  do IO<Unit>:
    value : Maybe<&1, U32> <- Chan.recv(U32, c)
    IO.print(String.append(tag, maybe(value)))

def start(c: Chan(U32)) -> IO(Unit):
  +ch = c
  do IO<Unit>:
    Unit <- IO.spawn(Unit, receive(ch, "a"))
    Unit <- IO.spawn(Unit, receive(ch, "b"))
    Unit <- IO.sleep(0)
    ok : Bool <- Chan.send(U32, ch, 1)
    ok : Bool <- Chan.send(U32, ch, 2)
    IO.print("sent")

def main() -> IO(Unit): IO.bind(Chan(U32), Unit, Chan.new(U32, 0), start)
"#,
        "sent\na1\nb2\n",
    );
}

#[test]
fn close_wakes_waiting_senders_once_without_discarding_the_buffer() {
    success(
        r"def send(c: Chan(U32), +n: U32) -> IO(Unit):
  do IO<Unit>:
    ok : Bool <- Chan.send(U32, c, n)
    IO.print(String.append(U32.show(n), flag(ok)))

def start(c: Chan(U32)) -> IO(Unit):
  +ch = c
  do IO<Unit>:
    ok : Bool <- Chan.send(U32, ch, 0)
    Unit <- IO.spawn(Unit, send(ch, 1))
    Unit <- IO.spawn(Unit, send(ch, 2))
    Unit <- IO.sleep(0)
    Unit <- Chan.close(U32, ch)
    Unit <- Chan.close(U32, ch)
    a : Maybe<&1, U32> <- Chan.recv(U32, ch)
    IO.print(maybe(a))

def main() -> IO(Unit): IO.bind(Chan(U32), Unit, Chan.new(U32, 1), start)
",
        "0\n1false\n2false\n",
    );
}

#[test]
fn close_wakes_waiting_receivers_once_in_fifo_order() {
    success(
        r#"def receive(c: Chan(U32), tag: String) -> IO(Unit):
  do IO<Unit>:
    value : Maybe<&1, U32> <- Chan.recv(U32, c)
    IO.print(String.append(tag, maybe(value)))

def start(c: Chan(U32)) -> IO(Unit):
  +ch = c
  do IO<Unit>:
    Unit <- IO.spawn(Unit, receive(ch, "a"))
    Unit <- IO.spawn(Unit, receive(ch, "b"))
    Unit <- IO.sleep(0)
    Unit <- Chan.close(U32, ch)
    Unit <- Chan.close(U32, ch)
    IO.print("closed")

def main() -> IO(Unit): IO.bind(Chan(U32), Unit, Chan.new(U32, 0), start)
"#,
        "closed\nanone\nbnone\n",
    );
}

#[test]
fn fork_join_transfers_affine_payloads_and_a_second_join_halts() {
    success(
        r"type Token is Type: Token{value: U32}

def show(token: Token) -> IO(Unit):
  match token:
    case Token{value}: IO.print(U32.show(value))

def main() -> IO(Unit):
  do IO<Unit>:
    child : Chan(Token) <- IO.fork(Token, IO.pure(Token, Token{42}))
    token : Token <- IO.join(Token, child)
    show(token)
",
        "42\n",
    );
    assert_eq!(
        run(r#"def join_twice(c: Chan(U32)) -> IO(Unit):
  +ch = c
  do IO<Unit>:
    first : U32 <- IO.join(U32, ch)
    Unit <- IO.print(U32.show(first))
    second : U32 <- IO.join(U32, ch)
    IO.print("BAD")

def main() -> IO(Unit): IO.bind(Chan(U32), Unit, IO.fork(U32, IO.pure(U32, 7)), join_twice)
"#),
        (
            1,
            b"7\n".to_vec(),
            b"IO.join: the channel was closed\n".to_vec()
        )
    );
}

#[test]
fn handles_and_captured_functions_remain_usable_as_channel_payloads() {
    success(
        r"def apply(f: U32 -> U32) -> IO(Unit): IO.print(U32.show(f(5)))

def transfer(c: Chan(U32)) -> IO(Unit):
  +ch = c
  do IO<Unit>:
    ok : Bool <- Chan.send(U32, ch, 37)
    carrier : Chan(Chan(U32)) <- IO.fork(Chan(U32), IO.pure(Chan(U32), ch))
    received : Chan(U32) <- IO.join(Chan(U32), carrier)
    n : U32 <- IO.join(U32, received)
    f : Chan(U32 -> U32) <- IO.fork(U32 -> U32, IO.pure(U32 -> U32, x => U32.add(n, x)))
    fun : (U32 -> U32) <- IO.join(U32 -> U32, f)
    apply(fun)

def main() -> IO(Unit): IO.bind(Chan(U32), Unit, Chan.new(U32, 1), transfer)
",
        "42\n",
    );
}

#[test]
fn erased_payloads_are_not_confused_with_the_native_receiver_sentinel() {
    // Native C parks receivers with TERM_HOLE; erased proofs/types are payloads.
    // JS uses null for both and deadlocks on sender-first unbuffered proofs.
    // Preserve native semantics instead of importing that JS representation bug.
    for (payload_type, payload) in [("{0 == 0 : U32}", "{==}"), ("Type", "Nat")] {
        let prefix = format!(
            r#"def P() -> Type: {payload_type}
def sender(c: Chan(P)) -> IO(Unit):
  do IO<Unit>:
    ok : Bool <- Chan.send(P, c, {payload})
    IO.print("sent")

def received(got: Maybe<&1, P>) -> IO(Unit):
  match got:
    case None{{}}: IO.print("BAD none")
    case Some{{value}}: IO.print("received")

def receiver(c: Chan(P)) -> IO(Unit):
  IO.bind(Maybe<&1, P>, Unit, Chan.recv(P, c), received)
"#
        );
        for (room, first, second, expected) in [
            (0, "sender", "receiver", "received\nsent\n"),
            (0, "receiver", "sender", "sent\nreceived\n"),
            (1, "sender", "receiver", "sent\nreceived\n"),
        ] {
            success(
                &format!(
                    "{prefix}def start(c: Chan(P)) -> IO(Unit):\n  +ch = c\n  do IO<Unit>:\n    Unit <- IO.spawn(Unit, {first}(ch))\n    IO.spawn(Unit, {second}(ch))\ndef main() -> IO(Unit): IO.bind(Chan(P), Unit, Chan.new(P, {room}), start)\n"
                ),
                expected,
            );
        }
    }
}

#[test]
fn channel_waits_report_deadlock_for_both_sender_and_receiver() {
    for source in [
        "def main() -> IO(Maybe<&1, U32>): IO.bind(Chan(U32), Maybe<&1, U32>, Chan.new(U32, 0), c => Chan.recv(U32, c))\n",
        "def main() -> IO(Bool): IO.bind(Chan(U32), Bool, Chan.new(U32, 0), c => Chan.send(U32, c, 42))\n",
    ] {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let error = Fixture::new(source)
            .checked()
            .run_main(&mut stdout, &mut stderr, &|| false)
            .unwrap_err();
        assert!(error.to_string().contains("deadlock"), "{error}");
        assert!(stdout.is_empty() && stderr.is_empty());
    }
}

#[test]
fn halt_cancels_channel_waiters_and_never_resumes_their_continuations() {
    for operation in [
        "value : Maybe<&1, U32> <- Chan.recv(U32, c)",
        "ok : Bool <- Chan.send(U32, c, 42)",
    ] {
        assert_eq!(
            run(&format!(
                r#"def child(c: Chan(U32)) -> IO(Unit):
  do IO<Unit>:
    {operation}
    IO.print("BAD")

def main() -> IO(Unit):
  do IO<Unit>:
    c : Chan(U32) <- Chan.new(U32, 0)
    Unit <- IO.spawn(Unit, child(c))
    Unit <- IO.sleep(0)
    IO.die(Unit, 4294967295, "stop")
"#
            )),
            (u32::MAX, vec![], b"stop\n".to_vec())
        );
    }
}

#[test]
fn cancellation_drops_parked_channel_tasks() {
    struct CancelWriter<'a> {
        cancelled: &'a Cell<bool>,
        bytes: Vec<u8>,
    }
    impl Write for CancelWriter<'_> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.bytes.extend_from_slice(buf);
            self.cancelled.set(true);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let fixture = Fixture::new(
        r#"def child(c: Chan(U32)) -> IO(Unit):
  do IO<Unit>:
    value : Maybe<&1, U32> <- Chan.recv(U32, c)
    IO.print("BAD")

def main() -> IO(Unit):
  do IO<Unit>:
    c : Chan(U32) <- Chan.new(U32, 0)
    Unit <- IO.spawn(Unit, child(c))
    Unit <- IO.sleep(0)
    IO.write("cancel")
"#,
    );
    let cancelled = Cell::new(false);
    let mut stdout = CancelWriter {
        cancelled: &cancelled,
        bytes: Vec::new(),
    };
    let mut stderr = Vec::new();
    let error = fixture
        .checked()
        .run_main(&mut stdout, &mut stderr, &|| cancelled.get())
        .unwrap_err();
    assert!(error.to_string().contains("cancelled"), "{error}");
    assert!(cancelled.get() && stderr.is_empty());
    assert_eq!(stdout.bytes, b"cancel");
}

#[test]
fn channel_payloads_cannot_expose_or_execute_private_effect_requests() {
    let fixture = Fixture::new(
        r#"def inspect(op: IO.OP<U32>) -> IO(Unit):
  match op:
    case Emit{x}: IO.print("EMIT")
    case _: IO.print("FALLBACK")

def inspect_result(value: Maybe<&1, IO.OP<U32>>) -> IO(Unit):
  match value:
    case None{}: IO.print("NONE")
    case Some{op}: inspect(op)

def start(c: Chan(IO.OP<U32>)) -> IO(Unit):
  +ch = c
  do IO<Unit>:
    ok : Bool <- Chan.send(IO.OP<U32>, ch, IO.print("SECRET")(U32, x => Emit{7}))
    value : Maybe<&1, IO.OP<U32>> <- Chan.recv(IO.OP<U32>, ch)
    inspect_result(value)

def main() -> IO(Unit): IO.bind(Chan(IO.OP<U32>), Unit, Chan.new(IO.OP<U32>, 1), start)
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
            .contains("foreign effect request cannot be matched as constructor data"),
        "{error}"
    );
    assert!(stdout.is_empty() && stderr.is_empty());
}
