// SPDX-License-Identifier: MPL-2.0
use std::rc::Rc;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::term;
use teamy_bend::syntax::parse_term;

#[test]
fn complete_strings_and_characters_use_pinned_literal_spelling() {
    for (source, expected) in [
        ("SNil{}", "\"\""),
        ("SCon{Chr{65}, SNil{}}", "\"A\""),
        ("SCon{Chr{39}, SCon{Chr{34}, SNil{}}}", "\"'\\\"\""),
        (
            "SCon{Chr{0}, SCon{Chr{10}, SCon{Chr{9}, SCon{Chr{13}, SCon{Chr{92}, SNil{}}}}}}",
            "\"\\0\\n\\t\\r\\\\\"",
        ),
        ("SCon{Chr{1}, SCon{Chr{127}, SNil{}}}", "\"\\u{1}\\u{7f}\""),
        (
            "SCon{Chr{55296}, SCon{Chr{1114112}, SNil{}}}",
            "\"\\u{d800}\\u{110000}\"",
        ),
        ("SCon{Chr{233}, SCon{Chr{128578}, SNil{}}}", "\"é🙂\""),
        ("Chr{39}", "'\\''"),
        ("Chr{34}", "'\"'"),
        ("Chr{4294967295}", "'\\u{ffffffff}'"),
        ("Chr{{65 : U32}}", "'A'"),
    ] {
        let term = parse_term(source).unwrap();
        assert_eq!(term.to_string(), expected, "{source}");
    }
}

#[test]
fn partial_and_malformed_strings_keep_their_structure() {
    for (source, expected) in [
        ("SCon{Chr{65}, tail}", "SCon{'A', tail}"),
        ("SCon{unknown, SNil{}}", "SCon{unknown, \"\"}"),
        (
            "SCon{unknown, SCon{Chr{66}, SNil{}}}",
            "SCon{unknown, \"B\"}",
        ),
        (
            "SCon{Chr{65}, SCon{unknown, SCon{Chr{66}, SNil{}}}}",
            "SCon{'A', SCon{unknown, \"B\"}}",
        ),
        (
            "SCon{Chr{65}, SCon{Chr{66}, tail}}",
            "SCon{'A', SCon{'B', tail}}",
        ),
        ("SCon{Chr{65}, SCon{Chr{66}}}", "SCon{'A', SCon{'B'}}"),
        ("SCon{Chr{65}}", "SCon{'A'}"),
        ("Chr{unknown}", "Chr{unknown}"),
        ("SCon{{Chr{65} : Char}, SNil{}}", "SCon{{'A' : Char}, \"\"}"),
    ] {
        let term = parse_term(source).unwrap();
        assert_eq!(term.to_string(), expected, "{source}");
    }
}

#[test]
fn long_open_string_is_rendered_without_recursive_tail_scans() {
    let head = parse_term("'a'").unwrap();
    let mut value = term(Term::Ref("tail".into()));
    for _ in 0..4096 {
        value = term(Term::Ctr {
            name: "SCon".into(),
            args: vec![Rc::clone(&head), value],
        });
    }
    let mut expected = "SCon{'a', ".repeat(4096);
    expected.push_str("tail");
    expected.push_str(&"}".repeat(4096));
    assert_eq!(value.to_string(), expected);
}
