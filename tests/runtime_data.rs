// SPDX-License-Identifier: MPL-2.0
use teamy_bend::kernel::CheckedBook;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

const NAT: &str = "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\n";
const ADD: &str = "def add(a: Nat, b: Nat) -> Nat:\n  match a:\n    case Zero{}: b\n    case Succ{p}: Succ{add(p, b)}\n";

fn checked(source: &str) -> CheckedBook {
    check_book(&parse(&format!("{NAT}{source}")).expect("parse runtime fixture"))
        .expect("runtime fixture passes complete proof checking")
}

#[test]
fn repeated_recursive_calls_match_normalization_without_cross_call_state() {
    let book = checked(ADD);
    for (left, right) in [(0, 5), (5, 0), (3, 4), (1, 9), (3, 4)] {
        let args = [
            parse_term(&format!("{left}n")).unwrap(),
            parse_term(&format!("{right}n")).unwrap(),
        ];
        assert_eq!(
            book.evaluate_data("add", &args).unwrap().to_string(),
            book.evaluate("add", &args).unwrap().to_string()
        );
    }
}

#[test]
fn arity_and_argument_types_are_checked_even_when_lazy_body_ignores_them() {
    let book = checked("type Bool is Data:\n  False{}\n  True{}\ndef discard(x: Nat) -> Nat: 0n\n");
    book.evaluate_data("discard", &[]).unwrap_err();
    book.evaluate_data("discard", &[parse_term("True{}").unwrap()])
        .unwrap_err();
    book.evaluate_data("discard", &[parse_term("?hidden").unwrap()])
        .unwrap_err();
    book.evaluate_data("missing", &[]).unwrap_err();
    book.evaluate_data(
        "discard",
        &[parse_term("0n").unwrap(), parse_term("0n").unwrap()],
    )
    .unwrap_err();
}

#[test]
fn lazy_unused_constructor_branch_does_not_materialize_oversized_data() {
    let book = checked(
        "def huge() -> Nat: 97n\ndef discard(x: Nat) -> Nat: 0n\ndef main() -> Nat: discard(huge)\n",
    );
    assert!(
        book.evaluate_data("huge", &[])
            .unwrap_err()
            .to_string()
            .contains("output depth")
    );
    assert_eq!(
        book.evaluate_data("main", &[]).unwrap().to_string(),
        "Zero{}"
    );
    // A failed call cannot poison another call's thunk memoization or budgets.
    assert_eq!(
        book.evaluate_data("discard", &[parse_term("1n").unwrap()])
            .unwrap()
            .to_string(),
        "Zero{}"
    );
}

#[test]
fn data_api_rejects_functions_and_erased_proofs_explicitly() {
    let book = checked(
        "def function() -> @x: Nat -> Nat: x => x\nlaw proof:\n  {0n == 0n : Nat}\ndef proof(): {==}\n",
    );
    assert!(
        book.evaluate_data("function", &[])
            .unwrap_err()
            .to_string()
            .contains("function")
    );
    assert!(
        book.evaluate_data("proof", &[])
            .unwrap_err()
            .to_string()
            .contains("erased")
    );
    book.evaluate("proof", &[]).unwrap();
}

#[test]
fn nested_patterns_and_lexical_shadowing_match_kernel() {
    let book = checked(&format!(
        "{ADD}{}",
        r"
def pick(n: Nat) -> Nat:
  match n:
    case Zero{}: 0n
    case Succ{Zero{}}: 1n
    case Succ{Succ{p}}: p
def main() -> Nat:
  x = {5n : Nat}
  f = {x => add(x, 2n) : @x: Nat -> Nat}
  pick(f(x))
"
    ));
    assert_eq!(
        book.evaluate_data("main", &[]).unwrap().to_string(),
        book.evaluate("main", &[]).unwrap().to_string()
    );
}
