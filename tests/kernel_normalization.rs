// SPDX-License-Identifier: MPL-2.0
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use normalization_shape::shape;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

const CASES: [&str; 3] = [
    include_str!("fixtures/cpu-normalization/computed_operator_match_002.bend"),
    include_str!("fixtures/cpu-normalization/computed_operator_match_002_dbl.bend"),
    include_str!("fixtures/cpu-normalization/computed_operator_match_002_sel.bend"),
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
        // The normal result must also survive the port's own textual representation.
        assert_eq!(
            shape(&actual).unwrap(),
            shape(&parse_term(&actual.to_string()).unwrap()).unwrap()
        );
    }
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
