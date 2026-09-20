// SPDX-License-Identifier: MPL-2.0
use teamy_bend::kernel::CheckedBook;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

const SOURCE: &str = r"type Nat is Data: Zero{} Succ{pred: Nat}
type Bool is Data: False{} True{}
type Tree is Data: Leaf{bit: Bool} Branch{left: Tree, right: Tree}

def flip(value: Bool) -> Bool:
  match value:
    case False{}: True{}
    case True{}: False{}

def xor(left: Bool, right: Bool) -> Bool:
  match left:
    case False{}: right
    case True{}: flip(right)

def tree(+n: Nat, +bit: Bool) -> Tree:
  match n:
    case Zero{}: Leaf{bit}
    case Succ{p}: Branch{tree(p, bit), tree(p, flip(bit))}

def fold(value: Tree) -> Bool:
  match value:
    case Leaf{bit}: bit
    case Branch{left, right}: xor(fold(left), fold(right))

def folded(n: Nat) -> Bool: fold(tree(n, True{}))
def built(n: Nat) -> Tree: tree(n, True{})
def built_with(bit: Bool, n: Nat) -> Tree:
  match bit:
    case False{}: tree(n, False{})
    case True{}: tree(n, True{})
def built_after(+n: Nat) -> Tree: built_with(folded(n), n)
def identity(value: Bool) -> Bool: value
";

fn checked() -> CheckedBook {
    check_book(&parse(SOURCE).unwrap()).unwrap()
}

fn natural(depth: usize) -> TermRef {
    parse_term(&format!("{depth}n")).unwrap()
}

#[test]
fn strict_data_fold_reclaims_intermediates_without_numeric_assumptions() {
    let book = checked();
    for depth in [0, 1, 2, 4] {
        let args = [natural(depth)];
        assert_eq!(
            book.evaluate_data("folded", &args).unwrap().to_string(),
            book.evaluate("folded", &args).unwrap().to_string()
        );
    }
    // This 4,096-leaf fold exhausted the append-only thunk arena. The model
    // declares its own ordinary datatypes and has no executable Base origins.
    assert_eq!(
        book.evaluate_data("folded", &[natural(12)])
            .unwrap()
            .to_string(),
        "False{}"
    );
}

fn assert_tree(value: &TermRef, depth: usize, expected: bool) -> usize {
    let Term::Ctr { name, args } = value.as_ref() else {
        panic!("materialized data must contain constructors");
    };
    if depth == 0 {
        assert_eq!(name, "Leaf");
        assert_eq!(args.len(), 1);
        let Term::Ctr { name, args } = args[0].as_ref() else {
            panic!("leaf must contain a Boolean constructor");
        };
        assert_eq!(name, if expected { "True" } else { "False" });
        assert!(args.is_empty());
        2
    } else {
        assert_eq!(name, "Branch");
        assert_eq!(args.len(), 2);
        1 + assert_tree(&args[0], depth - 1, expected) + assert_tree(&args[1], depth - 1, !expected)
    }
}

#[test]
fn pending_siblings_survive_collection_during_strict_data_materialization() {
    let book = checked();
    let value = book.evaluate_data("built_after", &[natural(12)]).unwrap();
    // Independently inspect every path: choosing a right branch flips its bit.
    // The preceding fold forces allocation pressure before output begins.
    assert_eq!(assert_tree(&value, 12, false), 12_287);
}

#[test]
fn output_limit_failure_does_not_poison_the_next_typed_call() {
    let book = checked();
    let error = book
        .evaluate_data("built", &[natural(13)])
        .unwrap_err()
        .to_string();
    assert!(error.contains("output depth or node budget"), "{error}");
    assert_eq!(
        book.evaluate_data("identity", &[parse_term("True{}").unwrap()])
            .unwrap()
            .to_string(),
        "True{}"
    );
    assert_eq!(
        book.evaluate_data("folded", &[natural(12)])
            .unwrap()
            .to_string(),
        "False{}"
    );
}
