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
        ("U32.show(U32.shr(copied(4294967295)))", "2147483647"),
        (
            "U32.show(U32.and(copied(2147483649), 4294967295))",
            "2147483649",
        ),
        (
            "U32.show(U32.and(4294967295, copied(2147483649)))",
            "2147483649",
        ),
        ("U32.show(apply(U32.shr, 2147483649))", "1073741824"),
        ("U32.show(apply(U32.and(2863311530), 1431655765))", "0"),
        ("U32.show(U32.shrn(4294967295, 31n))", "1"),
        ("U32.show(U32.shrn(4294967295, 32n))", "0"),
        ("U32.show(U32.shrn(4294967295, 33n))", "0"),
        ("U32.show(U32.shl(0))", "0"),
        ("U32.show(U32.shl(2147483648))", "0"),
        ("U32.show(U32.shl(4294967295))", "4294967294"),
        ("U32.show(U32.shln(1, 31n))", "2147483648"),
        ("U32.show(U32.shln(4294967295, 32n))", "0"),
        ("U32.show(U32.shln(1, 33n))", "0"),
    ]);
}

#[test]
fn original_unsigned_division_programs_complete_with_unchanged_limits() {
    for (source, expected) in [
        (include_str!("fixtures/cpu-words/word_div_mod.bend"), "9\n"),
        (include_str!("fixtures/cpu-words/u32_divmod.bend"), "14n\n"),
        (
            include_str!("fixtures/cpu-words/u32_divmod_zero.bend"),
            "6n\n",
        ),
    ] {
        assert_output(source, expected);
    }
}

#[test]
fn unsigned_division_preserves_full_width_partial_calls_and_ordinary_fallback() {
    assert_expressions(&[
        ("U32.show(U32.div(4294967295, 1))", "4294967295"),
        ("U32.show(U32.mod(4294967295, 1))", "0"),
        ("U32.show(U32.div(4000000000, 3000000000))", "1"),
        ("U32.show(U32.mod(4000000000, 3000000000))", "1000000000"),
        ("U32.show(U32.div(copied(17), 5))", "3"),
        ("U32.show(U32.mod(17, copied(5)))", "2"),
    ]);
    assert_output(
        r"import Base
def main() -> IO(Unit):
  divide = U32.div(4294967295)
  modulo = U32.mod(4294967295)
  do IO<Unit>:
    Unit <- IO.print(U32.show(divide(10)))
    IO.print(U32.show(modulo(10)))
",
        "429496729\n5\n",
    );
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
fn live_word_arguments_force_computed_tails_within_fixed_work_limits() {
    // Executable by-value demand includes live Word fields. The proof/pure
    // normalizer remains lazy; its observations are not an executable oracle.
    for expression in [
        "Bool.show(low(lazy_word()))",
        "Bool.show(low(U32.inc(lazy_word())))",
        "Bool.show(low(U32.shl(lazy_word())))",
        "Bool.show(low(U32.shln(lazy_word(), 1n)))",
        "U32.show(U32.mul(0, lazy_word()))",
        "Bool.show(low(U32.add(lazy_word(), 0)))",
        "U32.show(U32.div(lazy_word(), 0))",
        "Bool.show(low(U32.mod(lazy_word(), 0)))",
        "U32.show(U32.shr(lazy_word()))",
        "U32.show(U32.and(0, lazy_word()))",
        "U32.show(U32.and(lazy_word(), 0))",
        "U32.show(lazy_word())",
        "U32.show(U32.mul(0, expensive_whole(U32.to_nat(1000000))))",
        "U32.show(U32.div(expensive_whole(U32.to_nat(1000000)), 0))",
        "U32.show(U32.div(0, lazy_word()))",
        "U32.show(U32.mod(0, lazy_word()))",
    ] {
        let fixture = Fixture::new(&format!(
            "{WORD_HELPERS}{LAZY_WORD}def main() -> IO(Unit): IO.print({expression})\n"
        ));
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let error = fixture
            .checked()
            .run_main(&mut stdout, &mut stderr, &|| false)
            .expect_err("live expensive fields must retain resource limits")
            .to_string();
        assert!(
            error.contains("step budget exhausted"),
            "{expression}: {error}"
        );
        assert!(stdout.is_empty());
        assert!(stderr.is_empty());
    }
}

#[test]
fn pure_word_observation_retains_the_pinned_interpreters_lazy_semantics() {
    assert_output(
        &format!("{WORD_HELPERS}{LAZY_WORD}def main() -> Bool: low(lazy_word())\n"),
        "True{}\n",
    );
}

#[test]
fn live_numeric_fields_fail_before_shallow_observation_without_reordering_effects() {
    let helper = format!(
        r#"{WORD_HELPERS}
def reject(op: IO.OP<Unit>) -> Word(31n):
  match op:
    case Emit{{value}}: Word.zero(31n)
    case rest: Word.zero(31n)
def invalid_word() -> U32:
  U32{{WCon{{True{{}}, reject(IO.print("must not execute")(Unit, value => Emit{{value}}))}}}}
"#
    );
    let fixture = Fixture::new(&format!(
        r#"{helper}
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print("before")
    Unit <- IO.print(Bool.show(low(U32.inc(invalid_word()))))
    IO.print("after")
"#
    ));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = fixture
        .checked()
        .run_main(&mut stdout, &mut stderr, &|| false)
        .unwrap();
    assert_eq!(code, 1);
    assert_eq!(stdout, b"before\n");
    assert_eq!(stderr, b"bend: runtime fail-stop\n");
}

#[test]
fn ready_accumulator_loops_still_obey_work_and_cancellation_limits() {
    let fixture = Fixture::new(
        r"import Base
def count(n: Nat, acc: U32) -> U32:
  match n:
    case 0n: acc
    case 1n+p: count(p, U32.inc(acc))
def main() -> IO(Unit): IO.print(U32.show(count(U32.to_nat(1000000), 0)))
",
    );
    let checked = fixture.checked();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let error = checked
        .run_main(&mut stdout, &mut stderr, &|| false)
        .unwrap_err();
    assert!(
        error.to_string().contains("step budget exhausted"),
        "{error}"
    );
    assert!(stdout.is_empty() && stderr.is_empty());
    let calls = Cell::new(0);
    let error = checked
        .run_main(&mut stdout, &mut stderr, &|| {
            calls.set(calls.get() + 1);
            calls.get() > 200
        })
        .unwrap_err();
    assert!(error.to_string().contains("cancelled"), "{error}");
    assert!(stdout.is_empty() && stderr.is_empty());
}

#[test]
fn packed_binary_operations_keep_first_error_and_output_order() {
    for operation in ["div", "mod", "and"] {
        let helper = format!(
            r#"{WORD_HELPERS}{LAZY_WORD}
def reject(op: IO.OP<Unit>) -> U32:
  match op:
    case Emit{{value}}: 0
    case rest: 0
def invalid() -> U32: reject(IO.print("must not execute")(Unit, value => Emit{{value}}))
"#
        );
        for (left, right, language_failure) in [
            ("invalid()", "expensive_whole(U32.to_nat(1000000))", true),
            ("expensive_whole(U32.to_nat(1000000))", "invalid()", false),
            ("0", "invalid()", true),
        ] {
            let fixture = Fixture::new(&format!(
                r#"{helper}
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print("before")
    Unit <- IO.print(U32.show(U32.{operation}({left}, {right})))
    IO.print("after")
"#
            ));
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let result = fixture
                .checked()
                .run_main(&mut stdout, &mut stderr, &|| false);
            if language_failure {
                assert_eq!(result.unwrap(), 1);
                assert_eq!(stderr, b"bend: runtime fail-stop\n");
            } else {
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains("step budget exhausted")
                );
                assert!(stderr.is_empty());
            }
            assert_eq!(stdout, b"before\n");
        }
    }
}

#[test]
fn packed_division_loops_still_obey_work_and_cancellation_limits() {
    let fixture = Fixture::new(
        r"import Base
def count(n: Nat, acc: U32) -> U32:
  match n:
    case 0n: acc
    case 1n+p: count(p, U32.mod(U32.div(acc, 1), 4294967295))
def main() -> IO(Unit): IO.print(U32.show(count(U32.to_nat(1000000), 7)))
",
    );
    let checked = fixture.checked();
    for cancel in [false, true] {
        let calls = Cell::new(0);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let error = checked
            .run_main(&mut stdout, &mut stderr, &|| {
                calls.set(calls.get() + 1);
                cancel && calls.get() > 200
            })
            .unwrap_err();
        let expected = if cancel {
            "cancelled"
        } else {
            "step budget exhausted"
        };
        assert!(error.to_string().contains(expected), "{error}");
        assert!(stdout.is_empty() && stderr.is_empty());
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
    let code = fixture
        .checked()
        .run_main(&mut stdout, &mut stderr, &|| false)
        .unwrap();
    assert_eq!(code, 1);
    assert!(stdout.is_empty());
    assert_eq!(stderr, b"bend: runtime fail-stop\n");
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

#[test]
fn packed_shift_and_mask_loops_keep_work_depth_and_cancellation_bounds() {
    let tail = "def count(n: Nat, acc: U32) -> U32:\n  match n:\n    case 0n: acc\n    case 1n+p: count(p, U32.and(U32.shr(acc), 4294967295))\n";
    let nested = "def count(n: Nat, acc: U32) -> U32:\n  match n:\n    case 0n: acc\n    case 1n+p: U32.shr(count(p, acc))\n";
    for (body, count, expected) in [
        (tail, 1_000_000, "step budget exhausted"),
        (nested, 5_000, "continuation depth exhausted"),
    ] {
        let fixture = Fixture::new(&format!(
            "import Base\n{body}def main() -> IO(Unit): IO.print(U32.show(count(U32.to_nat({count}), 7)))\n"
        ));
        let checked = fixture.checked();
        for cancel in [false, true] {
            let ticks = Cell::new(0);
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            let error = checked
                .run_main(&mut stdout, &mut stderr, &|| {
                    ticks.set(ticks.get() + 1);
                    cancel && ticks.get() > 200
                })
                .unwrap_err();
            let expected = if cancel { "cancelled" } else { expected };
            assert!(error.to_string().contains(expected), "{error}");
            assert!(stdout.is_empty() && stderr.is_empty());
        }
    }
}
