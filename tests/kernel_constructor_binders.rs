// SPDX-License-Identifier: MPL-2.0
#[path = "support/constructor_binder_assertions.rs"]
mod constructor_binder_assertions;
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use constructor_binder_assertions::assert_fresh_match_binders;
use normalization_shape::Shape;
use normalization_shape::shape_in_book;
use std::path::Path;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse_term;

#[test]
fn constructor_continuations_and_minted_binders_preserve_graphs_text_and_identity() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-constructor-binders");
    for name in ["printer_fixpoint", "printer_fresh_binder_pick"] {
        let book = load(root.join(format!("{name}.bend"))).unwrap();
        let checked = check_book(&book).unwrap();
        let expected =
            std::fs::read_to_string(root.join("normal-forms").join(format!("{name}.stdout")))
                .unwrap();
        let graph = std::fs::read_to_string(root.join("normal-forms").join(format!("{name}.json")))
            .unwrap();
        let oracle: Shape = facet_json::from_str(&graph).unwrap();
        for _ in 0..2 {
            let value = checked.normalize_definition("main").unwrap();
            assert_eq!(format!("{value}\n"), expected, "{name}");
            assert_eq!(shape_in_book(&value, &book).unwrap(), oracle, "{name}");
            assert_eq!(
                shape_in_book(&parse_term(expected.trim()).unwrap(), &book).unwrap(),
                oracle,
                "minted names must parse back without changing structure"
            );
            if name == "printer_fresh_binder_pick" {
                assert_fresh_match_binders(&value);
            } else {
                let Term::Ctr { args, .. } = value.as_ref() else {
                    panic!("expected EPrint");
                };
                let Term::Lam { id, body, .. } = args[1].as_ref() else {
                    panic!("expected continuation");
                };
                let Term::Ctr { args, .. } = body.as_ref() else {
                    panic!("expected EDone");
                };
                let Term::Ctr { args, .. } = args[0].as_ref() else {
                    panic!("expected successor");
                };
                let Term::Var { id: used, .. } = args[0].as_ref() else {
                    panic!("expected bound input");
                };
                assert_eq!(id, used);
            }
        }
    }
}

#[test]
fn constructor_continuation_and_nested_match_applications_preserve_complete_values() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-constructor-binders/wrappers");
    let mut applications = 0;
    for name in ["printer_fixpoint", "printer_fresh_binder_pick"] {
        let book = load(root.join(format!("{name}.bend"))).unwrap();
        let checked = check_book(&book).unwrap();
        let inputs: &[u32] = if name == "printer_fixpoint" {
            &[0, 1, 2, 7]
        } else {
            &[0, 1, 2, 3, 7]
        };
        for n in inputs {
            let expected = if name == "printer_fixpoint" {
                format!("[1, 1, 0, {}]", n + 1)
            } else {
                format!("{}n", if *n < 2 { 0 } else { 2 })
            };
            let value = checked
                .evaluate("apply", &[parse_term(&format!("{n}n")).unwrap()])
                .unwrap();
            assert_eq!(value.to_string(), expected);
            assert_eq!(
                shape_in_book(&value, &book).unwrap(),
                shape_in_book(&parse_term(&expected).unwrap(), &book).unwrap()
            );
            applications += 1;
        }
        let expected = std::fs::read_to_string(root.join(format!("{name}.stdout"))).unwrap();
        let value = checked.evaluate("main", &[]).unwrap();
        assert_eq!(format!("{value}\n"), expected);
        assert_eq!(
            shape_in_book(&value, &book).unwrap(),
            shape_in_book(&parse_term(expected.trim()).unwrap(), &book).unwrap()
        );
    }
    assert_eq!(applications, 9);
}
