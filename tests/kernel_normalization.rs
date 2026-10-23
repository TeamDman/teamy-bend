// SPDX-License-Identifier: MPL-2.0
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use normalization_shape::shape;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

const CASES: [&str; 6] = [
    include_str!("fixtures/cpu-normalization/computed_operator_match_002.bend"),
    include_str!("fixtures/cpu-normalization/computed_operator_match_002_dbl.bend"),
    include_str!("fixtures/cpu-normalization/computed_operator_match_002_sel.bend"),
    include_str!("fixtures/cpu-normalization/computed_recursion_002.bend"),
    include_str!("fixtures/cpu-normalization/computed_recursion_003.bend"),
    include_str!("fixtures/cpu-normalization/computed_recursion_004.bend"),
];

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
        shape(&parse_term("{==}").unwrap()).is_err(),
        "unsupported syntax must fail closed"
    );
}
