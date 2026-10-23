// SPDX-License-Identifier: MPL-2.0
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use normalization_shape::shape;
use normalization_shape::shape_in_book;
use std::path::Path;
use std::rc::Rc;
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
