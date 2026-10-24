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

const CASES: [&str; 3] = [
    "eta_expand_print",
    "toplevel_ref_print",
    "hoas_binder_refresh",
];

#[test]
fn function_values_keep_distinct_open_graphs_and_shadowed_binder_identity() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-function-values");
    let mut graphs = Vec::new();
    for name in CASES {
        let book = load(root.join(format!("{name}.bend"))).unwrap();
        let checked = check_book(&book).unwrap();
        let expected =
            std::fs::read_to_string(root.join("normal-forms").join(format!("{name}.stdout")))
                .unwrap();
        let graph = std::fs::read_to_string(root.join("normal-forms").join(format!("{name}.json")))
            .unwrap();
        let oracle: Shape = facet_json::from_str(&graph).unwrap();
        for _ in 0..2 {
            for actual in [
                checked.normalize_definition("main").unwrap(),
                checked.evaluate("main", &[]).unwrap(),
            ] {
                assert_eq!(format!("{actual}\n"), expected, "{name}");
                assert_eq!(shape_in_book(&actual, &book).unwrap(), oracle, "{name}");
                if name == "hoas_binder_refresh" {
                    let Term::Lam {
                        id: outer, body, ..
                    } = actual.as_ref()
                    else {
                        panic!("expected outer lambda");
                    };
                    let Term::Lam {
                        id: inner, body, ..
                    } = body.as_ref()
                    else {
                        panic!("expected inner lambda");
                    };
                    let Term::Var { id, .. } = body.as_ref() else {
                        panic!("expected bound variable");
                    };
                    assert_ne!(outer, inner);
                    assert_eq!(id, inner);
                }
            }
        }
        graphs.push(oracle);
    }
    // Conversion may admit eta equality; normalization must not erase it.
    assert_ne!(graphs[0], graphs[1]);
    assert_eq!(graphs[1], Shape::Reference("id".into()));
    assert_eq!(
        graphs[2],
        Shape::Lambda(Box::new(Shape::Lambda(Box::new(Shape::Bound(0)))))
    );
}

#[test]
fn function_value_applications_keep_complete_closed_results() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-function-values");
    let mut applications = 0;
    for name in CASES {
        let book = load(root.join(format!("{name}.bend"))).unwrap();
        let checked = check_book(&book).unwrap();
        let vectors: Vec<Vec<String>> = if name == "hoas_binder_refresh" {
            [0_u32, 7, u32::MAX]
                .into_iter()
                .flat_map(|a| [0_u32, 7, u32::MAX].map(|b| vec![a.to_string(), b.to_string()]))
                .collect()
        } else {
            [0, 1, 2, 7].map(|n| vec![format!("{n}n")]).into()
        };
        for vector in vectors {
            let args = vector
                .iter()
                .map(|x| parse_term(x).unwrap())
                .collect::<Vec<_>>();
            let actual = checked.evaluate("main", &args).unwrap();
            let expected = parse_term(vector.last().unwrap()).unwrap();
            assert_eq!(
                actual.to_string(),
                expected.to_string(),
                "{name}: {vector:?}"
            );
            assert_eq!(
                shape_in_book(&actual, &book).unwrap(),
                shape_in_book(&expected, &book).unwrap(),
                "{name}: {vector:?}"
            );
            applications += 1;
        }
        let book = load(root.join("wrappers").join(format!("{name}.bend"))).unwrap();
        let actual = check_book(&book).unwrap().evaluate("main", &[]).unwrap();
        let expected =
            std::fs::read_to_string(root.join("wrappers").join(format!("{name}.stdout"))).unwrap();
        assert_eq!(format!("{actual}\n"), expected, "{name}");
        assert_eq!(
            shape_in_book(&actual, &book).unwrap(),
            shape_in_book(&parse_term(expected.trim()).unwrap(), &book).unwrap(),
            "{name}"
        );
    }
    assert_eq!(applications, 17);
}
