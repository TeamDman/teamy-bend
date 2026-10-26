// SPDX-License-Identifier: MPL-2.0
#[path = "support/normalization_shape.rs"]
mod normalization_shape;
use std::path::Path;
use teamy_bend::kernel as normalization_kernel;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load;
use teamy_bend::syntax::load_executable;

#[derive(facet::Facet)]
struct RawError {
    line: usize,
    column: usize,
    message: String,
}
#[derive(facet::Facet)]
struct FixtureExpectation {
    case: String,
    path: String,
    kind: String,
    io: bool,
    escaped: Option<String>,
    baseline_error: Option<RawError>,
    raw_error_unchanged: Option<bool>,
}
fn cases() -> Vec<FixtureExpectation> {
    facet_json::from_str(include_str!("fixtures/cpu-raw-apostrophe/cases.json")).unwrap()
}

#[test]
fn raw_apostrophe_controls_preserve_full_value_graphs_and_native_results() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-raw-apostrophe");
    let positives: Vec<_> = cases()
        .into_iter()
        .filter(|c| c.kind == "positive")
        .collect();
    assert_eq!(positives.len(), 12);
    for case in positives {
        let paths = [root.join(&case.path), root.join(case.escaped.unwrap())];
        if case.io {
            for path in &paths {
                check_book(&load(path).unwrap()).expect_err("IO remains outside strict proofs");
            }
        } else {
            let shapes: Vec<_> = paths
                .iter()
                .map(|path| {
                    let book = load(path).unwrap();
                    let normal = check_book(&book)
                        .unwrap()
                        .normalize_definition("main")
                        .unwrap();
                    normalization_shape::shape_in_book(&normal, &book).unwrap()
                })
                .collect();
            assert_eq!(shapes[0], shapes[1], "{}", case.case);
        }
        let mut results = Vec::new();
        for path in &paths {
            let checked = check_executable(&load_executable(path).unwrap()).unwrap();
            for _ in 0..2 {
                let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
                let status = checked
                    .run_main(&mut stdout, &mut stderr, &|| false)
                    .unwrap();
                assert_eq!(status, 0, "{}", case.case);
                assert!(stderr.is_empty(), "{}", case.case);
                assert!(!stdout.is_empty(), "{}", case.case);
                results.push((status, stdout, stderr));
            }
        }
        assert!(results.windows(2).all(|p| p[0] == p[1]), "{}", case.case);
    }
}

#[test]
fn rejected_quote_controls_preserve_errors_with_two_explicit_tokenization_changes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-raw-apostrophe");
    let negatives: Vec<_> = cases()
        .into_iter()
        .filter(|c| c.kind == "negative")
        .collect();
    assert_eq!(negatives.len(), 17);
    for case in negatives {
        let path = root.join(&case.path);
        let expected = if case.raw_error_unchanged.unwrap() {
            case.baseline_error.unwrap()
        } else {
            match case.case.as_str() {
                "name-slot" => RawError {
                    line: 3,
                    column: 5,
                    message: "expected a name, found \"'''\"".into(),
                },
                "import-later-error" => RawError {
                    line: 4,
                    column: 11,
                    message: "expected a name, found \"->\"".into(),
                },
                _ => panic!("unclassified raw error change"),
            }
        };
        for error in [
            load(&path).unwrap_err(),
            load_executable(&path).unwrap_err(),
        ] {
            assert_eq!(
                (error.line, error.column),
                (expected.line, expected.column),
                "{}",
                case.case
            );
            assert_eq!(error.message, expected.message, "{}", case.case);
            assert_eq!(error.source, path.to_string_lossy(), "{}", case.case);
            assert_eq!(
                error.to_string(),
                format!(
                    "{}:{}:{}: {}",
                    path.display(),
                    expected.line,
                    expected.column,
                    expected.message
                )
            );
        }
    }
}
