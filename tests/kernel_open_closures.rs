// SPDX-License-Identifier: MPL-2.0
#[path = "support/closure_composition_cases.rs"]
mod closure_composition_cases;
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use facet::Facet;
use normalization_shape::Shape;
use normalization_shape::shape_in_book;
use std::path::Path;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::Quant;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse_term;

#[test]
fn open_composition_and_computed_calls_keep_exact_graphs_and_text() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-open-closures");
    for name in ["open_closure_pipe", "stuck_split_mint"] {
        let book = load(root.join(format!("{name}.bend"))).unwrap();
        let checked = check_book(&book).unwrap();
        let text =
            std::fs::read_to_string(root.join("normal-forms").join(format!("{name}.stdout")))
                .unwrap();
        let graph = std::fs::read_to_string(root.join("normal-forms").join(format!("{name}.json")))
            .unwrap();
        let expected: Shape = facet_json::from_str(&graph).unwrap();
        for _ in 0..2 {
            let value = checked.normalize_definition("main").unwrap();
            assert_eq!(format!("{value}\n"), text);
            assert_eq!(shape_in_book(&value, &book).unwrap(), expected);
            assert_eq!(
                shape_in_book(&parse_term(text.trim()).unwrap(), &book).unwrap(),
                expected
            );
            if name == "open_closure_pipe" {
                closure_composition_cases::assert_binders(&value);
            }
        }
    }
}

#[test]
fn individual_closed_compositions_and_recursive_applications_preserve_complete_results() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-open-closures/wrappers");
    let book = load(root.join("open_closure_pipe.bend")).unwrap();
    let checked = check_book(&book).unwrap();
    let cases = closure_composition_cases::cases();
    assert_eq!(cases.len(), 27);
    for case in cases {
        let args = [
            parse_term(case.first).unwrap(),
            parse_term(case.second).unwrap(),
            parse_term(&case.input.to_string()).unwrap(),
        ];
        let actual = checked.evaluate("apply", &args).unwrap();
        let expected = parse_term(&case.expected.to_string()).unwrap();
        assert_eq!(actual.to_string(), expected.to_string());
        assert_eq!(
            shape_in_book(&actual, &book).unwrap(),
            shape_in_book(&expected, &book).unwrap()
        );
    }
    // Each call receives the existing fixed budget. This does not qualify the
    // separate aggregate main, whose normalization exhausts its single budget.
    let book = load(root.join("stuck_split_mint.bend")).unwrap();
    let checked = check_book(&book).unwrap();
    for input in [0, 1, 2, 7, 16] {
        let actual = checked
            .evaluate("apply", &[parse_term(&format!("{input}n")).unwrap()])
            .unwrap();
        let expected = parse_term(if input == 0 { "True{}" } else { "False{}" }).unwrap();
        assert_eq!(actual.to_string(), expected.to_string());
        assert_eq!(
            shape_in_book(&actual, &book).unwrap(),
            shape_in_book(&expected, &book).unwrap()
        );
    }
}

#[derive(Facet)]
struct TypePrintCase {
    source: String,
    printed: String,
}

// Keep this type-only vocabulary local. The shared normal-form comparator
// deliberately refuses universes and forall terms in its existing controls.
#[derive(Debug, Eq, PartialEq)]
enum TypeShape {
    Bound(usize),
    Universe(Quant),
    Forall(Quant, Box<Self>, Box<Self>),
}

fn type_shape(value: &TermRef, scope: &mut Vec<usize>, depth: usize) -> TypeShape {
    assert!(depth <= 128, "type comparison depth exhausted");
    match value.as_ref() {
        Term::Var { id, .. } => TypeShape::Bound(scope.iter().rev().position(|x| x == id).unwrap()),
        Term::Typ(quant) => {
            let Term::Qua(quant) = quant.as_ref() else {
                panic!("expected explicit universe quantity");
            };
            TypeShape::Universe(*quant)
        }
        Term::All {
            quant,
            id,
            domain,
            body,
            ..
        } => {
            let domain = type_shape(domain, scope, depth + 1);
            scope.push(*id);
            let body = type_shape(body, scope, depth + 1);
            scope.pop();
            TypeShape::Forall(*quant, Box::new(domain), Box::new(body))
        }
        _ => panic!("term outside the type-printer comparison vocabulary"),
    }
}

#[test]
fn higher_order_domains_keep_parentheses_and_dependent_binding_structure() {
    let cases: Vec<TypePrintCase> =
        facet_json::from_str(include_str!("fixtures/cpu-open-closures/type-printer.json")).unwrap();
    assert_eq!(cases.len(), 6);
    for case in cases {
        let value = parse_term(&case.source).unwrap();
        assert_eq!(value.to_string(), case.printed);
        assert_eq!(
            type_shape(&value, &mut Vec::new(), 0),
            type_shape(&parse_term(&value.to_string()).unwrap(), &mut Vec::new(), 0)
        );
    }
}
