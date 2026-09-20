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
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-channels-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn run(&self, source: &str, foreign: &str) -> Output {
        fs::write(self.0.join("main.bend"), format!("{HELPERS}{source}")).unwrap();
        fs::write(self.0.join("effect.js"), foreign).unwrap();
        let loaded = load_executable(self.0.join("main.bend")).unwrap();
        let checked = check_executable(&loaded).unwrap();
        let javascript = compile_executable_javascript(&checked).unwrap();
        fs::write(self.0.join("main.cjs"), javascript).unwrap();
        Command::new(std::env::var_os("TEAMY_BEND_NODE").unwrap_or_else(|| "node".into()))
            .arg(self.0.join("main.cjs"))
            .output()
            .expect("channel compiler tests require Node.js")
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
fn independent_buffers_drain_after_close_and_stale_copies_stay_closed() {
    let output = Fixture::new().run(
        r"def use(a: Chan(U32), b: Chan(U32)) -> IO(Unit):
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
    a : Maybe<&1, U32> <- Chan.recv(U32, x)
    Unit <- IO.print(maybe(a))
    Chan.close(U32, x)
def main() -> IO(Unit):
  do IO<Unit>:
    a : Chan(U32) <- Chan.new(U32, 2)
    b : Chan(U32) <- Chan.new(U32, 1)
    use(a, b)
",
        "",
    );
    success(&output, "false\n1\n9\n2\nnone\n");
}

#[test]
fn blocked_senders_resume_fifo_and_receives_refill_the_buffer() {
    let output = Fixture::new().run(
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
        "",
    );
    success(&output, "0\n1\n2\nsent1\nsent2\n");
}

#[test]
fn close_wakes_waiting_senders_once_without_discarding_the_buffer() {
    let output = Fixture::new().run(
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
        "",
    );
    success(&output, "0\n1false\n2false\n");
}

#[test]
fn close_wakes_waiting_receivers_once_in_fifo_order() {
    let output = Fixture::new().run(
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
        "",
    );
    success(&output, "closed\nanone\nbnone\n");
}

#[test]
fn fork_join_transfers_an_affine_payload_and_a_second_join_halts() {
    let fixture = Fixture::new();
    success(
        &fixture.run(
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
            "",
        ),
        "42\n",
    );
    let output = fixture.run(
        r#"def join_twice(c: Chan(U32)) -> IO(Unit):
  +ch = c
  do IO<Unit>:
    first : U32 <- IO.join(U32, ch)
    Unit <- IO.print(U32.show(first))
    second : U32 <- IO.join(U32, ch)
    IO.print("BAD")
def main() -> IO(Unit): IO.bind(Chan(U32), Unit, IO.fork(U32, IO.pure(U32, 7)), join_twice)
"#,
        "",
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"7\n");
    assert_eq!(output.stderr, b"IO.join: the channel was closed\n");
}

#[test]
fn reference_null_payload_sentinel_behavior_depends_on_registration_order() {
    let fixture = Fixture::new();
    let prefix = r#"def P() -> Type: {0 == 0 : U32}
def sender(c: Chan(P)) -> IO(Unit):
  do IO<Unit>:
    ok : Bool <- Chan.send(P, c, {==})
    IO.print("sent")
def receiver(c: Chan(P)) -> IO(Unit):
  do IO<Unit>:
    got : Maybe<&1, P> <- Chan.recv(P, c)
    IO.print("received")
"#;
    for (room, first, second, deadlock) in [
        (0, "sender", "receiver", true),
        (0, "receiver", "sender", false),
        (1, "sender", "receiver", false),
    ] {
        let output = fixture.run(&format!("{prefix}def start(c: Chan(P)) -> IO(Unit):\n  +ch = c\n  do IO<Unit>:\n    Unit <- IO.spawn(Unit, {first}(ch))\n    IO.spawn(Unit, {second}(ch))\ndef main() -> IO(Unit): IO.bind(Chan(P), Unit, Chan.new(P, {room}), start)\n"), "");
        if deadlock {
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8_lossy(&output.stderr).contains("deadlock"));
        } else {
            success(&output, "sent\nreceived\n");
        }
    }
}

#[test]
fn foreign_created_rows_keep_raw_array_identity_and_builtin_lexical_isolation() {
    let output = Fixture::new().run(
        r#"def make() -> IO(Chan(U32)): import "effect.js"
def inspect(c: Chan(U32)) -> IO(Unit): import "effect.js"
def start(c: Chan(U32)) -> IO(Unit):
  +ch = c
  do IO<Unit>:
    first : Maybe<&1, U32> <- Chan.recv(U32, ch)
    Unit <- inspect(ch)
    Unit <- IO.print(maybe(first))
    Unit <- Chan.close(U32, ch)
    second : Maybe<&1, U32> <- Chan.recv(U32, ch)
    IO.print(maybe(second))
def main() -> IO(Unit): IO.bind(Chan(U32), Unit, make(), start)
"#,
        "const ring=[7,8];const row={room:2,ring,wait:[],shut:false};function make(){return row}function inspect(c){if(c!==row||c.ring!==ring||ring.length!==1||ring[0]!==8)throw Error('raw row changed');return {$:'Unit'}}function $tbChanRecv(){throw Error('SHADOW')}function chan_take(){throw Error('SHADOW')}function $tbChanClose(){throw Error('SHADOW')}",
    );
    success(&output, "7\n8\n");
}

#[test]
fn retained_foreign_rows_are_unchanged_when_main_completes_or_halts() {
    let fixture = Fixture::new();
    for (terminal, status) in [
        ("IO.pure(Unit, Unit{})", 0),
        ("IO.die(Unit, 7, \"stop\")", 7),
    ] {
        let output = fixture.run(&format!(r#"def keep(c: Chan(U32)) -> IO(Unit): import "effect.js"
def start(c: Chan(U32)) -> IO(Unit):
  +ch = c
  do IO<Unit>:
    ok : Bool <- Chan.send(U32, ch, 9)
    Unit <- keep(ch)
    {terminal}
def main() -> IO(Unit): IO.bind(Chan(U32), Unit, Chan.new(U32, 1), start)
"#), "let saved;function keep(c){saved=c;return {$:'Unit'}}process.on('exit',()=>{process.stdout.write(JSON.stringify({shut:saved.shut,ring:saved.ring,wait:saved.wait.length})+'\\n')});");
        assert_eq!(output.status.code(), Some(status));
        assert_eq!(output.stdout, b"{\"shut\":false,\"ring\":[9],\"wait\":0}\n");
        assert_eq!(
            output.stderr,
            if status == 0 {
                &b""[..]
            } else {
                &b"stop\n"[..]
            }
        );
    }
}

#[test]
fn channel_payloads_do_not_grant_permission_to_inspect_pending_requests() {
    let output = Fixture::new().run(
        r#"def secret() -> IO(Unit): import "effect.js"
def inspect(op: IO.OP<U32>) -> IO(Unit):
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
    ok : Bool <- Chan.send(IO.OP<U32>, ch, secret()(U32, x => Emit{7}))
    value : Maybe<&1, IO.OP<U32>> <- Chan.recv(IO.OP<U32>, ch)
    inspect_result(value)
def main() -> IO(Unit): IO.bind(Chan(IO.OP<U32>), Unit, Chan.new(IO.OP<U32>, 1), start)
"#,
        "function secret(){process.stdout.write('SECRET');return {$:'Unit'}}",
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("request inspected outside"));
}

#[test]
fn channel_waits_deadlock_and_halt_cancels_a_blocked_child() {
    let fixture = Fixture::new();
    let output = fixture.run("def main() -> IO(Maybe<&1, U32>): IO.bind(Chan(U32), Maybe<&1, U32>, Chan.new(U32, 0), c => Chan.recv(U32, c))\n", "");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("deadlock"));
    let output = fixture.run(
        r#"def child(c: Chan(U32)) -> IO(Unit):
  do IO<Unit>:
    value : Maybe<&1, U32> <- Chan.recv(U32, c)
    IO.print("BAD")
def main() -> IO(Unit):
  do IO<Unit>:
    c : Chan(U32) <- Chan.new(U32, 0)
    Unit <- IO.spawn(Unit, child(c))
    Unit <- IO.sleep(0)
    IO.die(Unit, 7, "stop")
"#,
        "",
    );
    assert_eq!(output.status.code(), Some(7));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"stop\n");
}

#[test]
fn channel_rows_values_and_waiters_have_separate_aggregate_bounds() {
    let fixture = Fixture::new();
    let source = "def make() -> IO(Chan(U32)): import \"effect.js\"\ndef main() -> IO(Unit): IO.bind(Chan(U32), Unit, make(), c => Chan.close(U32, c))\n";
    for (foreign, diagnostic) in [
        (
            "function make(){return {room:1,ring:Array(131073).fill(1),wait:[],shut:false}}",
            "channel buffer budget",
        ),
        (
            "function make(){return {room:0,ring:[],wait:Array(131073).fill({cont:()=>0,item:null}),shut:false}}",
            "channel waiter budget",
        ),
        (
            "function make(){for(let i=0;i<=131072;i++)$tbChanNew(0);throw Error('missed limit')}",
            "channel handle budget",
        ),
        (
            "function make(){return {$:'Chan',ring:[],wait:null}}",
            "invalid channel row",
        ),
    ] {
        let output = fixture.run(source, foreign);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains(diagnostic));
    }
    let output = fixture.run(
        r#"def make() -> IO(Chan(U32)): import "effect.js"
def main() -> IO(Unit):
  do IO<Unit>:
    a : Chan(U32) <- make()
    b : Chan(U32) <- make()
    Unit <- Chan.close(U32, a)
    Chan.close(U32, b)
"#,
        "function make(){return {room:70000,ring:Array(70000).fill(1),wait:[],shut:false}}",
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("channel buffer budget"));
}

#[test]
fn for_each_is_sequential_empty_is_quiet_and_halt_skips_the_tail() {
    let fixture = Fixture::new();
    let visit = r"def visit(n: U32) -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.sleep(0)
    IO.print(U32.show(n))
";
    success(
        &fixture.run(
            &format!(
                "{visit}def main() -> IO(Unit): List.for_each(~&2, ~U32, ~visit, [1, 2, 3])\n"
            ),
            "",
        ),
        "1\n2\n3\n",
    );
    success(
        &fixture.run(
            &format!("{visit}def main() -> IO(Unit): List.for_each(~&2, ~U32, ~visit, [])\n"),
            "",
        ),
        "",
    );
    let output = fixture.run(
        r#"def visit(n: U32) -> IO(Unit): IO.die(Unit, n, "stop")
def main() -> IO(Unit): List.for_each(~&2, ~U32, ~visit, [7, 8, 9])
"#,
        "",
    );
    assert_eq!(output.status.code(), Some(7));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, b"stop\n");
}
