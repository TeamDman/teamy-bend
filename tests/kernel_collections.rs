// SPDX-License-Identifier: MPL-2.0
//! Finite, executable expectations for the ordinary checked Base collections.

use teamy_bend::kernel::CheckedBook;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

const BASE: &str = include_str!("../src/syntax/base.bend");

fn checked(source: &str) -> CheckedBook {
    let book = parse(&format!("{BASE}\n{source}")).expect("collection fixture parses");
    check_book(&book).expect("complete Base and fixture pass strict checking")
}

fn expect(program: &CheckedBook, entry: &str, expected: &str) {
    let actual = program
        .evaluate_data(entry, &[])
        .expect("collection fixture evaluates");
    let expected = parse_term(expected).expect("expected constructor tree parses");
    assert_eq!(actual.to_string(), expected.to_string(), "{entry}");
}

#[test]
fn list_index_slice_zip_and_pure_sort_helpers_preserve_finite_values() {
    let program = checked(
        r"
def out_of_bounds() -> Maybe<&2, Nat>: List.get(&2, Nat, [1n, 2n], 3n)
def unchanged() -> List<&2, Nat>: List.set(&2, Nat, [1n, 2n], 3n, 9n)
def taken() -> List<&2, Nat>: List.take(&2, Nat, [1n, 2n], 5n)
def dropped() -> List<&2, Nat>: List.drop(&2, Nat, [1n, 2n], 5n)
def zipped() -> List<&1, Nat & Nat>: List.zip(&2, Nat, &2, Nat, [1n, 2n, 3n], [4n, 5n])
def singleton_runs() -> List<&2, List<&2, Nat>>: List.sort.runs(Nat, [3n, 1n, 2n])
def kept_find() -> Maybe<&2, Nat>: List.find.put(Nat, 7n, Some{8n}, True{})
def skipped_find() -> Maybe<&2, Nat>: List.find.put(Nat, 7n, Some{8n}, False{})
def left_merge() -> List<&2, Nat> & List<&2, Nat> & List<&2, Nat>:
  List.merge.step(Nat, [0n], 1n, [3n], 2n, [4n], True{})
def right_merge() -> List<&2, Nat> & List<&2, Nat> & List<&2, Nat>:
  List.merge.step(Nat, [0n], 1n, [3n], 2n, [4n], False{})
",
    );
    for (entry, expected) in [
        ("out_of_bounds", "None{}"),
        ("unchanged", "[1n, 2n]"),
        ("taken", "[1n, 2n]"),
        ("dropped", "[]"),
        ("zipped", "[(1n,4n),(2n,5n)]"),
        ("singleton_runs", "[[3n],[1n],[2n]]"),
        ("kept_find", "Some{7n}"),
        ("skipped_find", "Some{8n}"),
        ("left_merge", "([1n,0n],[3n],[2n,4n])"),
        ("right_merge", "([2n,0n],[1n,3n],[4n])"),
    ] {
        expect(&program, entry, expected);
    }
}

#[test]
fn string_comparison_preserves_operands_and_orders_prefixes() {
    let program = checked(
        r#"
def prefix() -> (String & String) & Cmp: String.cmp("ab", "abc")
def greater() -> Cmp: String.order("b", "abc")
def empty_first() -> Cmp: String.order("", "a")
def identical() -> Bool: String.eq("ab", "ab")
def different() -> Bool: String.eq("a", "ab")
def character() -> Bool: Char.is_eq('a', 'a')
"#,
    );
    for (entry, expected) in [
        ("prefix", r#"(("ab","abc"),LT{})"#),
        ("greater", "GT{}"),
        ("empty_first", "LT{}"),
        ("identical", "True{}"),
        ("different", "False{}"),
        ("character", "True{}"),
    ] {
        expect(&program, entry, expected);
    }
}

const MAP: &str = r#"
def sample() -> Map<&2, Nat>:
  Map.set(&2, Nat, Map.set(&2, Nat, Map.set(&2, Nat, Map.new(&2, Nat), "a", 1n), "b", 2n), "a", 5n)
def read_value(r: Map<&2, Nat> & Nat) -> Nat:
  (m, x) = r
  x
def read_found(r: Map<&2, Nat> & Bool) -> Bool:
  (m, found) = r
  found
def found() -> Nat: read_value(Map.get(Nat, 0n, sample(), "a"))
def absent() -> Nat: read_value(Map.get(Nat, 9n, sample(), "zz"))
def has_b() -> Bool: read_found(Map.has(&2, Nat, sample(), "b"))
def has_c() -> Bool: read_found(Map.has(&2, Nat, sample(), "c"))
def size() -> Nat: Map.size(&2, Nat, sample())
def deleted() -> List<&2, Nat>: Map.values(&2, Nat, Map.del(&2, Nat, sample(), "a"))
def missing_delete() -> List<&2, Nat>: Map.values(&2, Nat, Map.del(&2, Nat, sample(), "z"))
def popped() -> Maybe<&2, Nat>:
  pop_value(Map.pop(&2, Nat, sample(), "a"))
"#;

#[test]
fn patricia_map_updates_preserves_reads_and_handles_missing_keys() {
    let program = checked(&format!(
        r#"
def pop_value(r: Map<&2, Nat> & Maybe<&2, Nat>) -> Maybe<&2, Nat>:
  (m, value) = r
  value
{MAP}
def empty_size() -> Nat: Map.size(&2, Nat, Map.new(&2, Nat))
def empty_get() -> Nat: read_value(Map.get(Nat, 7n, Map.new(&2, Nat), ""))
"#
    ));
    for (entry, expected) in [
        ("found", "5n"),
        ("absent", "9n"),
        ("has_b", "True{}"),
        ("has_c", "False{}"),
        ("size", "2n"),
        ("deleted", "[2n]"),
        ("missing_delete", "[5n,2n]"),
        ("popped", "Some{5n}"),
        ("empty_size", "0n"),
        ("empty_get", "7n"),
    ] {
        expect(&program, entry, expected);
    }
}

#[test]
fn patricia_map_prefix_keys_sort_and_union_is_right_biased() {
    let program = checked(
        r#"
def first() -> Map<&2, Nat>:
  Map.from_list(&2, Nat, [("ab",1n),("a",2n),("b",3n)])
def second() -> Map<&2, Nat>:
  Map.from_list(&2, Nat, [("a",4n),("c",5n)])
def listed() -> List<&2, Sigma<&2, &2, String, _ => Nat>>:
  Map.to_list(&2, Nat, first())
def forward() -> List<&2, Nat>: Map.values(&2, Nat, Map.union(&2, Nat, first(), second()))
def backward() -> List<&2, Nat>: Map.values(&2, Nat, Map.union(&2, Nat, second(), first()))
def keys() -> List<&2, String>: Map.keys(&2, Nat, Map.set(&2, Nat, first(), "", 9n))
def duplicates() -> List<&2, Nat>: Map.values(&2, Nat, Map.from_list(&2, Nat, [("a",1n),("a",6n)]))
"#,
    );
    for (entry, expected) in [
        ("listed", r#"[("a",2n),("ab",1n),("b",3n)]"#),
        ("forward", "[4n,1n,3n,5n]"),
        ("backward", "[2n,1n,3n,5n]"),
        ("keys", r#"["","a","ab","b"]"#),
        ("duplicates", "[6n]"),
    ] {
        expect(&program, entry, expected);
    }
}

#[test]
fn collection_equations_reject_false_lookup_and_slice_results() {
    for source in [
        "law wrong:\n  {List.get(&2, Nat, [1n,2n], 1n) == Some{1n} : Maybe<&2,Nat>}\ndef wrong(): {==}\n",
        "law wrong:\n  {Map.size(&2, Nat, Map.new(&2, Nat)) == 1n : Nat}\ndef wrong(): {==}\n",
    ] {
        let book = parse(&format!("{BASE}\n{source}")).unwrap();
        let error = check_book(&book).expect_err("wrong collection result must not prove");
        assert!(error.to_string().contains("reflexivity"));
    }
}

#[test]
fn string_search_split_and_ascii_case_helpers_preserve_boundary_behavior() {
    let program = checked(
        r#"
def prefix_hit() -> Bool: String.starts_with("abc", "ab")
def prefix_miss() -> Bool: String.starts_with("abc", "ac")
def suffix_hit() -> Bool: String.ends_with("abc", "bc")
def suffix_longer() -> Bool: String.ends_with("abc", "abcd")
def middle_hit() -> Bool: String.contains("ababc", "abc")
def middle_miss() -> Bool: String.contains("abc", "bd")
def empty_needle() -> Bool: String.contains("", "")
def split_edges() -> List<&2, String>: String.split(",a,,b,", ',')
def split_empty() -> List<&2, String>: String.split("", ',')
def lines() -> List<&2, String>: String.lines("a\nb\n")
def trimmed() -> String: String.trim(" \tAb\n ")
def spaces() -> String: String.trim(" \t\n\r")
def upper() -> String: String.to_upper("aZ09 é")
def lower() -> String: String.to_lower("Az09 É")
def ascii_letters() -> Bool: Bool.and(Char.is_alpha('a'), Char.is_alpha('Z'))
def ascii_digits() -> Bool: Bool.and(Char.is_digit('0'), Char.is_digit('9'))
def outside_digit() -> Bool: Bool.or(Char.is_digit('/'), Char.is_digit(':'))
def non_ascii_letter() -> Bool: Char.is_alpha('é')
"#,
    );
    for (entry, expected) in [
        ("prefix_hit", "True{}"),
        ("prefix_miss", "False{}"),
        ("suffix_hit", "True{}"),
        ("suffix_longer", "False{}"),
        ("middle_hit", "True{}"),
        ("middle_miss", "False{}"),
        ("empty_needle", "True{}"),
        ("split_edges", r#"["","a","","b",""]"#),
        ("split_empty", r#"[""]"#),
        ("lines", r#"["a","b",""]"#),
        ("trimmed", r#""Ab""#),
        ("spaces", r#""""#),
        ("upper", r#""AZ09 é""#),
        ("lower", r#""az09 É""#),
        ("ascii_letters", "True{}"),
        ("ascii_digits", "True{}"),
        ("outside_digit", "False{}"),
        ("non_ascii_letter", "False{}"),
    ] {
        expect(&program, entry, expected);
    }
}

#[test]
fn list_templates_specialize_closed_functions_and_preserve_fold_direction() {
    let program = checked(
        r#"
def increment(x: Nat) -> Nat: 1n+x
def above_one(x: Nat) -> Bool: Nat.is_gt(x, 1n)
def mapped() -> List<Nat>: List.map(~Nat, ~Nat, ~increment, [1n,2n,3n])
def filtered() -> List<&2, Nat>: List.filter(~Nat, ~above_one, [1n,3n,2n,0n])
def fold_left() -> String:
  List.foldl(~&2, ~String, ~String, ~(acc => x => String.append(acc, x)), ["a","b","c"], "")
def fold_right() -> String:
  List.foldr(~&2, ~String, ~String, ~(x => acc => String.append(acc, x)), ["a","b","c"], "")
def any_empty() -> Bool: List.any(~&2, ~Nat, ~above_one, [])
def all_empty() -> Bool: List.all(~&2, ~Nat, ~above_one, [])
def any_hit() -> Bool: List.any(~&2, ~Nat, ~above_one, [0n,2n])
def all_miss() -> Bool: List.all(~&2, ~Nat, ~above_one, [2n,0n])
def first_found() -> Maybe<&2, Nat>: List.find(~Nat, ~above_one, [0n,3n,2n])
def not_found() -> Maybe<&2, Nat>: List.find(~Nat, ~above_one, [0n,1n])
def contains_hit() -> Bool: List.contains(~Nat, ~Nat.is_eq, [1n,2n,3n], 2n)
def contains_miss() -> Bool: List.contains(~Nat, ~Nat.is_eq, [1n,2n,3n], 4n)
def rendered() -> String: List.show(~&2, ~String, ~(s => s), ["a","b","c"])
"#,
    );
    for (entry, expected) in [
        ("mapped", "[2n,3n,4n]"),
        ("filtered", "[3n,2n]"),
        ("fold_left", r#""abc""#),
        ("fold_right", r#""cba""#),
        ("any_empty", "False{}"),
        ("all_empty", "True{}"),
        ("any_hit", "True{}"),
        ("all_miss", "False{}"),
        ("first_found", "Some{3n}"),
        ("not_found", "None{}"),
        ("contains_hit", "True{}"),
        ("contains_miss", "False{}"),
        ("rendered", r#""[a, b, c]""#),
    ] {
        expect(&program, entry, expected);
    }
}

#[test]
fn list_merge_sort_preserves_duplicates_and_handles_empty_input() {
    let program = checked(
        r"
def sorted() -> List<&2, Nat>: List.sort(~Nat, ~Nat.is_le, [3n,1n,2n,1n,0n])
def descending() -> List<&2, Nat>: List.sort(~Nat, ~Nat.is_ge, [3n,1n,2n,1n,0n])
def empty() -> List<&2, Nat>: List.sort(~Nat, ~Nat.is_le, [])
def single() -> List<&2, Nat>: List.sort(~Nat, ~Nat.is_le, [4n])

type Item is Data:
  Item{key: Nat, tag: Nat}
def key_le(a: Item, b: Item) -> Bool:
  match a b:
    case Item{x, tx} Item{y, ty}: Nat.is_le(x, y)
def stable() -> List<&2, Item>:
  List.sort(~Item, ~key_le, [Item{2n,0n},Item{1n,1n},Item{2n,2n},Item{1n,3n}])

",
    );
    for (entry, expected) in [
        ("sorted", "[0n,1n,1n,2n,3n]"),
        ("descending", "[3n,2n,1n,1n,0n]"),
        ("empty", "[]"),
        ("single", "[4n]"),
        (
            "stable",
            "[Item{1n,1n},Item{1n,3n},Item{2n,0n},Item{2n,2n}]",
        ),
    ] {
        expect(&program, entry, expected);
    }
}

#[test]
fn map_reads_preserve_the_map_and_deletions_collapse_empty_branches() {
    let program = checked(
        r#"
def sample() -> Map<&2, Nat>:
  Map.from_list(&2, Nat, [("a",1n),("ab",2n),("b",3n)])
def read_result(result: Map<&2, Nat> & Nat) -> List<&2, Nat> & Nat:
  (m, value) = result
  (Map.values(&2, Nat, m), value)
def has_result(result: Map<&2, Nat> & Bool) -> List<&2, Nat> & Bool:
  (m, found) = result
  (Map.values(&2, Nat, m), found)
def pop_result(result: Map<&2, Nat> & Maybe<&2, Nat>) -> List<&2, Nat> & Maybe<&2, Nat>:
  (m, value) = result
  (Map.values(&2, Nat, m), value)
def read() -> List<&2, Nat> & Nat: read_result(Map.get(Nat, 9n, sample(), "ab"))
def absent_read() -> List<&2, Nat> & Nat: read_result(Map.get(Nat, 9n, sample(), "z"))
def has() -> List<&2, Nat> & Bool: has_result(Map.has(&2, Nat, sample(), "a"))
def pop() -> List<&2, Nat> & Maybe<&2, Nat>: pop_result(Map.pop(&2, Nat, sample(), "ab"))
def absent_pop() -> List<&2, Nat> & Maybe<&2, Nat>:
  pop_result(Map.pop(&2, Nat, sample(), "z"))
def last() -> Map<&2, Nat>:
  Map.del(&2, Nat, Map.del(&2, Nat, sample(), "a"), "b")
def all_deleted() -> Map<&2, Nat>:
  Map.del(&2, Nat, last(), "ab")
def empty_key_and_unicode() -> List<&2, Nat>:
  Map.values(&2, Nat, Map.from_list(&2, Nat, [("é",3n),("",1n),("a",2n),("é",4n)]))
"#,
    );
    for (entry, expected) in [
        ("read", "([1n,2n,3n],2n)"),
        ("absent_read", "([1n,2n,3n],9n)"),
        ("has", "([1n,2n,3n],True{})"),
        ("pop", "([1n,3n],Some{2n})"),
        ("absent_pop", "([1n,2n,3n],None{})"),
        ("last", r#"MLeaf{"ab",2n}"#),
        ("all_deleted", "MTip{}"),
        ("empty_key_and_unicode", "[1n,2n,4n]"),
    ] {
        expect(&program, entry, expected);
    }
}

#[test]
fn maps_and_list_templates_preserve_affine_element_types() {
    let declarations = r#"
type Ticket is Type:
  Ticket{number: Nat}
def ticket(number: Nat) -> Ticket: Ticket{number}
def sample() -> Map<&1, Ticket>:
  Map.from_list(&1, Ticket, [("b",Ticket{2n}),("a",Ticket{1n})])
"#;
    let program = checked(&format!(
        r#"
{declarations}
def read_result(result: Map<&1, Ticket> & Bool) -> List<&1, Ticket> & Bool:
  (m, found) = result
  (Map.values(&1, Ticket, m), found)
def pop_result(result: Map<&1, Ticket> & Maybe<&1, Ticket>) -> List<&1, Ticket> & Maybe<&1, Ticket>:
  (m, value) = result
  (Map.values(&1, Ticket, m), value)
def read_without_copying() -> List<&1, Ticket> & Bool:
  read_result(Map.has(&1, Ticket, sample(), "a"))
def pop_affine() -> List<&1, Ticket> & Maybe<&1, Ticket>:
  pop_result(Map.pop(&1, Ticket, sample(), "a"))
def mapped() -> List<Ticket>: List.map(~Nat, ~Ticket, ~ticket, [1n,2n])
"#
    ));
    for (entry, expected) in [
        ("read_without_copying", "([Ticket{1n},Ticket{2n}],True{})"),
        ("pop_affine", "([Ticket{2n}],Some{Ticket{1n}})"),
        ("mapped", "[Ticket{1n},Ticket{2n}]"),
    ] {
        expect(&program, entry, expected);
    }

    let illegal_read = parse(&format!(
        r#"
{BASE}
{declarations}
def copied() -> Map<&2, Ticket> & Ticket:
  Map.get(Ticket, Ticket{{0n}}, sample(), "a")
"#
    ))
    .expect("affine-copy attempt parses");
    check_book(&illegal_read).expect_err("Map.get must not duplicate a Type-only value");
}
