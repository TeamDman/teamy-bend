// SPDX-License-Identifier: MPL-2.0
//! Strict reductions and affine controls for the ordinary Base formatting/Set kit.

use std::fmt::Write;
use teamy_bend::kernel::CheckedBook;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

const BASE: &str = include_str!("../src/syntax/base.bend");
const HELPERS: &str = r"
type Ticket is Type:
  Ticket{number: Nat}
def ticket_text(t: Ticket) -> String:
  match t:
    case Ticket{number}: Nat.show(number)
def membership_result(p: Set() & Bool) -> List<&2, String> & Bool:
  (s, found) = p
  (Set.to_list(s), found)
def set_as_map(s: Set()) -> Map<&2, Unit>: s
def map_as_set(m: Map<&2, Unit>) -> Set(): m
";

fn witnesses(
    cases: &[(&str, &str, &str)],
    full_equations: usize,
    projections: &str,
) -> CheckedBook {
    // Larger String/Map constructor equalities can reach the unchanged kernel
    // nesting limit. Check bounded complete equations or projections, and also
    // evaluate every well-typed result against its full constructor value.
    let mut source = format!("{BASE}\n{HELPERS}\n");
    for (index, (ty, expression, expected)) in cases.iter().enumerate() {
        writeln!(source, "def value{index}() -> {ty}: {expression}").unwrap();
        if index < full_equations {
            writeln!(
                source,
                "def proof{index}() -> {{value{index}() == {expected} : {ty}}}: {{==}}"
            )
            .unwrap();
        }
    }
    source.push_str(projections);
    let book = parse(&source).expect("ordinary formatting/Set witnesses parse");
    let checked = check_book(&book).expect("all finite equality witnesses check strictly");
    for (index, (_, _, expected)) in cases.iter().enumerate() {
        let expected = parse_term(expected).expect("expected constructor value parses");
        let actual = checked
            .evaluate_data(&format!("value{index}"), &[])
            .expect("checked runtime agrees with kernel reduction");
        assert_eq!(actual.to_string(), expected.to_string(), "case {index}");
    }
    checked
}

#[test]
fn bool_and_maybe_formatting_reduce_in_checked_proofs() {
    witnesses(
        &[
            ("String", "Bool.show(False{})", r#""False""#),
            ("String", "Bool.show(True{})", r#""True""#),
            (
                "String",
                "Maybe.show(~&2, ~Nat, ~Nat.show, None{})",
                r#""None""#,
            ),
            (
                "String",
                "Maybe.show(~&2, ~Nat, ~Nat.show, Some{7n})",
                r#""Some(7)""#,
            ),
            (
                "String",
                "Maybe.show(~&2, ~Bool, ~Bool.show, Some{False{}})",
                r#""Some(False)""#,
            ),
            (
                "String",
                r#"Maybe.show(~&2, ~String, ~(x => x), Some{"é🙂"})"#,
                r#""Some(é🙂)""#,
            ),
            (
                "String",
                r#"Maybe.show(~&2, ~String, ~(x => x), Some{""})"#,
                r#""Some()""#,
            ),
        ],
        3,
        r"
def some_length() -> {String.length(value3()) == 7n : Nat}: {==}
def bool_length() -> {String.length(value4()) == 11n : Nat}: {==}
def unicode_length() -> {String.length(value5()) == 8n : Nat}: {==}
def empty_payload_length() -> {String.length(value6()) == 6n : Nat}: {==}
",
    );
}

#[test]
fn maybe_show_consumes_an_affine_payload_once() {
    witnesses(
        &[
            (
                "String",
                "Maybe.show(~&1, ~Ticket, ~ticket_text, None{})",
                r#""None""#,
            ),
            (
                "String",
                "Maybe.show(~&1, ~Ticket, ~ticket_text, Some{Ticket{4n}})",
                r#""Some(4)""#,
            ),
        ],
        1,
        r"
def affine_length() -> {String.length(value1()) == 7n : Nat}: {==}
",
    );
    for body in [
        r"def bad(t: Ticket) -> String:
  String.append(Maybe.show(~&1, ~Ticket, ~ticket_text, Some{t}),
    Maybe.show(~&1, ~Ticket, ~ticket_text, Some{t}))",
        r"def bad() -> String:
  Maybe.show(~&1, ~Ticket, ~(t => String.append(ticket_text(t), ticket_text(t))), Some{Ticket{1n}})",
    ] {
        let book = parse(&format!("{BASE}\n{HELPERS}\n{body}\n"))
            .expect("duplicated affine fixture is well formed");
        let error = check_book(&book)
            .expect_err("formatting cannot duplicate affine payloads")
            .to_string();
        assert!(error.contains("observed Many"), "{error}");
    }
}

#[test]
fn set_alias_empty_add_delete_and_duplicate_inputs_have_checked_results() {
    witnesses(
        &[
            ("Set()", "map_as_set(set_as_map(Set.new()))", "MTip{}"),
            ("Nat", "Set.size(Set.new())", "0n"),
            ("List<&2, String>", "Set.to_list(Set.new())", "[]"),
            ("Nat", r#"Set.size(Set.add(Set.new(), "a"))"#, "1n"),
            (
                "Nat",
                r#"Set.size(Set.add(Set.add(Set.new(), "a"), "a"))"#,
                "1n",
            ),
            ("Nat", r#"Set.size(Set.from_list(["x", "y", "x"]))"#, "2n"),
            (
                "List<&2, String>",
                r#"Set.to_list(Set.from_list(["x", "y", "x"]))"#,
                r#"["x", "y"]"#,
            ),
            (
                "List<&2, String>",
                r#"Set.to_list(Set.del(Set.from_list(["b", "a"]), "a"))"#,
                r#"["b"]"#,
            ),
            (
                "List<&2, String>",
                r#"Set.to_list(Set.del(Set.from_list(["b", "a"]), "b"))"#,
                r#"["a"]"#,
            ),
            (
                "List<&2, String>",
                r#"Set.to_list(Set.del(Set.from_list(["b", "a"]), "z"))"#,
                r#"["a", "b"]"#,
            ),
            (
                "List<&2, String>",
                r#"Set.to_list(Set.del(Set.add(Set.new(), "a"), "a"))"#,
                "[]",
            ),
            (
                "List<&2, String>",
                r#"Set.to_list(Set.del(Set.new(), "a"))"#,
                "[]",
            ),
            (
                "List<&2, String>",
                r#"Set.to_list(Set.from_list.go(["b", "a"], Set.add(Set.new(), "c")))"#,
                r#"["a", "b", "c"]"#,
            ),
        ],
        13,
        "",
    );
}

#[test]
fn membership_preserves_the_set_and_keys_retain_map_traversal_order() {
    witnesses(
        &[
            (
                "List<&2, String> & Bool",
                r#"membership_result(Set.has(Set.new(), "a"))"#,
                "([], False{})",
            ),
            (
                "List<&2, String> & Bool",
                r#"membership_result(Set.has(Set.from_list(["b", "a"]), "a"))"#,
                r#"(["a", "b"], True{})"#,
            ),
            (
                "List<&2, String> & Bool",
                r#"membership_result(Set.has(Set.from_list(["b", "a"]), "z"))"#,
                r#"(["a", "b"], False{})"#,
            ),
            (
                "List<&2, String>",
                r#"Set.to_list(Set.from_list(["b", "ab", "", "a"]))"#,
                r#"["", "a", "ab", "b"]"#,
            ),
            (
                "List<&2, String>",
                r#"Set.to_list(Set.from_list(["a", "", "ab", "b", "ab"]))"#,
                r#"["", "a", "ab", "b"]"#,
            ),
            (
                "List<&2, String>",
                r#"Set.to_list(Set.from_list(["é", "z", "a"]))"#,
                r#"["a", "z", "é"]"#,
            ),
        ],
        3,
        "",
    );
}

#[test]
fn false_formatting_and_set_equalities_remain_rejected() {
    for (ty, left, right) in [
        ("String", "Bool.show(True{})", r#""true""#),
        (
            "String",
            "Maybe.show(~&2, ~Nat, ~Nat.show, Some{1n})",
            r#""None""#,
        ),
        ("Nat", "Set.size(Set.new())", "1n"),
        ("Nat", r#"Set.size(Set.from_list(["a", "a"]))"#, "2n"),
        (
            "List<&2, String>",
            r#"Set.to_list(Set.from_list(["b", "a"]))"#,
            r#"["b", "a"]"#,
        ),
    ] {
        let book = parse(&format!(
            "{BASE}\ndef false_equation() -> {{{left} == {right} : {ty}}}: {{==}}\n"
        ))
        .expect("false equality fixture parses");
        let error = check_book(&book)
            .expect_err("ordinary library code cannot prove a false equality")
            .to_string();
        assert!(error.contains("reflexivity"), "{error}");
    }
}
