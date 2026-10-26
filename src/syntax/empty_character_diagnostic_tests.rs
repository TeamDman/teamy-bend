// SPDX-License-Identifier: MPL-2.0
use super::DiagnosticOrigins;
use super::LoadDiagnosticContent;

#[test]
fn empty_character_metadata_requires_exact_source_and_bmp_or_eof_observation() {
    for (source, column, observed) in [
        ("''", 1, "end of input"),
        ("''\n", 1, "'\n'"),
        ("''\r\n", 1, "'\r'"),
        ("'' \n", 1, "' '"),
        ("''\t", 1, "'\t'"),
        ("''# comment", 1, "'#'"),
        ("''é", 1, "'é'"),
        ("😀é''}", 3, "'}'"),
    ] {
        let mut origins = DiagnosticOrigins::default();
        origins.source("fixture", source);
        assert!(origins.empty_character("fixture", 1, column));
        let report = origins.load_failure.unwrap();
        let LoadDiagnosticContent::ExpectedObserved {
            expected,
            observed: actual,
        } = report.content
        else {
            panic!("wrong diagnostic kind")
        };
        assert_eq!(expected, "a closing '");
        assert_eq!(actual, observed);
    }
    for (source, line, column) in [
        ("'''", 1, 1),
        ("''''", 1, 1),
        ("''''''", 1, 1),
        ("''😀", 1, 1),
        ("'a'", 1, 1),
        ("''", 1, 2),
        ("''", 0, 1),
        ("''", 1, 0),
        ("''", 2, 1),
    ] {
        let mut origins = DiagnosticOrigins::default();
        origins.source("fixture", source);
        assert!(
            !origins.empty_character("fixture", line, column),
            "{source:?}:{line}:{column}"
        );
        assert!(origins.load_failure.is_none());
    }
}

#[test]
fn empty_character_contexts_preserve_raw_errors_and_pinned_optional_diagnostics() {
    #[derive(facet::Facet)]
    struct FixtureExpectation {
        case: String,
        path: String,
        kind: String,
        io: Option<bool>,
        exact: Option<bool>,
        expected: Option<String>,
        line: Option<usize>,
        column: Option<usize>,
        message: Option<String>,
        error_source: Option<String>,
    }
    let cases: Vec<FixtureExpectation> = facet_json::from_str(include_str!(
        "../../tests/fixtures/cpu-empty-character-context/cases.json"
    ))
    .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut refused = 0;
    for case in cases.into_iter().filter(|c| c.kind == "negative") {
        let path = root.join(&case.path);
        let plain = crate::syntax::load(&path).unwrap_err();
        assert_eq!(
            (plain.line, plain.column),
            (case.line.unwrap(), case.column.unwrap()),
            "{}",
            case.case
        );
        assert_eq!(plain.message, case.message.unwrap(), "{}", case.case);
        assert_eq!(
            plain.source,
            root.join(case.error_source.unwrap()).to_string_lossy()
        );
        assert_eq!(
            plain.to_string(),
            crate::syntax::load_executable(&path)
                .unwrap_err()
                .to_string()
        );
        for failure in [
            crate::syntax::load_with_diagnostics(&path).err().unwrap(),
            crate::syntax::load_executable_with_diagnostics(&path)
                .err()
                .unwrap(),
        ] {
            assert_eq!(plain.to_string(), failure.error.to_string());
            if case.exact.unwrap() {
                let report = failure.diagnostic.unwrap();
                let LoadDiagnosticContent::ExpectedObserved { expected, observed } = report.content
                else {
                    panic!("wrong diagnostic kind")
                };
                let actual = format!(
                    "Error:\n- expected : {expected}\n- observed : {observed}\nLocation:\n{}\n",
                    report.excerpt
                );
                assert_eq!(
                    actual,
                    std::fs::read_to_string(
                        root.join("cpu-empty-character-context")
                            .join(case.expected.as_ref().unwrap())
                    )
                    .unwrap(),
                    "{}",
                    case.case
                );
            }
        }
        refused += 1;
    }
    assert_eq!(refused, 22);
}
