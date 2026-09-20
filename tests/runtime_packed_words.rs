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
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-packed-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&directory).unwrap();
        std::fs::write(directory.join("main.bend"), source).unwrap();
        Self(directory)
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

const WORD_HELPERS: &str = r"import Base
def copy(n: Nat, word: Word(n)) -> Word(n):
  match n:
    case Zero{}: WNil{}
    case Succ{p}:
      match word:
        case WCon{bit, tail}: WCon{bit, copy(p, tail)}

def copied(value: U32) -> U32:
  match value:
    case U32{word}: U32{copy(32n, word)}

def make(value: U32) -> F32:
  match value:
    case U32{word}: F32{copy(32n, word)}

def low(value: U32) -> Bool:
  match value:
    case U32{word}:
      match word:
        case WCon{bit, tail}: bit

def apply(f: U32 -> U32, value: U32) -> U32: f(value)
";

fn assert_output(source: &str, expected: &str) {
    let fixture = Fixture::new(source);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        fixture
            .checked()
            .run_main(&mut stdout, &mut stderr, &|| false)
            .unwrap(),
        0,
        "{source}"
    );
    assert_eq!(stdout, expected.as_bytes(), "{source}");
    assert!(stderr.is_empty(), "{source}");
}

fn assert_expressions(cases: &[(&str, &str)]) {
    for (expression, expected) in cases {
        assert_output(
            &format!("{WORD_HELPERS}\ndef main() -> IO(Unit): IO.print({expression})\n"),
            &format!("{expected}\n"),
        );
    }
}

#[test]
fn packed_results_and_ordinary_word_reconstruction_interoperate() {
    assert_expressions(&[
        ("U32.show(copied(0))", "0"),
        ("U32.show(copied(2147483649))", "2147483649"),
        ("U32.show(copied(2863311530))", "2863311530"),
        ("U32.show(copied(4294967295))", "4294967295"),
        ("U32.show(copied(U32.mul(65537, 65537)))", "131073"),
        ("U32.show(U32.add(copied(4294967295), 1))", "0"),
        ("U32.show(U32.mul(copied(65537), copied(65537)))", "131073"),
        ("Bool.show(low(U32.mul(3, 5)))", "True"),
        ("Bool.show(low(U32.shl(4294967295)))", "False"),
    ]);
}

#[test]
fn wrapping_multiplication_and_one_bit_shift_keep_partial_application_semantics() {
    assert_expressions(&[
        ("U32.show(U32.mul(0, 4294967295))", "0"),
        ("U32.show(U32.mul(4294967295, 4294967295))", "1"),
        ("U32.show(U32.mul(2147483648, 2))", "0"),
        ("U32.show(U32.mul(2147483648, 3))", "2147483648"),
        ("U32.show(apply(U32.mul(65537), 65537))", "131073"),
        ("U32.show(apply(U32.add(4294967295), 1))", "0"),
        ("U32.show(apply(U32.shl, 2147483649))", "2"),
        ("U32.show(U32.shl(0))", "0"),
        ("U32.show(U32.shl(2147483648))", "0"),
        ("U32.show(U32.shl(4294967295))", "4294967294"),
        ("U32.show(U32.shln(1, 31n))", "2147483648"),
        ("U32.show(U32.shln(4294967295, 32n))", "0"),
        ("U32.show(U32.shln(1, 33n))", "0"),
    ]);
}

#[test]
fn raw_float_payloads_survive_word_views_and_repacking() {
    // These are bit patterns, never host float values: both NaN categories must
    // retain their payloads across constructor matching and ordinary copying.
    for bits in [
        0_u32,
        0x8000_0000,
        1,
        0x7f80_0000,
        0xff80_0000,
        0x7fc1_2345,
        0xffc1_2345,
        0x7f81_2345,
        0xff81_2345,
    ] {
        for (expression, expected) in [
            (format!("copied(F32.bits(make({bits})))"), bits),
            (
                format!("copied(F32.bits(F32.neg(make({bits}))))"),
                bits ^ 0x8000_0000,
            ),
            (
                format!("copied(F32.bits(F32.abs(make({bits}))))"),
                bits & 0x7fff_ffff,
            ),
        ] {
            assert_output(
                &format!(
                    "{WORD_HELPERS}\ndef main() -> IO(Unit): IO.print(U32.show({expression}))\n"
                ),
                &format!("{expected}\n"),
            );
        }
    }
}

#[test]
fn namespaced_lookalikes_keep_their_ordinary_implementations() {
    let fixture = Fixture::new(
        r#"import Local.bend as Local
import Base
def show(value: Local.U32) -> String:
  match value:
    case Local.U32{tag}:
      match tag:
        case Local.Off{}: "off"
        case Local.On{}: "on"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(show(Local.U32.add(Local.U32{Local.Off{}}, Local.U32{Local.On{}})))
    Unit <- IO.print(show(Local.U32.mul(Local.U32{Local.Off{}}, Local.U32{Local.On{}})))
    IO.print(show(Local.U32.shl(Local.U32{Local.Off{}})))
"#,
    );
    std::fs::write(
        fixture.0.join("Local.bend"),
        r"type Tag is Data: Off{} On{}
type U32 is Data: U32{tag: Tag}
def U32.add(a: U32, b: U32) -> U32: b
def U32.mul(a: U32, b: U32) -> U32: b
def U32.shl(a: U32) -> U32: U32{On{}}
",
    )
    .unwrap();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        fixture
            .checked()
            .run_main(&mut stdout, &mut stderr, &|| false)
            .unwrap(),
        0
    );
    assert_eq!(stdout, b"on\non\non\n");
    assert!(stderr.is_empty());
}

const LAZY_WORD: &str = r"def expensive(n: Nat) -> Word(31n):
  match n:
    case Zero{}: Word.zero(31n)
    case Succ{p}: expensive(p)
def lazy_word() -> U32:
  U32{WCon{True{}, expensive(U32.to_nat(1000000))}}
def expensive_whole(n: Nat) -> U32:
  match n:
    case Zero{}: 7
    case Succ{p}: expensive_whole(p)
";

#[test]
fn ordinary_arithmetic_optimizations_preserve_demand_for_unused_input_bits() {
    // Independently checked with the upstream pure normalizer. The previous
    // native release already returned the shift/multiply values but exposed
    // an eager Add bug; an ordinary Word.add wrapper returned True there.
    for (expression, expected) in [
        ("Bool.show(low(U32.shl(lazy_word())))", "False\n"),
        ("Bool.show(low(U32.shln(lazy_word(), 1n)))", "False\n"),
        ("U32.show(U32.mul(0, lazy_word()))", "0\n"),
        ("Bool.show(low(U32.add(lazy_word(), 0)))", "True\n"),
    ] {
        assert_output(
            &format!("{WORD_HELPERS}{LAZY_WORD}def main() -> IO(Unit): IO.print({expression})\n"),
            expected,
        );
    }
}

#[test]
fn recognizing_a_word_does_not_force_its_unused_tail() {
    assert_output(
        &format!(
            "{WORD_HELPERS}{LAZY_WORD}def main() -> IO(Unit): IO.print(Bool.show(low(lazy_word())))\n"
        ),
        "True\n",
    );
    for expression in [
        "U32.show(lazy_word())",
        // Multiplication still demands both outer U32 constructors, even when
        // its first operand is zero. This also refused in the previous release.
        "U32.show(U32.mul(0, expensive_whole(U32.to_nat(1000000))))",
    ] {
        let fixture = Fixture::new(&format!(
            "{WORD_HELPERS}{LAZY_WORD}def main() -> IO(Unit): IO.print({expression})\n"
        ));
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let error = fixture
            .checked()
            .run_main(&mut stdout, &mut stderr, &|| false)
            .expect_err("demanding an expensive field must retain resource limits")
            .to_string();
        assert!(error.contains("exhausted"), "{error}");
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }
}

#[test]
fn packed_halt_codes_and_request_matching_remain_distinct() {
    let fixture = Fixture::new(
        "import Base\ndef main() -> IO(Unit): IO.die(Unit, U32.mul(4294967295, 3), \"stop\")\n",
    );
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    assert_eq!(
        fixture
            .checked()
            .run_main(&mut stdout, &mut stderr, &|| false)
            .unwrap(),
        4_294_967_293
    );
    assert!(stdout.is_empty());
    assert_eq!(stderr, b"stop\n");

    let fixture = Fixture::new(
        r#"import Base
def intercept(op: IO.OP<Unit>) -> IO(Unit):
  match op:
    case Emit{value}: IO.pure(Unit, Unit{})
    case rest: IO.print("fallback")
def main() -> IO(Unit):
  intercept(IO.print(U32.show(U32.mul(7, 6)))(Unit, value => Emit{value}))
"#,
    );
    stdout.clear();
    stderr.clear();
    let error = fixture
        .checked()
        .run_main(&mut stdout, &mut stderr, &|| false)
        .unwrap_err()
        .to_string();
    assert!(error.contains("foreign effect request"), "{error}");
    assert!(stdout.is_empty());
    assert!(stderr.is_empty());
}

#[test]
fn long_numeric_chains_remain_cancellable_and_continuation_bounded() {
    let helpers = "import Base\ndef chain(n: Nat) -> U32:\n  match n:\n    case Zero{}: 1\n    case Succ{p}: U32.mul(chain(p), 3)\n";
    assert_output(
        &format!("{helpers}def main() -> IO(Unit): IO.print(U32.show(chain(10n)))\n"),
        "59049\n",
    );
    let fixture = Fixture::new(&format!(
        "{helpers}def main() -> IO(Unit): IO.print(U32.show(chain(U32.to_nat(5000))))\n"
    ));
    let checked = fixture.checked();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let ticks = Cell::new(0);
    let error = checked
        .run_main(&mut stdout, &mut stderr, &|| {
            ticks.set(ticks.get() + 1);
            ticks.get() > 1000
        })
        .unwrap_err()
        .to_string();
    assert!(error.contains("cancelled"), "{error}");
    assert!(stdout.is_empty());
    assert!(stderr.is_empty());
    let error = checked
        .run_main(&mut stdout, &mut stderr, &|| false)
        .unwrap_err()
        .to_string();
    assert!(error.contains("continuation depth exhausted"), "{error}");
    assert!(stdout.is_empty());
    assert!(stderr.is_empty());
}
