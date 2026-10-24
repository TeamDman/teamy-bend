// SPDX-License-Identifier: MPL-2.0
#[path = "support/normalization_shape.rs"]
mod normalization_shape;

use facet::Facet;
use normalization_shape::Shape;
use normalization_shape::shape_in_book;
use std::path::Path;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse_term;

const NAMES: [&str; 3] = [
    "dependent_telescope",
    "capture_price_twin_000",
    "capture_price_twin_001",
];

fn root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-telescope-capture")
}

#[derive(Facet)]
struct Definition {
    name: String,
    entry: String,
    key: String,
}

#[test]
fn complete_graphs_preserve_telescope_fields_and_capture_binders() {
    let definitions: Vec<Definition> = facet_json::from_str(include_str!(
        "fixtures/cpu-telescope-capture/definitions.json"
    ))
    .unwrap();
    assert_eq!(definitions.len(), 8);
    for definition in definitions {
        let book = load(root().join(format!("{}.bend", definition.name))).unwrap();
        let checked = check_book(&book).unwrap();
        let path = root().join("normal-forms");
        let text =
            std::fs::read_to_string(path.join(format!("{}.stdout", definition.key))).unwrap();
        let expected: Shape = facet_json::from_str(
            &std::fs::read_to_string(path.join(format!("{}.json", definition.key))).unwrap(),
        )
        .unwrap();
        for _ in 0..2 {
            let value = checked.normalize_definition(&definition.entry).unwrap();
            assert_eq!(format!("{value}\n"), text, "{}", definition.key);
            assert_eq!(shape_in_book(&value, &book).unwrap(), expected);
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
fn valid_dependent_records_and_branch_closures_match_original_applications() {
    let cases: Vec<Application> = facet_json::from_str(include_str!(
        "fixtures/cpu-telescope-capture/closed-applications.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 20);
    let mut count = 0;
    for name in NAMES {
        let book = load(root().join(format!("{name}.bend"))).unwrap();
        let checked = check_book(&book).unwrap();
        for case in cases.iter().filter(|case| case.name == name) {
            let args = case
                .arguments
                .iter()
                .map(|value| parse_term(value).unwrap())
                .collect::<Vec<_>>();
            let actual = checked.evaluate(&case.entry, &args).unwrap();
            assert_eq!(actual.to_string(), case.value);
            let expected = parse_term(&case.value).unwrap();
            assert_eq!(
                shape_in_book(&actual, &book).unwrap(),
                shape_in_book(&expected, &book).unwrap()
            );
            count += 1;
        }
    }
    assert_eq!(count, 20);
}

#[test]
fn typed_callers_complete_values_and_scalar_observations_keep_all_children() {
    let applications: Vec<String> = facet_json::from_str(include_str!(
        "fixtures/cpu-telescope-capture/typed-applications.json"
    ))
    .unwrap();
    assert_eq!(applications.len(), 20);
    let mut cases = applications;
    for name in NAMES {
        for folder in ["values", "wrappers", "applicability"] {
            cases.push(format!("{folder}/{name}"));
        }
    }
    assert_eq!(cases.len(), 29);
    for case in cases {
        let path = root().join(format!("{case}.bend"));
        let book = load(&path).unwrap();
        let checked = check_book(&book).unwrap();
        let value = checked.evaluate("main", &[]).unwrap();
        let expected_text = std::fs::read_to_string(path.with_extension("stdout")).unwrap();
        let expected_graph = std::fs::read_to_string(path.with_extension("json")).unwrap();
        assert_eq!(format!("{value}\n"), expected_text, "{case}");
        // Keep complete ordered constructor graphs without recursively decoding
        // deeply nested JSON on the test thread. No child or field is omitted.
        let actual_graph = constructor_graph_json(&shape_in_book(&value, &book).unwrap()).unwrap();
        assert_eq!(
            actual_graph,
            expected_graph.trim_end_matches('\n'),
            "{case}"
        );
    }
}

#[test]
fn false_first_field_and_impossible_positive_scrutinee_witnesses_reject() {
    for name in NAMES {
        let book = load(root().join("negative").join(format!("{name}.bend"))).unwrap();
        assert!(check_book(&book).is_err(), "{name}");
    }
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
