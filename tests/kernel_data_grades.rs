// SPDX-License-Identifier: MPL-2.0
#[path = "support/data_grades.rs"]
mod data_grades;
#[path = "support/normalization_shape.rs"]
mod normalization_shape;
use data_grades::root;
use data_grades::source;
use facet::Facet;
use normalization_shape::Shape;
use normalization_shape::shape_in_book;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::check_book;
use teamy_bend::syntax::load;
use teamy_bend::syntax::parse;
use teamy_bend::syntax::parse_term;

fn expected(folder: &str, key: &str) -> (String, Shape) {
    let path = root().join(folder).join(key);
    (
        std::fs::read_to_string(path.with_extension("stdout")).unwrap(),
        facet_json::from_str(&std::fs::read_to_string(path.with_extension("json")).unwrap())
            .unwrap(),
    )
}

#[test]
fn original_definitions_keep_complete_data_proof_closure_and_empty_match_graphs() {
    let names: Vec<String> =
        facet_json::from_str(include_str!("fixtures/cpu-data-grades/definitions.json")).unwrap();
    assert_eq!(names.len(), 10);
    let book = load(root().join("accept_data.bend")).unwrap();
    let checked = check_book(&book).unwrap();
    for name in names {
        let (text, graph) = expected("normal-forms", &name);
        for _ in 0..2 {
            let value = checked.normalize_definition(&name).unwrap();
            assert_eq!(format!("{value}\n"), text, "{name}");
            assert_eq!(shape_in_book(&value, &book).unwrap(), graph, "{name}");
            assert_eq!(
                shape_in_book(&parse_term(text.trim()).unwrap(), &book).unwrap(),
                graph,
                "{name}"
            );
        }
    }
}

#[derive(Facet)]
struct Application {
    key: String,
    entry: String,
    arguments: Vec<String>,
    result_type: String,
    runtime: bool,
    body: String,
    value: String,
}

#[test]
fn bounded_calls_preserve_reusable_data_evidence_phantom_types_and_weakening() {
    let cases: Vec<Application> =
        facet_json::from_str(include_str!("fixtures/cpu-data-grades/applications.json")).unwrap();
    assert_eq!(cases.len(), 26);
    assert_eq!(cases.iter().filter(|case| case.runtime).count(), 20);
    let book = load(root().join("accept_data.bend")).unwrap();
    let checked = check_book(&book).unwrap();
    for case in cases {
        let args = case
            .arguments
            .iter()
            .map(|arg| parse_term(arg).unwrap())
            .collect::<Vec<_>>();
        let value = checked.evaluate(&case.entry, &args).unwrap();
        let (_, graph) = expected("expected", &case.key);
        assert_eq!(value.to_string(), case.value, "{}", case.key);
        assert_eq!(shape_in_book(&value, &book).unwrap(), graph);
        assert!(!case.result_type.is_empty() && !case.body.is_empty());
        if case.runtime {
            assert_eq!(
                shape_in_book(&checked.evaluate_data(&case.entry, &args).unwrap(), &book).unwrap(),
                graph
            );
        } else {
            // Evidence/closures remain full graph observations, not DataValue.
            teamy_bend::protocol::DataValue::from_term(&value).unwrap_err();
        }
    }
}

#[test]
fn consuming_affine_closures_and_combined_observations_match_all_graph_children() {
    let mut paths = std::fs::read_dir(root().join("fragments"))
        .unwrap()
        .map(|p| p.unwrap().path())
        .collect::<Vec<_>>();
    paths.sort();
    assert_eq!(paths.len(), 36);
    for path in paths {
        let key = path.file_stem().unwrap().to_str().unwrap();
        let book = parse(&source("fragments", key)).unwrap();
        let checked = check_book(&book).unwrap();
        let (text, graph) = expected("expected", key);
        let value = checked.evaluate("observe", &[]).unwrap();
        assert_eq!(format!("{value}\n"), text, "{key}");
        assert_eq!(shape_in_book(&value, &book).unwrap(), graph, "{key}");
        if !key.starts_with("dupe-") && !matches!(key, "weak-4" | "weak-5") {
            assert_eq!(
                shape_in_book(&checked.evaluate_data("observe", &[]).unwrap(), &book).unwrap(),
                graph,
                "{key}"
            );
        }
    }
}

#[test]
fn invalid_affine_duplication_false_evidence_and_empty_inhabitants_still_reject() {
    let paths = std::fs::read_dir(root().join("negative"))
        .unwrap()
        .map(|p| p.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "bend"))
        .collect::<Vec<_>>();
    assert_eq!(paths.len(), 11);
    for path in paths {
        let key = path.file_stem().unwrap().to_str().unwrap();
        assert!(
            check_book(&parse(&source("negative", key)).unwrap()).is_err(),
            "{key}"
        );
    }
}
