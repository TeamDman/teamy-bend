// SPDX-License-Identifier: MPL-2.0
use super::LoadDiagnosticContent;
use std::path::Path;
use std::path::PathBuf;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-malformed-literal-diagnostics")
}

#[test]
fn literal_failures_keep_public_errors_and_exact_consumed_source() {
    let cases: Vec<String> = facet_json::from_str(include_str!(
        "../../tests/fixtures/cpu-malformed-literal-diagnostics/negative-cases.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 13);
    for case in cases {
        let path = root().join(&case);
        let plain = crate::syntax::load(&path).unwrap_err();
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
            assert_eq!(plain.to_string(), failure.error.to_string(), "{case}");
            let report = failure.diagnostic.unwrap();
            let LoadDiagnosticContent::ExpectedObserved { expected, observed } = report.content
            else {
                panic!("expected a literal syntax failure")
            };
            assert_eq!(
                format!(
                    "Error:\n- expected : {expected}\n- observed : {observed}\nLocation:\n{}\n",
                    report.excerpt
                ),
                std::fs::read_to_string(path.with_extension("stderr")).unwrap(),
                "{case}"
            );
        }
    }
}

#[test]
fn literal_diagnostics_do_not_guess_unicode_contracts() {
    let cases: Vec<String> = facet_json::from_str(include_str!(
        "../../tests/fixtures/cpu-malformed-literal-diagnostics/unresolved-cases.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 3);
    for case in cases {
        let path = root().join(&case);
        let plain = crate::syntax::load(&path).unwrap_err();
        for failure in [
            crate::syntax::load_with_diagnostics(&path).err().unwrap(),
            crate::syntax::load_executable_with_diagnostics(&path)
                .err()
                .unwrap(),
        ] {
            assert_eq!(plain.to_string(), failure.error.to_string());
            assert!(failure.diagnostic.is_none(), "{case}");
        }
    }
}

#[test]
fn literal_refusal_preserves_public_cursor_and_message() {
    let overflow = crate::syntax::parse_term("4294967296\n").unwrap_err();
    assert_eq!((overflow.line, overflow.column), (2, 1));
    assert_eq!(
        overflow.message,
        "U32 literal must be at most 4294967295: number too large to fit in target type"
    );
    let escape = crate::syntax::parse_term("\"a\\qb\"\n").unwrap_err();
    assert_eq!((escape.line, escape.column), (2, 1));
    assert_eq!(escape.message, "unknown escape: \\q");
    assert_eq!(
        crate::syntax::parse_term("\"\\u{xyz}\"")
            .unwrap_err()
            .message,
        "invalid Unicode escape: invalid digit found in string"
    );
}

#[test]
fn uppercase_refusals_preserve_public_loader_errors_and_first_error_positions() {
    let directory = root()
        .parent()
        .unwrap()
        .join("cpu-uppercase-unicode/controls");
    for name in [
        "empty",
        "plus",
        "nonhex",
        "nine-digits",
        "overflow",
        "missing-brace",
        "missing-close",
        "invalid-before-valid",
        "valid-before-invalid",
    ] {
        let path = directory.join(format!("{name}.bend"));
        let escape = if matches!(name, "invalid-before-valid" | "valid-before-invalid") {
            'q'
        } else {
            'U'
        };
        let plain = crate::syntax::load(&path).unwrap_err();
        assert_eq!((plain.line, plain.column), (4, 1), "{name}");
        assert_eq!(plain.message, format!("unknown escape: \\{escape}"));
        assert_eq!(
            plain.to_string(),
            format!("{}:4:1: unknown escape: \\{escape}", path.display())
        );
        assert_eq!(
            format!("{plain:?}"),
            format!("{:?}", crate::syntax::load_executable(&path).unwrap_err())
        );
        for failure in [
            crate::syntax::load_with_diagnostics(&path).err().unwrap(),
            crate::syntax::load_executable_with_diagnostics(&path)
                .err()
                .unwrap(),
        ] {
            assert_eq!(format!("{:?}", failure.error), format!("{plain:?}"));
            assert!(failure.diagnostic.is_some(), "{name}");
        }
    }
}

#[test]
fn literal_and_expression_guards_remain_on_the_default_stack() {
    let source = format!("\"{}\"", "a".repeat(129));
    assert!(
        crate::syntax::parse_term(&source)
            .unwrap_err()
            .message
            .contains("128 characters")
    );
    assert!(
        crate::syntax::parse_term("129n")
            .unwrap_err()
            .message
            .contains("128")
    );
    for source in [
        format!("{}x", "+ ".repeat(64)),
        format!("{}x", "@x:Type -> ".repeat(64)),
        format!("{}x", "%r: Type; ".repeat(64)),
    ] {
        assert!(
            crate::syntax::parse_term(&source)
                .unwrap_err()
                .message
                .contains("resource limit of 64")
        );
    }
    for source in [
        "0",
        "4294967295",
        "'é'",
        "\"\\n\\t\\r\\0\\\\\\'\\\"\\u{1F600}\"",
    ] {
        crate::syntax::parse_term(source).unwrap();
    }
}

#[test]
fn uppercase_literal_diagnostic_eligibility_preserves_first_upstream_failure() {
    #[derive(facet::Facet)]
    struct FixtureExpectation {
        case: String,
        path: String,
        kind: String,
        exact: bool,
        expected: String,
    }
    let cases: Vec<FixtureExpectation> = facet_json::from_str(include_str!(
        "../../tests/fixtures/cpu-uppercase-escape-diagnostics/cases.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 39);
    let fixtures = root().parent().unwrap().to_path_buf();
    for case in cases {
        let path = fixtures.join(&case.path);
        let plain = crate::syntax::load(&path).unwrap_err();
        assert_eq!(
            format!("{plain:?}"),
            format!("{:?}", crate::syntax::load_executable(&path).unwrap_err()),
            "{}",
            case.case
        );
        for failure in [
            crate::syntax::load_with_diagnostics(&path).err().unwrap(),
            crate::syntax::load_executable_with_diagnostics(&path)
                .err()
                .unwrap(),
        ] {
            assert_eq!(format!("{:?}", failure.error), format!("{plain:?}"));
            if case.exact {
                let report = failure
                    .diagnostic
                    .unwrap_or_else(|| panic!("missing diagnostic: {}", case.case));
                let LoadDiagnosticContent::ExpectedObserved { expected, observed } = report.content
                else {
                    panic!("expected syntax diagnostic: {}", case.case)
                };
                assert_eq!(
                    format!(
                        "Error:\n- expected : {expected}\n- observed : {observed}\nLocation:\n{}\n",
                        report.excerpt
                    ),
                    std::fs::read_to_string(
                        fixtures
                            .join("cpu-uppercase-escape-diagnostics")
                            .join(case.expected.clone())
                    )
                    .unwrap(),
                    "{}",
                    case.case
                );
            } else if case.kind == "unresolved" {
                assert!(failure.diagnostic.is_none(), "{}", case.case);
            }
        }
    }
}
