// SPDX-License-Identifier: MPL-2.0
//! Tests that the selected upstream library is checked as ordinary Bend code.
use teamy_bend::kernel::CheckedBook;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::parse;

const BASE: &str = include_str!("../src/syntax/base.bend");

fn checked(source: &str) -> CheckedBook {
    let book = parse(&format!("{BASE}\n{source}")).expect("library and program parse");
    check_book(&book).expect("library and program type check")
}

#[test]
fn boolean_law_is_proved_for_every_constructor_pair() {
    let program = checked(
        r"
law both_commute:
  for +a: Bool
  for +b: Bool
  {Bool.and(a, b) == Bool.and(b, a) : Bool}
def both_commute(a, b):
  match a b:
    case False{} False{}: {==}
    case False{} True{}: {==}
    case True{} False{}: {==}
    case True{} True{}: {==}

law negate_twice:
  for b: Bool
  {Bool.not(Bool.not(b)) == b : Bool}
def negate_twice(b):
  match b:
    case False{}: {==}
    case True{}: {==}

def main() -> {False{} == False{} : Bool}:
  negate_twice(False{})

",
    );
    assert_eq!(
        program
            .evaluate("main", &[])
            .expect("proof computes")
            .to_string(),
        "{==}"
    );
}

#[test]
fn equality_library_transports_open_hypotheses() {
    checked(
        r"
law flip_then_chain:
  for -a: Nat
  for -b: Nat
  for e: {a == b : Nat}
  {Succ{b} == Succ{a} : Nat}
def flip_then_chain(a, b, e):
  Equal.cong(Nat, Nat, n => Succ{n}, b, a, Equal.sym(Nat, a, b, e))

",
    );
}

#[test]
fn dependent_pair_carries_a_checked_existential_witness() {
    let program = checked(
        r"
law find_two:
  exs n: Nat
  {Nat.add(n, 1n) == 3n : Nat}
def find_two():
  (2n, {==})

",
    );
    assert_eq!(
        program
            .evaluate("find_two", &[])
            .expect("existential proof computes")
            .to_string(),
        "Tuple{Succ{Succ{Zero{}}}, {==}}"
    );
}

#[test]
fn polymorphic_list_helpers_preserve_values() {
    let program = checked(
        r"
law reversed:
  {List.reverse(&2, Nat, [1n, 2n, 3n]) == [3n, 2n, 1n] : List<&2, Nat>}
def reversed(): {==}

def main() -> Nat:
  List.length(&2, Nat, List.append(&2, Nat, [1n, 2n], [3n]))

",
    );
    assert_eq!(
        program
            .evaluate("reversed", &[])
            .expect("list law computes")
            .to_string(),
        "{==}"
    );
    assert_eq!(
        program
            .evaluate("main", &[])
            .expect("list length computes")
            .to_string(),
        "Succ{Succ{Succ{Zero{}}}}"
    );
}

#[test]
fn structural_division_returns_quotient_and_remainder() {
    let program = checked("def main() -> Nat & Nat:\n  Nat.divmod(7n, 3n)\n");
    assert_eq!(
        program
            .evaluate("main", &[])
            .expect("division computes")
            .to_string(),
        "Tuple{Succ{Succ{Zero{}}}, Succ{Zero{}}}"
    );
}

#[test]
fn library_equations_do_not_accept_false_results() {
    let book = parse(&format!(
        "{BASE}\n law wrong:\n  {{Nat.mul(2n, 3n) == 5n : Nat}}\ndef wrong(): {{==}}\n"
    ))
    .expect("false claim parses");
    let error =
        check_book(&book).expect_err("ordinary library evaluation must reject a false theorem");
    assert!(error.to_string().contains("reflexivity"));
}
