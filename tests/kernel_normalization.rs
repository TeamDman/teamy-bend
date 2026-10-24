// SPDX-License-Identifier: MPL-2.0
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use normalization_shape::shape;
use normalization_shape::shape_in_book;
use std::path::Path;
use std::rc::Rc;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::term;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

const CASES: [&str; 9] = [
    include_str!("fixtures/cpu-normalization/computed_operator_match_002.bend"),
    include_str!("fixtures/cpu-normalization/computed_operator_match_002_dbl.bend"),
    include_str!("fixtures/cpu-normalization/computed_operator_match_002_sel.bend"),
    include_str!("fixtures/cpu-normalization/computed_recursion_002.bend"),
    include_str!("fixtures/cpu-normalization/computed_recursion_003.bend"),
    include_str!("fixtures/cpu-normalization/computed_recursion_004.bend"),
    include_str!("fixtures/cpu-normalization/computed_recursion_002_sel.bend"),
    include_str!("fixtures/cpu-normalization/computed_recursion_003_flip.bend"),
    include_str!("fixtures/cpu-normalization/computed_recursion_004_sel.bend"),
];

#[test]
fn checking_only_list_fixture_preserves_named_values_and_proofs() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-checking/list_reverse_sort.bend");
    let book = load(path).unwrap();
    let checked = check_book(&book).expect("all concrete list equalities check");
    checked
        .normalize_definition("main")
        .expect_err("checking-only source has no main");
    for (entry, expected) in [
        ("l123", "[1, 2, 3]"),
        ("l321", "[3, 2, 1]"),
        ("rev", "{==}"),
        ("rev2", "{==}"),
        ("srt", "{==}"),
    ] {
        let actual = checked.normalize_definition(entry).unwrap();
        assert_eq!(actual.to_string(), expected, "{entry}");
        assert_eq!(
            shape_in_book(&actual, &book).unwrap(),
            shape_in_book(&parse_term(expected).unwrap(), &book).unwrap(),
            "{entry}"
        );
    }
}

#[test]
fn checking_only_list_fixture_rejects_wrong_reverse_and_lost_duplicate() {
    let original = include_str!("fixtures/cpu-checking/list_reverse_sort.bend");
    for (before, after, definition) in [
        ("[3, 2, 1]", "[3, 1, 2]", "rev"),
        ("[1, 2, 3, 3]", "[1, 2, 3]", "srt"),
    ] {
        assert_eq!(original.matches(before).count(), 1);
        let source = original
            .replace(before, after)
            .replace("import Base\n", include_str!("../src/syntax/base.bend"));
        let book = parse(&source).expect("false proof still parses");
        let error = check_book(&book).expect_err("incorrect concrete equality must fail");
        assert!(
            error.to_string().starts_with(&format!(
                "{definition}: reflexivity requires equal endpoints:"
            )),
            "{error}"
        );
    }
}

#[test]
fn original_function_mains_match_pinned_normal_forms_without_calling_them() {
    for source in CASES {
        let checked = check_book(&parse(source).unwrap()).unwrap();
        let expected = source
            .lines()
            .filter_map(|line| line.strip_prefix("#|"))
            .collect::<Vec<_>>()
            .join("\n");
        let actual = checked.normalize_definition("main").unwrap();
        assert_eq!(
            shape(&actual).unwrap(),
            shape(&parse_term(&expected).unwrap()).unwrap()
        );
        assert_eq!(actual.to_string(), expected);
        // The normal result must also survive the port's own textual representation.
        assert_eq!(
            shape(&actual).unwrap(),
            shape(&parse_term(&actual.to_string()).unwrap()).unwrap()
        );
    }
}

#[test]
fn recursive_function_normalization_preserves_complete_application_results() {
    for (source, expected) in [
        (CASES[3], ["S{Z{}}", "Z{}", "S{Z{}}"]),
        (CASES[4], ["T{}", "F{}", "T{}"]),
        (CASES[5], ["S{Z{}}", "Z{}", "S{Z{}}"]),
    ] {
        let checked = check_book(&parse(source).unwrap()).unwrap();
        checked.normalize_definition("main").unwrap();
        for (argument, expected) in ["Zn{}", "Sn{Zn{}}", "Sn{Sn{Zn{}}}"]
            .into_iter()
            .zip(expected)
        {
            assert_eq!(
                checked
                    .evaluate("main", &[parse_term(argument).unwrap()])
                    .unwrap()
                    .to_string(),
                expected
            );
        }
    }
}

#[test]
fn selector_and_flip_companions_preserve_nested_and_default_application_results() {
    for (source, cases) in [
        (
            CASES[6],
            vec![
                ("Z{}", "S{Z{}}"),
                ("S{Z{}}", "Z{}"),
                ("S{S{Z{}}}", "S{Z{}}"),
            ],
        ),
        (CASES[7], vec![("T{}", "F{}"), ("F{}", "T{}")]),
        (
            CASES[8],
            vec![("Z{}", "S{Z{}}"), ("S{Z{}}", "Z{}"), ("S{S{Z{}}}", "Z{}")],
        ),
    ] {
        let checked = check_book(&parse(source).unwrap()).unwrap();
        for (argument, expected) in cases {
            assert_eq!(
                checked
                    .evaluate("main", &[parse_term(argument).unwrap()])
                    .unwrap()
                    .to_string(),
                expected
            );
        }
    }
}

#[test]
fn base_function_companions_preserve_normalized_structure_and_exact_text() {
    for name in [
        "computed_recursion_001_main_dec",
        "computed_recursion_001_not",
        "computed_recursion_001_par",
        "computed_recursion_000",
        "computed_recursion_000_big",
        "computed_recursion_001",
        "chain_suffix_u32_f",
        "chain_suffix_u32_g",
        "chain_suffix_u32_h",
        "chain_brace_suffix_f",
        "chain_brace_suffix_g",
        "chain_brace_suffix_h",
        "chain_default_nat_f",
        "chain_default_nat_g",
        "chain_spacing_rules_f",
        "chain_spacing_rules_g",
        "chain_spacing_rules_h",
        "chain_paren_group_f",
        "chain_paren_group_g",
        "chain_paren_group_h",
        "chain_paren_group_k",
        "chain_paren_group_m",
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-normalization")
            .join(format!("{name}.bend"));
        let source = std::fs::read_to_string(&path).unwrap();
        let book = load(&path).unwrap();
        let checked = check_book(&book).unwrap();
        let expected = source
            .lines()
            .filter_map(|line| line.strip_prefix("#|"))
            .collect::<Vec<_>>()
            .join("\n");
        let actual = checked.normalize_definition("main").unwrap();
        assert_eq!(actual.to_string(), expected, "{name}");
        assert_eq!(
            shape_in_book(&actual, &book).unwrap(),
            shape_in_book(&parse_term(&expected).unwrap(), &book).unwrap(),
            "{name}"
        );
    }
}

#[test]
fn base_function_companions_preserve_captured_values_and_recursive_parity() {
    for (name, cases) in [
        (
            "computed_recursion_001_main_dec",
            vec![
                (vec!["S{Z{}}", "T{}", "T{}"], "S{S{Z{}}}"),
                (vec!["S{Z{}}", "T{}", "F{}"], "Z{}"),
                (vec!["S{Z{}}", "F{}", "T{}"], "S{Z{}}"),
                (vec!["S{Z{}}", "F{}", "F{}"], "S{Z{}}"),
            ],
        ),
        (
            "computed_recursion_001_not",
            vec![(vec!["T{}"], "F{}"), (vec!["F{}"], "T{}")],
        ),
        (
            "computed_recursion_001_par",
            vec![
                (vec!["Z{}"], "T{}"),
                (vec!["S{Z{}}"], "F{}"),
                (vec!["S{S{Z{}}}"], "T{}"),
                (vec!["S{S{S{Z{}}}}"], "F{}"),
            ],
        ),
    ] {
        let book = load(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/cpu-normalization")
                .join(format!("{name}.bend")),
        )
        .unwrap();
        let checked = check_book(&book).unwrap();
        checked.normalize_definition("main").unwrap();
        for (arguments, expected) in cases {
            let args = arguments
                .into_iter()
                .map(|argument| parse_term(argument).unwrap())
                .collect::<Vec<_>>();
            assert_eq!(
                checked.evaluate("main", &args).unwrap().to_string(),
                expected,
                "{name}"
            );
        }
    }
}

#[test]
fn nat_fuel_and_reusable_bindings_preserve_complete_application_results() {
    for (name, cases) in [
        (
            "computed_recursion_000",
            vec![
                (vec!["0n", "Sn{Sn{Zn{}}}"], "Z{}"),
                (vec!["1n", "Zn{}"], "Z{}"),
                (vec!["1n", "Sn{Zn{}}"], "Z{}"),
                (vec!["1n", "Sn{Sn{Sn{Sn{Zn{}}}}}"], "S{S{Z{}}}"),
                (vec!["2n", "Sn{Sn{Sn{Sn{Zn{}}}}}"], "S{S{S{S{Z{}}}}}"),
            ],
        ),
        (
            "computed_recursion_000_big",
            vec![
                (vec!["Zn{}"], "F{}"),
                (vec!["Sn{Zn{}}"], "F{}"),
                (vec!["Sn{Sn{Zn{}}}"], "T{}"),
                (vec!["Sn{Sn{Sn{Zn{}}}}"], "T{}"),
            ],
        ),
        (
            "computed_recursion_001",
            vec![
                (vec!["0n", "S{S{S{Z{}}}}"], "Z{}"),
                (vec!["1n", "Z{}"], "Z{}"),
                (vec!["1n", "S{Z{}}"], "S{Z{}}"),
                (vec!["1n", "S{S{Z{}}}"], "Z{}"),
                (vec!["2n", "S{S{Z{}}}"], "S{Z{}}"),
                (vec!["3n", "S{S{S{Z{}}}}"], "S{S{Z{}}}"),
            ],
        ),
    ] {
        let book = load(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/cpu-normalization")
                .join(format!("{name}.bend")),
        )
        .unwrap();
        let checked = check_book(&book).unwrap();
        for (arguments, expected) in cases {
            let args = arguments
                .into_iter()
                .map(|argument| parse_term(argument).unwrap())
                .collect::<Vec<_>>();
            assert_eq!(
                checked.evaluate("main", &args).unwrap().to_string(),
                expected,
                "{name}"
            );
        }
    }
}

#[test]
fn complete_nat_rendering_preserves_core_structure_and_open_boundaries() {
    for (source, expected) in [
        ("Zero{}", "0n"),
        ("Succ{Zero{}}", "1n"),
        ("Succ{Succ{Succ{Zero{}}}}", "3n"),
        ("31n", "31n"),
        (
            "x => {Nat.add(x, Succ{Zero{}}) == Zero{} : Nat}",
            "x => {Nat.add(x, 1n) == 0n : Nat}",
        ),
        ("Pair{Succ{Zero{}}, Zero{}}", "Pair{1n, 0n}"),
        ("f(Succ{Zero{}})(Zero{})", "f(1n, 0n)"),
        ("Zero{x}", "Zero{x}"),
        ("Succ{}", "Succ{}"),
        ("Succ{Zero{}, Zero{}}", "Succ{0n, 0n}"),
        ("Local.Succ{Local.Zero{}}", "Local.Succ{Local.Zero{}}"),
    ] {
        let value = parse_term(source).unwrap();
        assert_eq!(value.to_string(), expected, "{source}");
        assert_eq!(
            shape(&value).unwrap(),
            shape(&parse_term(expected).unwrap()).unwrap(),
            "{source}"
        );
    }
    // Open or annotated tails must retain their structure under NUMBER n+tail.
    for source in ["Succ{tail}", "Succ{f(Zero{})}"] {
        let value = parse_term(source).unwrap();
        assert_eq!(
            shape(&value).unwrap(),
            shape(&parse_term(&value.to_string()).unwrap()).unwrap(),
            "{source}"
        );
    }
    let hole = parse_term("Succ{?unknown}").unwrap();
    assert_eq!(hole.to_string(), "1n+?unknown");
    let rendered = parse_term(&hole.to_string()).unwrap();
    let Term::Ctr { name, args } = rendered.as_ref() else {
        panic!("the open constructor must survive");
    };
    assert_eq!(name, "Succ");
    assert!(
        matches!(args.as_slice(), [tail] if matches!(tail.as_ref(), Term::Hole(name) if name == "unknown"))
    );
    let annotated = parse_term("Succ{{Zero{} : Nat}}").unwrap();
    assert_eq!(annotated.to_string(), "1n+{0n : Nat}");
    let rendered = parse_term(&annotated.to_string()).unwrap();
    let Term::Ctr { name, args } = rendered.as_ref() else {
        panic!("the annotated constructor must survive");
    };
    assert_eq!(name, "Succ");
    let [tail] = args.as_slice() else {
        panic!("the annotation must remain the only field");
    };
    let Term::Ann(value, ty) = tail.as_ref() else {
        panic!("printing must not erase the annotation boundary");
    };
    assert_eq!(
        shape(value).unwrap(),
        shape(&parse_term("0n").unwrap()).unwrap()
    );
    assert_eq!(
        shape(ty).unwrap(),
        shape(&parse_term("Nat").unwrap()).unwrap()
    );
}

#[test]
fn open_nat_rendering_preserves_spines_precedence_and_binding() {
    for (source, expected) in [
        ("Succ{tail}", "1n+tail"),
        ("Succ{Succ{tail}}", "2n+tail"),
        ("x => Succ{x}", "x => 1n+x"),
        ("Succ{x => x}", "1n+(x => x)"),
        ("Succ{f(Zero{})}", "1n+f(0n)"),
        ("f(Succ{tail})(Zero{})", "f(1n+tail, 0n)"),
        ("Succ{tail}(Zero{})", "(1n+tail)(0n)"),
        ("Succ{(%{==} : N; X{})}", "1n+(%{==} : N; X{})"),
        (
            "{Succ{tail} == Succ{other} : Nat}",
            "{1n+tail == 1n+other : Nat}",
        ),
        ("Succ{Zero{tail}}", "1n+Zero{tail}"),
        ("Succ{Succ{}}", "1n+Succ{}"),
        ("Succ{Succ{a, b}}", "1n+Succ{a, b}"),
        ("Local.Succ{Succ{tail}}", "Local.Succ{1n+tail}"),
    ] {
        let value = parse_term(source).unwrap();
        assert_eq!(value.to_string(), expected, "{source}");
        assert_eq!(
            shape(&value).unwrap(),
            shape(&parse_term(expected).unwrap()).unwrap(),
            "{source}"
        );
    }
    // An open chain is syntax, not addition or an evaluation request.
    let source = format!("{}unresolved{}", "Succ{".repeat(64), "}".repeat(64));
    assert!(
        parse_term(&source)
            .unwrap_err()
            .to_string()
            .contains("parser resource limit of 64")
    );
    // Core values can contain a longer spine than the deeply nested input form.
    // Construct it directly instead of changing the parser's input budget.
    let mut value = parse_term("unresolved").unwrap();
    for _ in 0..64 {
        value = term(Term::Ctr {
            name: "Succ".into(),
            args: vec![value],
        });
    }
    assert_eq!(value.to_string(), "64n+unresolved");
    assert_eq!(
        shape(&value).unwrap(),
        shape(&parse_term(&value.to_string()).unwrap()).unwrap()
    );

    // A let tail keeps its scope, and an arrow domain must bind more tightly.
    let value = parse_term("Succ{(x = Zero{}; x)}").unwrap();
    assert_eq!(value.to_string(), "1n+(x = 0n; x)");
    let rendered = parse_term(&value.to_string()).unwrap();
    let Term::Ctr { args, .. } = rendered.as_ref() else {
        panic!("successor survives")
    };
    let Term::Let { bindings, body } = args[0].as_ref() else {
        panic!("let scope survives")
    };
    assert_eq!(bindings.len(), 1);
    assert!(matches!(body.as_ref(), Term::Var { id, .. } if *id == bindings[0].id));
    let domain = parse_term("@x:Succ{tail} -> x").unwrap();
    assert_eq!(domain.to_string(), "@x:(1n+tail) -> x");
    let rendered = parse_term(&domain.to_string()).unwrap();
    let Term::All {
        id, domain, body, ..
    } = rendered.as_ref()
    else {
        panic!("arrow survives")
    };
    assert_eq!(
        shape(domain).unwrap(),
        shape(&parse_term("Succ{tail}").unwrap()).unwrap()
    );
    assert!(matches!(body.as_ref(), Term::Var { id: body_id, .. } if body_id == id));
}

#[test]
fn array_rendering_preserves_complete_trees_and_partial_boundaries() {
    for (source, expected) in [
        ("ALeaf{x}", "[x]"),
        ("ANode{ALeaf{x}, ALeaf{y}}", "[x, y]"),
        (
            "ANode{ALeaf{1n}, ANode{ALeaf{2n}, ALeaf{3n}}}",
            "[1n, 2n, 3n]",
        ),
        ("ALeaf{ANode{ALeaf{x}, ALeaf{y}}}", "[[x, y]]"),
        ("ALeaf{x => x}", "[x => x]"),
        ("ALeaf{unknown(x)}", "[unknown(x)]"),
        ("ANode{ALeaf{x}, tail}", "ANode{[x], tail}"),
        ("ANode{{ALeaf{x} : A}, ALeaf{y}}", "ANode{{[x] : A}, [y]}"),
        ("ALeaf{}", "ALeaf{}"),
        ("ALeaf{x, y}", "ALeaf{x, y}"),
        ("ANode{ALeaf{x}}", "ANode{[x]}"),
        (
            "ANode{ALeaf{x}, ALeaf{y}, ALeaf{z}}",
            "ANode{[x], [y], [z]}",
        ),
        ("Local.ALeaf{x}", "Local.ALeaf{x}"),
        ("ALeaf{(x = 1n; x)}", "[(x = 1n; x)]"),
    ] {
        assert_eq!(
            parse_term(source).unwrap().to_string(),
            expected,
            "{source}"
        );
    }
    // Walk an existing core tree iteratively without changing the input guard.
    let leaf = parse_term("ALeaf{x}").unwrap();
    let mut value = Rc::clone(&leaf);
    for _ in 0..64 {
        value = term(Term::Ctr {
            name: "ANode".into(),
            args: vec![Rc::clone(&leaf), value],
        });
    }
    assert_eq!(value.to_string(), format!("[{}]", ["x"; 65].join(", ")));
    let deep = format!(
        "{}ALeaf{{x}}{}",
        "ANode{ALeaf{x}, ".repeat(64),
        "}".repeat(64)
    );
    assert!(
        parse_term(&deep)
            .unwrap_err()
            .to_string()
            .contains("parser resource limit of 64")
    );
}

#[test]
fn sequence_literal_rendering_preserves_tuple_and_closed_list_structure() {
    for (source, expected) in [
        ("Nil{}", "[]"),
        ("Con{1n, Con{2n, Nil{}}}", "[1n, 2n]"),
        ("Con{x => Succ{x}, Nil{}}", "[x => 1n+x]"),
        ("Tuple{1n, Tuple{2n, 3n}}", "(1n, 2n, 3n)"),
        ("Tuple{Tuple{1n, 2n}, 3n}", "((1n, 2n), 3n)"),
        ("Tuple{1n, Tuple{2n}}", "(1n, Tuple{2n})"),
        (
            "Tuple{1n, Local.Tuple{2n, 3n}}",
            "(1n, Local.Tuple{2n, 3n})",
        ),
        ("Tuple{1n, 2n, 3n}", "Tuple{1n, 2n, 3n}"),
        ("Nil{tail}", "Nil{tail}"),
        ("Con{1n}", "Con{1n}"),
        ("Local.Con{1n, Local.Nil{}}", "Local.Con{1n, Local.Nil{}}"),
    ] {
        let value = parse_term(source).unwrap();
        assert_eq!(value.to_string(), expected, "{source}");
        assert_eq!(
            shape(&value).unwrap(),
            shape(&parse_term(expected).unwrap()).unwrap(),
            "{source}"
        );
    }
    // No empty-list guess for a broken spine. Open-list shorthand is separate.
    let value = parse_term("Con{1n, tail}").unwrap();
    assert_eq!(
        shape(&value).unwrap(),
        shape(&parse_term(&value.to_string()).unwrap()).unwrap()
    );
}

#[test]
fn nat_literal_fixture_preserves_exact_core_and_computation() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-normalization/nat_literal_roundtrip.bend");
    let source = std::fs::read_to_string(&path).unwrap();
    let expected = source
        .lines()
        .find_map(|line| line.strip_prefix("#|"))
        .unwrap();
    let book = load(path).unwrap();
    let checked = check_book(&book).unwrap();
    let value = checked.normalize_definition("main").unwrap();
    assert_eq!(value.to_string(), expected);
    assert_eq!(
        shape_in_book(&value, &book).unwrap(),
        shape_in_book(&parse_term(expected).unwrap(), &book).unwrap()
    );
    for (argument, expected) in [("0n", "0n"), ("1n", "2n"), ("3n", "6n")] {
        assert_eq!(
            checked
                .evaluate("dbl", &[parse_term(argument).unwrap()])
                .unwrap()
                .to_string(),
            expected
        );
    }
    checked
        .evaluate_data("main", &[])
        .expect_err("a function inside the tuple is not a data result");
}

#[test]
fn application_rendering_preserves_spines_argument_nesting_and_binding() {
    for (source, expected) in [
        ("f(a)(b)(c)", "f(a, b, c)"),
        ("f(g(a)(b))(c)", "f(g(a, b), c)"),
        ("(x => g(x))(a)(b)", "(x => g(x))(a, b)"),
        ("f(x => g(x))(a)", "f(x => g(x), a)"),
        ("(%{==} : N; f)(a)(b)", "(%{==} : N; f)(a, b)"),
        ("f(%{==} : N; a)(b)", "f(%{==} : N; a, b)"),
    ] {
        let parsed = parse_term(source).unwrap();
        assert_eq!(parsed.to_string(), expected);
        assert_eq!(
            shape(&parsed).unwrap(),
            shape(&parse_term(expected).unwrap()).unwrap(),
            "{source}"
        );
    }
    // Keep annotations and let scope instead of flattening through their bodies.
    for (source, expected) in [
        ("{f(a) : T}(b)(c)", "{f(a) : T}(b, c)"),
        ("(x = A{}; f(x))(b)(c)", "(x = A{}; f(x))(b, c)"),
        ("f((x = A{}; x))(b)", "f((x = A{}; x), b)"),
    ] {
        let parsed = parse_term(source).unwrap();
        assert_eq!(parsed.to_string(), expected);
        assert_eq!(parse_term(expected).unwrap().to_string(), expected);
        let Term::App(function, _) = parsed.as_ref() else {
            panic!("outer application must survive");
        };
        assert!(matches!(function.as_ref(), Term::App(..)));
    }
}

#[test]
fn generated_pattern_names_are_local_while_semantic_binder_ids_stay_global() {
    let prefix =
        "type Noise is Data: Quiet{} Loud{next: Noise}\ndef unused(x: Noise) -> Noise: x\n";
    let book = parse(&format!("{prefix}{}", CASES[6])).unwrap();
    let parameters = |name: &str| {
        book.declarations
            .iter()
            .find_map(|declaration| match declaration {
                teamy_bend::kernel::Declaration::Def(definition) if definition.name == name => {
                    Some(definition.parameters[0].id)
                }
                _ => None,
            })
            .unwrap()
    };
    assert_ne!(parameters("unused"), parameters("main"));
    let checked = check_book(&book).unwrap();
    assert_eq!(
        checked.normalize_definition("main").unwrap().to_string(),
        r"\{S: \{Z: Z{}; (_3 => S{Z{}})}; (r => S{Z{}})}"
    );
    assert_eq!(
        checked
            .evaluate("unused", &[parse_term("Quiet{}").unwrap()])
            .unwrap()
            .to_string(),
        "Quiet{}"
    );
    assert_eq!(
        checked
            .evaluate("main", &[parse_term("S{Z{}}").unwrap()])
            .unwrap()
            .to_string(),
        "Z{}"
    );
}

#[test]
fn match_rendering_preserves_order_nested_arms_and_explicit_defaults() {
    for (source, expected) in [
        (r"\{}", r"\{}"),
        (r"\{Z: Z{}; \{}}", r"\{Z: Z{}}"),
        (r"\{Z: \{}; \{}}", r"\{Z: \{}}"),
        (
            r"\{Z: Z{}; \{S: p => S{p}; \{}}}",
            r"\{Z: Z{}; S: (p => S{p})}",
        ),
        (r"\{Z: Z{}; y => RK{y}}", r"\{Z: Z{}; (y => RK{y})}"),
        (
            r"\{A: \{B: V{}; \{}}; \{C: W{}; tail}}",
            r"\{A: \{B: V{}}; C: W{}; tail}",
        ),
    ] {
        let value = parse_term(source).unwrap();
        assert_eq!(value.to_string(), expected);
        assert_eq!(
            shape(&value).unwrap(),
            shape(&parse_term(expected).unwrap()).unwrap()
        );
    }
}

#[test]
fn let_match_branch_body_remains_inside_the_branch() {
    let source = r"\{Z: (x = V{}; x); tail}";
    let original = parse_term(source).unwrap();
    let rendered = original.to_string();
    assert_eq!(rendered, source);
    let roundtrip = parse_term(&rendered).unwrap();
    let Term::Mat { arm, fallback, .. } = roundtrip.as_ref() else {
        panic!("match must survive rendering");
    };
    assert!(matches!(fallback.as_ref(), Term::Ref(name) if name == "tail"));
    let Term::Let { bindings, body } = arm.as_ref() else {
        panic!("let body must remain in its branch");
    };
    assert_eq!(bindings.len(), 1);
    assert!(matches!(body.as_ref(), Term::Var { id, .. } if *id == bindings[0].id));
}

#[test]
fn normalization_does_not_relax_checked_call_arity_or_argument_types() {
    let checked = check_book(&parse(CASES[0]).unwrap()).unwrap();
    checked.normalize_definition("main").unwrap();
    assert!(
        checked
            .normalize_definition("missing")
            .unwrap_err()
            .to_string()
            .contains("undefined definition")
    );
    for result in [
        checked.evaluate("main", &[]),
        checked.evaluate_data("main", &[]),
    ] {
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("needs 1 arguments")
        );
    }
    checked
        .evaluate("main", &[parse_term("R1{}").unwrap()])
        .expect_err("wrong argument type");
    let result = checked
        .evaluate("main", &[parse_term("S{Z{}}").unwrap()])
        .unwrap();
    assert_eq!(result.to_string(), "RK{S{S{Z{}}}}");
}

#[test]
fn structural_comparison_ignores_only_binder_names_and_print_layout() {
    let view = |s| shape(&parse_term(s).unwrap()).unwrap();
    assert_eq!(view("x => x"), view("renamed => renamed"));
    assert_eq!(
        view(r"\{Z: Z{}; S: (p => S{p})}"),
        view(r"\{Z: Z{}; \{S: p => S{p}; \{}}}")
    );
    for (left, right) in [
        ("x => y => x", "x => y => y"),
        ("x => f(g(x))", "x => g(f(x))"),
        (r"\{Z: Z{}; S: (p => S{p})}", r"\{Z: Z{}; S: (p => p)}"),
        (r"\{Z: Z{}; (x => S{x})}", r"\{Z: Z{}; (x => x)}"),
    ] {
        assert_ne!(view(left), view(right));
    }
    assert!(
        shape(&parse_term("Type").unwrap()).is_err(),
        "unsupported syntax must fail closed"
    );
}

#[test]
fn rewrite_normal_forms_preserve_every_motive_and_evidence_binder() {
    for (file, source) in [
        (
            "cpu-normalization/stuck_rewrite_print.bend",
            include_str!("fixtures/cpu-normalization/stuck_rewrite_print.bend"),
        ),
        (
            "cpu-normalization/context_rewrite_hypothesis.bend",
            include_str!("fixtures/cpu-normalization/context_rewrite_hypothesis.bend"),
        ),
        (
            "cpu-normalization/context_rewrite_value.bend",
            include_str!("fixtures/cpu-normalization/context_rewrite_value.bend"),
        ),
        (
            "cpu-rewrite/unnamed.bend",
            include_str!("fixtures/cpu-rewrite/unnamed.bend"),
        ),
        (
            "cpu-rewrite/named.bend",
            include_str!("fixtures/cpu-rewrite/named.bend"),
        ),
        (
            "cpu-rewrite/shadow.bend",
            include_str!("fixtures/cpu-rewrite/shadow.bend"),
        ),
        (
            "cpu-rewrite/nested.bend",
            include_str!("fixtures/cpu-rewrite/nested.bend"),
        ),
    ] {
        let book = load(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(file),
        )
        .unwrap();
        let checked = check_book(&book).unwrap();
        let value = checked.normalize_definition("main").unwrap();
        let expected = source
            .lines()
            .filter_map(|line| line.strip_prefix("#|"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(value.to_string(), expected, "{file}");
        assert_eq!(
            shape_in_book(&value, &book).unwrap(),
            shape_in_book(&parse_term(&expected).unwrap(), &book).unwrap(),
            "{file}"
        );
    }
    for source in [
        include_str!("fixtures/cpu-rewrite/named.bend"),
        include_str!("fixtures/cpu-rewrite/shadow.bend"),
    ] {
        let checked = check_book(&parse(source).unwrap()).unwrap();
        assert_eq!(
            checked
                .evaluate(
                    "main",
                    &[
                        parse_term("Z{}").unwrap(),
                        parse_term("Z{}").unwrap(),
                        parse_term("{==}").unwrap()
                    ]
                )
                .unwrap()
                .to_string(),
            "{==}"
        );
    }
}

#[test]
fn rewrite_roundtrips_keep_inner_lambdas_shadowing_and_match_grouping() {
    for source in [
        "e => %e : N; Z{}",
        "e => %h@e : {h == h : {_ == _ : N}}; {==}",
        "e => %e@e : {e == e : {_ == _ : N}}; e",
        "e => %h@e : {Box{x => x} == Box{_ => _} : B}; e",
        r"\{Z: (%{==} : N; Z{}); tail}",
        "e => %h@e : (_ => _); e",
    ] {
        let value = parse_term(source).unwrap();
        let rendered = value.to_string();
        assert_eq!(rendered, source);
        assert_eq!(
            shape(&value).unwrap(),
            shape(&parse_term(&rendered).unwrap()).unwrap()
        );
    }
}

#[test]
fn annotated_rewrite_binders_and_non_telescope_motives_keep_their_printing_contract() {
    let parsed = parse_term("%h@e : N; body").unwrap();
    let Term::Rwt {
        evidence,
        motive,
        body,
    } = parsed.as_ref()
    else {
        panic!("rewrite")
    };
    let Term::Lam {
        name,
        id,
        body: inner,
    } = motive.as_ref()
    else {
        panic!("motive endpoint")
    };
    let annotation = parse_term("Type").unwrap();
    let annotated = term(Term::Rwt {
        evidence: Rc::clone(evidence),
        motive: term(Term::Ann(
            term(Term::Lam {
                name: name.clone(),
                id: *id,
                body: term(Term::Ann(Rc::clone(inner), Rc::clone(&annotation))),
            }),
            annotation,
        )),
        body: Rc::clone(body),
    });
    assert_eq!(annotated.to_string(), "%h@e : N; body");
    for (motive, expected) in [("P", "%e : P; body"), ("x => x", "%e : (x => x); body")] {
        let value = term(Term::Rwt {
            evidence: Rc::clone(evidence),
            motive: parse_term(motive).unwrap(),
            body: Rc::clone(body),
        });
        assert_eq!(value.to_string(), expected);
    }
}

#[test]
fn rewrite_comparison_rejects_changed_fields_types_and_binding_targets() {
    let view = |s| shape(&parse_term(s).unwrap()).unwrap();
    for (left, right) in [
        ("e => f => %e : N; Z{}", "e => f => %f : N; Z{}"),
        ("e => %e : N; Z{}", "e => %e : M; Z{}"),
        ("e => %e : N; Z{}", "e => %e : N; S{Z{}}"),
        ("e => %h@e : {h == h : N}; e", "e => %h@e : {e == e : N}; e"),
        ("e => %e : {_ == _ : N}; e", "e => %e : {_ == _ : M}; e"),
    ] {
        assert_ne!(view(left), view(right));
    }
    let book = parse("type N is Data:\n  Z{}\n").unwrap();
    let datatype = term(Term::Adt {
        name: "N".into(),
        args: Vec::new(),
        excluded: Vec::new(),
    });
    assert_eq!(
        shape_in_book(&datatype, &book).unwrap(),
        shape_in_book(&parse_term("N").unwrap(), &book).unwrap()
    );
    for changed in [
        term(Term::Adt {
            name: "N".into(),
            args: vec![parse_term("Z{}").unwrap()],
            excluded: Vec::new(),
        }),
        term(Term::Adt {
            name: "N".into(),
            args: Vec::new(),
            excluded: vec!["Z".into()],
        }),
    ] {
        assert_ne!(
            shape_in_book(&datatype, &book).unwrap(),
            shape_in_book(&changed, &book).unwrap()
        );
    }
    assert_ne!(
        shape(&datatype).unwrap(),
        shape(&parse_term("N").unwrap()).unwrap(),
        "no global type-name assumption without book context"
    );
    shape(&parse_term("Type").unwrap()).expect_err("unsupported syntax must fail closed");
}

#[test]
fn open_list_rendering_preserves_structure_and_operand_precedence() {
    for (source, expected) in [
        ("Con{x, tail}", "x <> tail"),
        ("Con{x, Con{y, tail}}", "x <> y <> tail"),
        ("Con{Con{x, tail}, rest}", "(x <> tail) <> rest"),
        ("Con{x => x, tail}", "(x => x) <> tail"),
        ("Con{x, y => y}", "x <> (y => y)"),
        ("Con{Succ{n}, tail}", "(1n+n) <> tail"),
        ("Succ{Con{x, tail}}", "1n+x <> tail"),
        ("Con{x, tail}(y)", "(x <> tail)(y)"),
        ("f(Con{x, tail})", "f(x <> tail)"),
        ("Con{x, Nil{}}", "[x]"),
        ("Con{x, Local.Nil{}}", "x <> Local.Nil{}"),
        ("Con{x, Con{}}", "x <> Con{}"),
        ("Local.Con{x, tail}", "Local.Con{x, tail}"),
    ] {
        let value = parse_term(source).unwrap();
        assert_eq!(value.to_string(), expected, "{source}");
        assert_eq!(
            shape(&value).unwrap(),
            shape(&parse_term(expected).unwrap()).unwrap(),
            "{source}"
        );
    }
}
