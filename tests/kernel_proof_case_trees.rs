// SPDX-License-Identifier: MPL-2.0
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use facet::Facet;
use normalization_shape::Shape;
use normalization_shape::shape_in_book;
use std::collections::BTreeMap;
use std::path::Path;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse_term;

const NAMES: [&str; 3] = [
    "eval_case_tree_001",
    "eval_case_tree_002",
    "eval_split_apply",
];

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-proof-case-trees")
}

#[test]
fn original_match_graphs_and_text_match_the_pinned_normalizer() {
    for name in NAMES {
        let book = load(root().join(format!("{name}.bend"))).unwrap();
        let checked = check_book(&book).unwrap();
        let text =
            std::fs::read_to_string(root().join(format!("normal-forms/{name}.stdout"))).unwrap();
        let json =
            std::fs::read_to_string(root().join(format!("normal-forms/{name}.json"))).unwrap();
        let expected: Shape = facet_json::from_str(&json).unwrap();
        for _ in 0..2 {
            let value = checked.normalize_definition("main").unwrap();
            assert_eq!(format!("{value}\n"), text, "{name}");
            assert_eq!(shape_in_book(&value, &book).unwrap(), expected, "{name}");
            assert_eq!(
                shape_in_book(&parse_term(text.trim()).unwrap(), &book).unwrap(),
                expected
            );
        }
    }
}

#[derive(Facet)]
struct Application {
    name: String,
    entry: String,
    arguments: Vec<String>,
    value: String,
}

#[test]
fn closed_proofs_and_helpers_keep_their_complete_results() {
    let cases: Vec<Application> = facet_json::from_str(include_str!(
        "fixtures/cpu-proof-case-trees/closed-applications.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 52);
    for name in NAMES {
        let book = load(root().join(format!("{name}.bend"))).unwrap();
        let checked = check_book(&book).unwrap();
        for case in cases.iter().filter(|case| case.name == name) {
            let arguments = case
                .arguments
                .iter()
                .map(|x| parse_term(x).unwrap())
                .collect::<Vec<_>>();
            let value = checked.evaluate(&case.entry, &arguments).unwrap();
            let expected = parse_term(&case.value).unwrap();
            assert_eq!(value.to_string(), case.value);
            assert_eq!(
                shape_in_book(&value, &book).unwrap(),
                shape_in_book(&expected, &book).unwrap()
            );
        }
    }
}

#[test]
fn complete_constructor_results_and_scalar_projections_keep_distinct_graphs() {
    let cases: BTreeMap<String, Vec<Vec<u32>>> = facet_json::from_str(include_str!(
        "fixtures/cpu-proof-case-trees/applications.json"
    ))
    .unwrap();
    let mut count = 0;
    for name in NAMES {
        for folder in ["wrappers", "values"] {
            let path = root().join(folder).join(format!("{name}.bend"));
            let book = load(&path).unwrap();
            let checked = check_book(&book).unwrap();
            let expected = std::fs::read_to_string(path.with_extension("stdout")).unwrap();
            let expected_graph = std::fs::read_to_string(path.with_extension("json")).unwrap();
            let value = checked.evaluate("main", &[]).unwrap();
            assert_eq!(format!("{value}\n"), expected);
            // Compare the complete bounded constructor graph with an explicit stack.
            // Generic recursive JSON encoding can overflow the test-thread stack.
            let actual_graph =
                constructor_graph_json(&shape_in_book(&value, &book).unwrap()).unwrap();
            assert_eq!(actual_graph, expected_graph.strip_suffix('\n').unwrap());
            if folder == "wrappers" {
                let outputs: Vec<Vec<u32>> = facet_json::from_str(expected.trim()).unwrap();
                assert_eq!(outputs.len(), cases[name].len());
                for (inputs, wanted) in cases[name].iter().zip(outputs) {
                    let args = inputs
                        .iter()
                        .map(|x| parse_term(&format!("{x}n")).unwrap())
                        .collect::<Vec<_>>();
                    let value = checked.evaluate("apply", &args).unwrap();
                    let wanted = parse_term(&format!("{wanted:?}")).unwrap();
                    assert_eq!(
                        shape_in_book(&value, &book).unwrap(),
                        shape_in_book(&wanted, &book).unwrap()
                    );
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, 16);
}

// These closed wrapper graphs contain only constructors. Keep every name and
// ordered child; reject any other node instead of silently changing vocabulary.
fn constructor_graph_json(root: &Shape) -> Result<String, &'static str> {
    enum Piece<'a> {
        Node(&'a Shape),
        Text(&'static str),
    }
    let mut output = String::new();
    let mut pending = vec![Piece::Node(root)];
    while let Some(piece) = pending.pop() {
        match piece {
            Piece::Text(text) => output.push_str(text),
            Piece::Node(Shape::Constructor(name, children)) => {
                output.push_str("{\"Constructor\":[");
                output.push_str(&facet_json::to_string(name).map_err(|_| "invalid graph name")?);
                output.push_str(",[");
                pending.push(Piece::Text("]]}"));
                for (index, child) in children.iter().enumerate().rev() {
                    pending.push(Piece::Node(child));
                    if index > 0 {
                        pending.push(Piece::Text(","));
                    }
                }
            }
            Piece::Node(_) => return Err("non-constructor in closed wrapper graph"),
        }
    }
    Ok(output)
}

#[test]
fn iterative_constructor_graph_encoding_keeps_names_order_and_deep_children() {
    let leaf = || Shape::Constructor("Z".into(), vec![]);
    for node in [
        leaf(),
        Shape::Constructor(
            "escaped\"\\é".into(),
            vec![leaf(), Shape::Constructor("S".into(), vec![leaf()])],
        ),
    ] {
        assert_eq!(
            constructor_graph_json(&node).unwrap(),
            facet_json::to_string(&node).unwrap()
        );
    }
    let mut deep = leaf();
    for _ in 0..96 {
        deep = Shape::Constructor("S".into(), vec![deep]);
    }
    let expected =
        "{\"Constructor\":[\"S\",[".repeat(96) + "{\"Constructor\":[\"Z\",[]]}" + &"]]}".repeat(96);
    assert_eq!(constructor_graph_json(&deep).unwrap(), expected);
    assert!(constructor_graph_json(&Shape::Bound(0)).is_err());
    assert!(
        constructor_graph_json(&Shape::Constructor("C".into(), vec![Shape::EmptyMatch])).is_err()
    );
}

#[test]
fn typed_closed_proofs_check_and_normalize_while_false_claims_reject() {
    let mut count = 0;
    for name in NAMES {
        let size = if name == "eval_case_tree_002" { 8 } else { 4 };
        for i in 0..size {
            let path = root().join("proofs").join(format!("{name}-{i}.bend"));
            let book = load(&path).unwrap();
            let checked = check_book(&book).unwrap();
            let value = checked.evaluate("main", &[]).unwrap();
            assert_eq!(
                format!("{value}\n"),
                std::fs::read_to_string(path.with_extension("stdout")).unwrap()
            );
            let expected: Shape = facet_json::from_str(
                &std::fs::read_to_string(path.with_extension("json")).unwrap(),
            )
            .unwrap();
            assert_eq!(expected, Shape::Reflexivity);
            assert_eq!(shape_in_book(&value, &book).unwrap(), expected);
            count += 1;
        }
        let book = load(root().join("negative").join(format!("{name}.bend"))).unwrap();
        assert!(check_book(&book).is_err());
    }
    assert_eq!(count, 16);
}
