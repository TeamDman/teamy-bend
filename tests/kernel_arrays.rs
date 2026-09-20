// SPDX-License-Identifier: MPL-2.0
//! Ordinary checked Array operations preserve elements and affine ownership.

use teamy_bend::kernel::CheckedBook;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

const BASE: &str = include_str!("../src/syntax/base.bend");
const FIXTURES: &str = r"
def sample() -> Array<Nat>:
  ANode{ANode{ALeaf{1n}, ALeaf{2n}}, ANode{ALeaf{3n}, ALeaf{4n}}}

def get_result(result: Array<Nat> & Nat) -> List<Nat> & Nat:
  (array, value) = result
  (Array.to_list(~Nat, array), value)

def size_result(result: Array<Nat> & U32) -> List<Nat> & Nat:
  (array, size) = result
  (Array.to_list(~Nat, array), U32.to_nat(size))

def clone_result(result: Array<Nat> & Array<Nat>) -> List<Nat> & List<Nat>:
  (left, right) = result
  (Array.to_list(~Nat, Array.set(Nat, left, 1, 9n)), Array.to_list(~Nat, right))

def size() -> List<Nat> & Nat: size_result(Array.size(Nat, sample()))
def one() -> List<Nat>: Array.to_list(~Nat, Array.new(Nat, 0n, 7n))
def repeated() -> List<Nat>: Array.to_list(~Nat, [7n: Nat * 4n])
def wrapped_get() -> List<Nat> & Nat: get_result(Array.get(Nat, sample(), 5))
def highest_get() -> List<Nat> & Nat: get_result(Array.get(Nat, sample(), 4294967295))
def single_get() -> List<Nat> & Nat: get_result(Array.get(Nat, ALeaf{6n}, 4294967295))
def swap_low() -> List<Nat> & Nat: get_result(Array.swap(Nat, sample(), 0, 8n))
def swap_high() -> List<Nat> & Nat: get_result(Array.swap(Nat, sample(), 3, 8n))
def wrapped_set() -> List<Nat>: Array.to_list(~Nat, Array.set(Nat, sample(), 6, 9n))
def cloned() -> List<Nat> & List<Nat>: clone_result(Array.clone(Nat, sample()))
def increment(x: Nat) -> Nat: 1n+x
def mapped() -> List<Nat>: Array.to_list(~Nat, Array.map(~Nat, ~Nat, ~increment, sample()))

def sequential_writes(a: Array<U32>) -> List<Nat>:
  a[0] <- 8
  a[5] <- 9
  Array.to_list(~Nat, Array.map(~U32, ~Nat, ~U32.to_nat, a))

def written() -> List<Nat>: sequential_writes([0: U32 * 4n])

type Ticket is Type:
  Ticket{number: Nat}

def tickets() -> Array<Ticket>: ANode{ALeaf{Ticket{1n}}, ALeaf{Ticket{2n}}}
def ticket(number: Nat) -> Ticket: Ticket{number}
def ticket_result(result: Array<Ticket> & Ticket) -> List<Ticket> & Ticket:
  (array, old) = result
  (Array.to_list(~Ticket, array), old)
def swap_affine() -> List<Ticket> & Ticket:
  ticket_result(Array.swap(Ticket, tickets(), 1, Ticket{3n}))
def map_affine() -> List<Ticket>:
  Array.to_list(~Ticket, Array.map(~Nat, ~Ticket, ~ticket, sample()))
";

fn checked() -> CheckedBook {
    let source = format!("{BASE}\n{FIXTURES}");
    let book = parse(&source).expect("Array library and fixtures parse");
    check_book(&book).expect("Array declarations and instances pass ordinary checking")
}

fn expect(program: &CheckedBook, entry: &str, expected: &str) {
    let actual = program
        .evaluate_data(entry, &[])
        .expect("Array fixture evaluates");
    let expected = parse_term(expected).expect("expected constructor tree parses");
    assert_eq!(actual.to_string(), expected.to_string(), "{entry}");
}

#[test]
fn arrays_create_power_of_two_storage_and_size_preserves_contents() {
    let program = checked();
    for (entry, expected) in [
        ("size", "([1n,2n,3n,4n],4n)"),
        ("one", "[7n]"),
        ("repeated", "[7n,7n,7n,7n]"),
    ] {
        expect(&program, entry, expected);
    }
}

#[test]
fn array_indices_wrap_and_swaps_return_displaced_values() {
    let program = checked();
    for (entry, expected) in [
        ("wrapped_get", "([1n,2n,3n,4n],2n)"),
        ("highest_get", "([1n,2n,3n,4n],4n)"),
        ("single_get", "([6n],6n)"),
        ("swap_low", "([8n,2n,3n,4n],1n)"),
        ("swap_high", "([1n,2n,3n,8n],4n)"),
        ("wrapped_set", "[1n,2n,9n,4n]"),
    ] {
        expect(&program, entry, expected);
    }
}

#[test]
fn clone_map_and_sequential_write_syntax_preserve_array_order() {
    let program = checked();
    for (entry, expected) in [
        ("cloned", "([1n,9n,3n,4n],[1n,2n,3n,4n])"),
        ("mapped", "[2n,3n,4n,5n]"),
        ("written", "[8n,9n,0n,0n]"),
    ] {
        expect(&program, entry, expected);
    }
}

#[test]
fn arrays_move_affine_elements_without_copying() {
    let program = checked();
    for (entry, expected) in [
        ("swap_affine", "([Ticket{1n},Ticket{3n}],Ticket{2n})"),
        (
            "map_affine",
            "[Ticket{1n},Ticket{2n},Ticket{3n},Ticket{4n}]",
        ),
    ] {
        expect(&program, entry, expected);
    }
}

#[test]
fn array_operations_reject_affine_copying_and_false_equations() {
    for (source, diagnostic) in [
        (
            "def invalid() -> Array<Ticket> & Ticket: Array.get(Ticket, tickets(), 0)\n",
            "type mismatch",
        ),
        (
            "def invalid() -> Array<Ticket> & Array<Ticket>: Array.clone(Ticket, tickets())\n",
            "type mismatch",
        ),
        (
            "def invalid() -> Array<Ticket>: Array.new(Ticket, 1n, Ticket{1n})\n",
            "type mismatch",
        ),
        (
            "def invalid(a: Array<Nat>) -> Array<Nat> & Array<Nat>: (a, a)\n",
            "Many",
        ),
        (
            "type TicketPair is Type:\n  TicketPair{left: Ticket, right: Ticket}\ndef invalid() -> Array<TicketPair>: Array.map(~Ticket, ~TicketPair, ~(x => TicketPair{x,x}), tickets())\n",
            "Many",
        ),
        (
            "law invalid: {Array.new(Nat, 0n, 1n) == ALeaf{2n} : Array<Nat>}\ndef invalid(): {==}\n",
            "reflexivity",
        ),
    ] {
        let book =
            parse(&format!("{BASE}\n{FIXTURES}\n{source}")).expect("negative control parses");
        let error = check_book(&book).expect_err("invalid array control must fail checking");
        assert!(error.to_string().contains(diagnostic), "{error}");
    }
}
