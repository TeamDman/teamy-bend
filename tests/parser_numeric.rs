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
    assert_eq!(
        evaluate(&book, "U32.to_nat", &["3"]).to_string(),
        "Succ{Succ{Succ{Zero{}}}}"
    );
    assert_eq!(
        evaluate(&book, "U32.is_gt", &["4294967295", "2147483648"]).to_string(),
        "True{}"
    );
}

#[test]
fn deep_pure_word_division_reports_resource_exhaustion() {
    let book = library();
    let args = [
        parse_term("17").expect("literal"),
        parse_term("5").expect("literal"),
    ];
    let error = book
        .evaluate("U32.div", &args)
        .expect_err("recursive divider exceeds current kernel limit");
    assert!(error.to_string().contains("nesting limit"));
}

#[test]
fn inlined_full_adder_formula_has_an_exhaustive_checked_proof() {
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
