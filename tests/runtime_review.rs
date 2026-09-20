// SPDX-License-Identifier: MPL-2.0
use std::rc::Rc;
use teamy_bend::kernel::CheckedBook;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

const NAT: &str = "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\n";

fn checked(source: &str) -> CheckedBook {
    check_book(&parse(&format!("{NAT}{source}")).expect("parse review fixture"))
        .expect("review fixture passes complete checking")
}

#[test]
fn dependent_erased_arguments_are_checked_before_lazy_execution() {
    let book = checked(
        r"
type Bool is Data:
  False{}
  True{}
def identity(-A: Data, value: A) -> A: value
def witnessed(-n: Nat, -proof: {n == 0n : Nat}) -> Nat: 0n
",
    );
    let nat = parse_term("Nat").expect("datatype");
    let zero = parse_term("0n").expect("natural");
    let one = parse_term("1n").expect("natural");
    let truth = parse_term("True{}").expect("Boolean");
    let proof = parse_term("{==}").expect("reflexivity");
    assert_eq!(
        book.evaluate_data("identity", &[Rc::clone(&nat), Rc::clone(&one)])
            .expect("dependent identity")
            .to_string(),
        "Succ{Zero{}}"
    );
    book.evaluate_data("identity", &[nat, truth])
        .expect_err("second argument must inhabit the supplied first argument");
    book.evaluate_data("witnessed", &[one, Rc::clone(&proof)])
        .expect_err("erased proof still has to prove the dependent claim");
    assert_eq!(
        book.evaluate_data("witnessed", &[zero, proof])
            .expect("valid proof after failed dependent calls")
            .to_string(),
        "Zero{}"
    );
}

#[test]
fn proof_fields_can_be_discarded_but_cannot_escape_as_data() {
    let book = checked(
        r"
type Evidence is Data:
  Evidence{-proof: {0n == 0n : Nat}, value: Nat}
def evidence() -> Evidence: Evidence{{==}, 1n}
def extract(record: Evidence) -> Nat:
  match record:
    case Evidence{proof, value}: value
def main() -> Nat: extract(evidence)
",
    );
    book.evaluate_data("evidence", &[])
        .expect_err("proof fields cannot be serialized as data");
    assert_eq!(
        book.evaluate_data("main", &[])
            .expect("runtime ignores erased evidence")
            .to_string(),
        book.evaluate("main", &[])
            .expect("kernel reference")
            .to_string()
    );
}

#[test]
fn functions_nested_in_constructor_fields_are_not_data_results() {
    let book = checked(
        r"
type FunctionBox is Type:
  FunctionBox{callback: @x: Nat -> Nat}
def boxed() -> FunctionBox: FunctionBox{x => x}
",
    );
    assert!(
        book.evaluate_data("boxed", &[])
            .expect_err("a constructor shell cannot make a function into data")
            .to_string()
            .contains("function")
    );
}

#[test]
fn shared_trees_obey_materialized_node_budget_and_reset_after_failure() {
    let book = checked(
        r"
type Tree is Data:
  Leaf{}
  Node{left: Tree, right: Tree}
def grow(n: Nat) -> Tree:
  match n:
    case Zero{}: Leaf{}
    case Succ{p}:
      +sub = {grow(p): Tree}
      Node{sub, sub}
",
    );
    let thirteen = parse_term("13n").expect("natural");
    let fourteen = parse_term("14n").expect("natural");
    let result = book
        .evaluate_data("grow", &[thirteen])
        .expect("16,383 materialized nodes fit the output budget");
    assert!(matches!(result.as_ref(), Term::Ctr { name, .. } if name == "Node"));
    assert!(
        book.evaluate_data("grow", &[fourteen])
            .expect_err("32,767 materialized nodes exceed the output budget")
            .to_string()
            .contains("node budget")
    );
    assert_eq!(
        book.evaluate_data("grow", &[parse_term("0n").expect("natural")])
            .expect("a failed call leaves no memoized state or consumed budget")
            .to_string(),
        "Leaf{}"
    );
}

#[test]
fn exponential_work_stops_at_step_budget_without_poisoning_next_call() {
    use std::fmt::Write;

    let mut source = String::from(
        r"
def sequence(first: Nat, second: Nat) -> Nat:
  match first:
    case Zero{}: second
    case Succ{p}: second
def work0(n: Nat) -> Nat: n
",
    );
    for index in 1..=17 {
        writeln!(
            source,
            "def work{index}(+n: Nat) -> Nat: sequence(work{0}(n), work{0}(n))",
            index - 1
        )
        .expect("append definition");
    }
    let book = checked(&source);
    assert!(
        book.evaluate_data("work17", &[parse_term("0n").expect("natural")])
            .expect_err("reclamation permits more allocation, but total work remains bounded")
            .to_string()
            .contains("step budget exhausted")
    );
    assert_eq!(
        book.evaluate_data("work1", &[parse_term("0n").expect("natural")])
            .expect("subsequent small call")
            .to_string(),
        "Zero{}"
    );
}

#[test]
fn long_checked_definition_chain_stops_at_continuation_budget() {
    use std::fmt::Write;

    let mut source = String::from("def step0(n: Nat) -> Nat: n\n");
    for index in 1..=2200 {
        writeln!(
            source,
            "def step{index}(n: Nat) -> Nat: step{}(n)",
            index - 1
        )
        .expect("append definition");
    }
    let book = checked(&source);
    assert!(
        book.evaluate_data("step2200", &[parse_term("0n").expect("natural")])
            .expect_err("checked function chain exceeds bounded continuations")
            .to_string()
            .contains("continuation depth exhausted")
    );
    assert_eq!(
        book.evaluate_data("step1", &[parse_term("0n").expect("natural")])
            .expect("subsequent small call")
            .to_string(),
        "Zero{}"
    );
}
