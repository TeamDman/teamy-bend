// SPDX-License-Identifier: MPL-2.0
//! Pure upstream Word/U32 operations are checked and evaluated without native axioms.
use teamy_bend::kernel::CheckedBook;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

fn library() -> CheckedBook {
    check_book(&parse(include_str!("../src/syntax/base.bend")).expect("Base parses"))
        .expect("Base checks")
}

fn number(value: &TermRef) -> u32 {
    let Term::Ctr { name, args } = value.as_ref() else {
        panic!("expected U32");
    };
    assert_eq!(name, "U32");
    let mut word = &args[0];
    let mut result = 0;
    for bit in 0..32 {
        let Term::Ctr { name, args } = word.as_ref() else {
            panic!("expected WCon");
        };
        assert_eq!(name, "WCon");
        if matches!(args[0].as_ref(),Term::Ctr{name,..} if name=="True") {
            result |= 1 << bit;
        }
        word = &args[1];
    }
    assert!(matches!(word.as_ref(),Term::Ctr{name,args} if name=="WNil" && args.is_empty()));
    result
}

fn evaluate(book: &CheckedBook, name: &str, args: &[&str]) -> TermRef {
    book.evaluate(
        name,
        &args
            .iter()
            .map(|arg| parse_term(arg).expect("literal parses"))
            .collect::<Vec<_>>(),
    )
    .unwrap_or_else(|error| panic!("{name} {args:?}: {error}"))
}

#[test]
fn word_arithmetic_wraps_at_32_bits() {
    let book = library();
    for (name, args, expected) in [
        ("U32.add", ["4294967295", "1"], 0),
        ("U32.add", ["2147483647", "1"], 2_147_483_648),
        ("U32.sub", ["0", "1"], 4_294_967_295),
        ("U32.mul", ["65537", "65537"], 131_073),
        ("U32.mul", ["2147483648", "2"], 0),
        ("U32.and", ["255", "15"], 15),
        ("U32.xor", ["255", "15"], 240),
    ] {
        assert_eq!(
            number(&evaluate(&book, name, &args)),
            expected,
            "{name} {args:?}"
        );
    }
}

#[test]
fn pure_shifts_comparison_and_small_conversion_agree() {
    let book = library();
    assert_eq!(
        number(&evaluate(&book, "U32.shln", &["1", "31n"])),
        2_147_483_648
    );
    assert_eq!(
        number(&evaluate(&book, "U32.shr", &["2147483648"])),
        1_073_741_824
    );
    assert_eq!(number(&evaluate(&book, "U32.from_nat", &["7n"])), 7);
    assert_eq!(evaluate(&book, "U32.to_nat", &["3"]).to_string(), "3n");
    assert_eq!(
        evaluate(&book, "U32.is_gt", &["4294967295", "2147483648"]).to_string(),
        "True{}"
    );
}

#[test]
fn pure_word_division_reduces_without_accumulating_tail_frames() {
    let book = library();
    // This previously hit depth 128 through redundant tail-reduction frames.
    // Pending-depth and work exhaustion remain covered by kernel guard tests.
    assert_eq!(number(&evaluate(&book, "U32.div", &["17", "5"])), 3);
    assert_eq!(number(&evaluate(&book, "U32.mod", &["17", "5"])), 2);
}

#[test]
fn composed_word_arithmetic_preserves_carries_wrapping_and_subtraction() {
    let book = check_book(
        &parse(&format!(
            "{}\n{}",
            include_str!("../src/syntax/base.bend"),
            r"
def mul_add() -> U32: (9 * 10 + 7 : U32)
def mul_sub() -> U32: (9 * 10 - 7 : U32)
def carry() -> U32: (255 * 257 + 1 : U32)
def wrapping() -> U32: (4294967295 * 2 + 3 : U32)
"
        ))
        .unwrap(),
    )
    .unwrap();
    for (name, expected) in [
        ("mul_add", 97),
        ("mul_sub", 83),
        ("carry", 65536),
        ("wrapping", 1),
    ] {
        assert_eq!(number(&evaluate(&book, name, &[])), expected, "{name}");
    }
}

#[test]
fn word_adder_preserves_zero_width_and_unobserved_tail_demand() {
    let book = check_book(
        &parse(&format!(
            "{}\n{}",
            include_str!("../src/syntax/base.bend"),
            r"
def zero_width(f: Bool, c: Bool) -> Word(0n):
  Word.adc(0n, WNil{}, WNil{}, f, c)
def first(w: Word(2n)) -> Bool:
  match w:
    case WCon{bit, tail}: bit
def head_only(t: Word(1n)) -> Bool:
  first(Word.adc(2n, WCon{False{}, t}, WCon{False{}, WCon{False{}, WNil{}}}, False{}, False{}))
"
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        book.normalize_definition("zero_width").unwrap().to_string(),
        "f => c => WNil{}"
    );
    assert_eq!(
        book.normalize_definition("head_only").unwrap().to_string(),
        "t => False{}"
    );
}

#[test]
fn full_adder_formula_has_an_exhaustive_checked_proof() {
    let source = format!(
        "{}{}",
        include_str!("../src/syntax/base.bend"),
        r"
law full_adder_formula:
  for +a: Bool
  for +b: Bool
  for +c: Bool
  {Bool.full_add(a,b,c) ==
    (Bool.xor(Bool.xor(a,b),c), Bool.or(Bool.and(a,b), Bool.and(c,Bool.xor(a,b)))) : Bool & Bool}
def full_adder_formula(a,b,c):
  match a b c:
    case False{} False{} False{}: {==}
    case False{} False{} True{}: {==}
    case False{} True{} False{}: {==}
    case False{} True{} True{}: {==}
    case True{} False{} False{}: {==}
    case True{} False{} True{}: {==}
    case True{} True{} False{}: {==}
    case True{} True{} True{}: {==}
"
    );
    check_book(&parse(&source).expect("formula parses")).expect("all carry/sum cases prove");
}
