// SPDX-License-Identifier: MPL-2.0
use std::cell::Cell;
use std::path::PathBuf;
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
            "teamy-bend-reclamation-{}-{}.bend",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, source).unwrap();
        Self(path)
    }

    fn checked(&self) -> ExecutableBook {
        check_executable(&load_executable(&self.0).unwrap()).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = std::fs::remove_file(&self.0);
    }
}

const WORK: &str = r"import Base
def work(n: Nat) -> U32:
  match n:
    case Zero{}: 1
    case Succ{+p}: U32.add(work(p), work(p))
";

fn assert_run(source: &str, status: u32, expected_stdout: &str, expected_stderr: &str) {
    let fixture = Fixture::new(source);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        fixture
            .checked()
            .run_main(&mut stdout, &mut stderr, &|| false)
            .unwrap(),
        status
    );
    assert_eq!(stdout, expected_stdout.as_bytes());
    assert_eq!(stderr, expected_stderr.as_bytes());
}

#[test]
fn discarded_intermediates_are_reclaimed_without_changing_the_workload() {
    // The clean pre-collector release succeeds at depth 12 but exhausts its
    // append-only arena at depth 13. The expected integer is independently 2^13.
    assert_run(
        &format!("{WORK}def main() -> IO(Unit): IO.print(U32.show(work(13n)))\n"),
        0,
        "8192\n",
        "",
    );
}

#[test]
fn shared_closures_and_simultaneous_let_environments_survive_collection() {
    assert_run(
        &format!(
            r"{WORK}
def capture(+seed: U32) -> U32 -> U32: value => U32.add(seed, value)
def scoped(+seed: U32) -> IO(Unit):
  f g = capture(seed) capture(seed)
  seed old = U32.add(seed, 100) seed
  do IO<Unit>:
    Unit <- IO.print(U32.show(work(13n)))
    Unit <- IO.print(U32.show(f(old)))
    Unit <- IO.print(U32.show(g(1)))
    IO.print(U32.show(seed))
def main() -> IO(Unit): scoped(7)
"
        ),
        0,
        "8192\n14\n8\n107\n",
        "",
    );
}

const WORD_VIEW: &str = r"def delayed_bit(n: Nat) -> Bool: U32.is_eq(work(n), 0)
def replace_low(n: Nat, value: U32) -> U32:
  match value:
    case U32{word}:
      match word:
        case WCon{bit, tail}: U32{WCon{delayed_bit(n), tail}}
def as_float(value: U32) -> F32:
  match value:
    case U32{word}: F32{word}
";

#[test]
fn packed_word_tails_survive_collection_while_computed_heads_are_demanded() {
    assert_run(
        &format!(
            "{WORK}{WORD_VIEW}def main() -> IO(Unit): IO.print(U32.show(F32.bits(as_float(replace_low(13n, 2147483649)))))\n"
        ),
        0,
        "2147483648\n",
        "",
    );
}

#[test]
fn numeric_continuations_keep_later_arguments_alive() {
    assert_run(
        &format!(
            "{WORK}def main() -> IO(Unit): IO.print(F32.show(F32.add(U32.to_f32(work(13n)), U32.to_f32(work(1n)))))\n"
        ),
        0,
        "8194\n",
        "",
    );
}

#[test]
fn request_arguments_and_continuations_preserve_effect_order_across_collections() {
    assert_run(
        &format!(
            r#"{WORK}
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.write("first:" ++ U32.show(work(12n)) ++ "\0")
    Unit <- IO.print_err("second:" ++ U32.show(work(12n)))
    IO.print("third:" ++ U32.show(work(12n)))
"#
        ),
        0,
        "first:4096\0third:4096\n",
        "second:4096\n",
    );
}

#[test]
fn halt_message_siblings_survive_code_decoding_and_stop_later_effects() {
    assert_run(
        &format!(
            r#"{WORK}{WORD_VIEW}
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print("before")
    Unit <- IO.die(Unit, replace_low(13n, 2147483649), "halt\0é")
    IO.print("after")
"#
        ),
        2_147_483_648,
        "before\n",
        "halt\0é\n",
    );
}

#[test]
fn collection_never_turns_a_request_into_matchable_data() {
    let fixture = Fixture::new(&format!(
        r#"{WORK}
def intercept(op: IO.OP<Unit>) -> IO(Unit):
  match op:
    case Emit{{value}}: IO.pure(Unit, Unit{{}})
    case other: IO.print("fallback")
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(U32.show(work(13n)))
    intercept(IO.print("forbidden")(Unit, value => Emit{{value}}))
"#
    ));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let error = fixture
        .checked()
        .run_main(&mut stdout, &mut stderr, &|| false)
        .unwrap_err()
        .to_string();
    assert!(error.contains("foreign effect request"), "{error}");
    assert_eq!(stdout, b"8192\n");
    assert!(stderr.is_empty());
}

#[test]
fn cancellation_remains_effective_after_substantial_allocation() {
    let fixture = Fixture::new(&format!(
        "{WORK}def main() -> IO(Unit): IO.print(U32.show(work(16n)))\n"
    ));
    let ticks = Cell::new(0);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let error = fixture
        .checked()
        .run_main(&mut stdout, &mut stderr, &|| {
            ticks.set(ticks.get() + 1);
            ticks.get() > 700_000
        })
        .unwrap_err()
        .to_string();
    assert!(error.contains("cancelled"), "{error}");
    assert!(ticks.get() > 700_000);
    assert!(stdout.is_empty(), "unexpected output: {stdout:?}");
    assert!(stderr.is_empty());
}

#[test]
fn retained_graph_pressure_remains_bounded() {
    let fields = (0..32)
        .map(|index| format!("b{index}: Bool"))
        .collect::<Vec<_>>()
        .join(", ");
    let names = (0..32)
        .map(|index| format!("b{index}"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut values = vec!["True{}"; 32];
    values[0] = "salt";
    let values = values.join(", ");
    // 4,096 distinct, forced Wide values require at least 131,072 distinct
    // field thunks, before counting tree nodes or any execution state. Keep the
    // tree in a shared function parameter while its first traversal runs.
    let fixture = Fixture::new(&format!(
        r"import Base
type Wide is Data: Wide{{{fields}}}
type Tree is Data:
  Leaf{{wide: Wide}}
  Branch{{left: Tree, right: Tree}}
def make(n: Nat, salt: Bool) -> Tree:
  match n:
    case Zero{{}}: Leaf{{Wide{{{values}}}}}
    case Succ{{+p}}: Branch{{make(p, True{{}}), make(p, True{{}})}}
def count(tree: Tree) -> U32:
  match tree:
    case Leaf{{wide}}:
      match wide:
        case Wide{{{names}}}: 1
    case Branch{{left, right}}: U32.add(count(left), count(right))
def retained(+tree: Tree) -> U32: U32.add(count(tree), count(tree))
def main() -> IO(Unit): IO.print(U32.show(retained(make(12n, True{{}}))))
"
    ));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let error = fixture
        .checked()
        .run_main(&mut stdout, &mut stderr, &|| false)
        .unwrap_err()
        .to_string();
    // Marking a large retained graph can spend the unchanged step budget before
    // the live arena fills. Both are explicit resource refusals, never success.
    assert!(
        error.contains("thunk budget exhausted") || error.contains("step budget exhausted"),
        "{error}"
    );
    assert!(stdout.is_empty(), "unexpected output: {stdout:?}");
    assert!(stderr.is_empty());
}
