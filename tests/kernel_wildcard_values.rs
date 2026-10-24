// SPDX-License-Identifier: MPL-2.0
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use normalization_shape::Shape;
use normalization_shape::shape_in_book;
use std::path::Path;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse_term;

#[test]
fn wildcard_and_empty_match_values_preserve_graph_text_and_binding_identity() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-wildcard-values");
    for name in ["underscore_print", "wildcard_chain_print", "efq_arm_value"] {
        let book = load(root.join(format!("{name}.bend"))).unwrap();
        let checked = check_book(&book).unwrap();
        let expected =
            std::fs::read_to_string(root.join("normal-forms").join(format!("{name}.stdout")))
                .unwrap();
        let graph = std::fs::read_to_string(root.join("normal-forms").join(format!("{name}.json")))
            .unwrap();
        let oracle: Shape = facet_json::from_str(&graph).unwrap();
        for _ in 0..2 {
            for value in [
                checked.normalize_definition("main").unwrap(),
                checked.evaluate("main", &[]).unwrap(),
            ] {
                assert_eq!(format!("{value}\n"), expected, "{name}");
                assert_eq!(shape_in_book(&value, &book).unwrap(), oracle, "{name}");
                assert_eq!(
                    shape_in_book(&parse_term(expected.trim()).unwrap(), &book).unwrap(),
                    oracle
                );
                match name {
                    "underscore_print" => {
                        let Term::Lam { name, body, .. } = value.as_ref() else {
                            panic!("expected wildcard lambda");
                        };
                        assert_eq!(name, "_");
                        assert_eq!(body.to_string(), "0n");
                    }
                    "wildcard_chain_print" => {
                        let Term::Lam {
                            name: first,
                            id: outer,
                            body,
                            ..
                        } = value.as_ref()
                        else {
                            panic!("expected first lambda");
                        };
                        let Term::Lam {
                            name: second,
                            id: inner,
                            body,
                            ..
                        } = body.as_ref()
                        else {
                            panic!("expected second lambda");
                        };
                        assert_eq!((first.as_str(), second.as_str()), ("_", "_"));
                        assert_ne!(
                            outer, inner,
                            "identical printed wildcards bind distinct identities"
                        );
                        assert_eq!(body.to_string(), "5n");
                    }
                    _ => {
                        let Term::Ctr { name, args } = value.as_ref() else {
                            panic!("expected tuple");
                        };
                        assert_eq!(name, "Tuple");
                        assert_eq!(args.len(), 2);
                        assert!(
                            matches!(args[0].as_ref(), Term::Efq),
                            "selected empty-match value must not refold"
                        );
                        let Term::Lam { id, body, .. } = args[1].as_ref() else {
                            panic!("expected open empty-domain function");
                        };
                        let Term::App(function, argument) = body.as_ref() else {
                            panic!("expected blocked call");
                        };
                        let Term::App(function, _) = function.as_ref() else {
                            panic!("expected explicit type argument");
                        };
                        assert!(
                            matches!(function.as_ref(), Term::Ref(name) if name == "Empty.absurd")
                        );
                        assert!(
                            matches!(argument.as_ref(), Term::Var { id: used, .. } if used == id)
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn wildcard_applications_are_constant_without_capturing_partial_arguments() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-wildcard-values/wrappers");
    let mut complete = 0;
    let mut partial = 0;
    for name in ["underscore_print", "wildcard_chain_print"] {
        let book = load(root.join(format!("{name}.bend"))).unwrap();
        let checked = check_book(&book).unwrap();
        let inputs = [0_u32, 7, u32::MAX];
        let vectors: Vec<Vec<u32>> = if name == "underscore_print" {
            inputs.map(|x| vec![x]).into()
        } else {
            inputs
                .into_iter()
                .flat_map(|x| inputs.map(|y| vec![x, y]))
                .collect()
        };
        let expected = if name == "underscore_print" {
            "0n"
        } else {
            "5n"
        };
        let oracle = shape_in_book(&parse_term(expected).unwrap(), &book).unwrap();
        for vector in vectors {
            let args = vector
                .iter()
                .map(|x| parse_term(&x.to_string()).unwrap())
                .collect::<Vec<_>>();
            let value = checked.evaluate("apply", &args).unwrap();
            assert_eq!(value.to_string(), expected);
            assert_eq!(shape_in_book(&value, &book).unwrap(), oracle);
            complete += 1;
        }
        if name == "wildcard_chain_print" {
            for input in inputs {
                let value = checked
                    .evaluate("partial", &[parse_term(&input.to_string()).unwrap()])
                    .unwrap();
                assert_eq!(value.to_string(), "_ => 5n");
                assert_eq!(
                    shape_in_book(&value, &book).unwrap(),
                    shape_in_book(&parse_term("_ => 5n").unwrap(), &book).unwrap()
                );
                partial += 1;
            }
        }
        let expected = std::fs::read_to_string(root.join(format!("{name}.stdout"))).unwrap();
        assert_eq!(
            format!("{}\n", checked.evaluate("main", &[]).unwrap()),
            expected
        );
    }
    assert_eq!((complete, partial), (12, 3));
}

#[test]
fn both_boolean_arms_return_empty_match_functions_without_applying_them() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-wildcard-values/efq_arm_value.bend");
    let book = load(path).unwrap();
    let checked = check_book(&book).unwrap();
    for input in ["True{}", "False{}"] {
        let value = checked
            .evaluate("pick", &[parse_term(input).unwrap()])
            .unwrap();
        assert_eq!(value.to_string(), "\\{}");
        assert!(matches!(value.as_ref(), Term::Efq));
        assert_eq!(shape_in_book(&value, &book).unwrap(), Shape::EmptyMatch);
    }
    // No Empty value exists here: `use` remains an open, blocked eliminator.
    // A fabricated argument would not be executable qualification of this source.
}
