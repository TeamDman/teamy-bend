// SPDX-License-Identifier: MPL-2.0
use super::lex;
use super::lex_tokens_mode;

#[test]
fn raw_apostrophes_keep_source_spelling_and_adjacent_token_boundaries() {
    for count in [1, 2, 3] {
        let source = format!("f({})", "'''".repeat(count));
        let tokens = lex(&source, "literal").unwrap();
        let literals: Vec<_> = tokens.iter().filter(|token| token.text == "'''").collect();
        assert_eq!(literals.len(), count);
        for token in literals {
            assert_eq!(token.end - token.start, 3);
            assert_eq!(&source[token.start..token.end], token.text);
            assert_eq!(token.column, token.start + 1);
        }
    }
    let raw = super::parse_term("'''").unwrap();
    let escaped = super::parse_term("'\\''").unwrap();
    assert_eq!(format!("{raw:?}"), format!("{escaped:?}"));
}

#[test]
fn malformed_quote_runs_restore_whole_legacy_tokenization() {
    for count in [1, 2, 4, 5, 7, 8] {
        for suffix in ["", "\n", " )"] {
            let source = format!("{}{}", "'".repeat(count), suffix);
            assert_eq!(
                format!("{:?}", lex(&source, "literal")),
                format!("{:?}", lex_tokens_mode(&source, "literal", None, false)),
                "{source:?}"
            );
        }
    }
    for source in ["\"'''\"", "# '''\n'\\q'", "\"'''\" 0x", "''' 0x"] {
        assert_eq!(
            format!("{:?}", lex(source, "literal")),
            format!("{:?}", lex_tokens_mode(source, "literal", None, false)),
            "{source:?}"
        );
    }
}

#[test]
fn quote_trial_keeps_existing_literal_and_depth_guards() {
    let source = format!("# '''\n{}x", "+ ".repeat(64));
    assert!(
        super::parse_term(&source)
            .unwrap_err()
            .message
            .contains("resource limit of 64")
    );
    let source = format!("\"'''{}\"", "a".repeat(126));
    assert!(
        super::parse_term(&source)
            .unwrap_err()
            .message
            .contains("128 characters")
    );
}

#[test]
fn canonical_depth_refusal_and_public_legacy_error_are_separate() {
    let source = format!("{}'''", "+ ".repeat(64));
    let mut book = super::Book::default();
    let mut fresh = 0;
    let mut templates = super::Templates::default();
    let mut parser = super::Parser {
        tokens: lex(&source, "<expression>").unwrap(),
        at: 0,
        label: "<expression>".into(),
        namespace: String::new(),
        aliases: super::BTreeMap::new(),
        scope: Vec::new(),
        fresh: &mut fresh,
        display_fresh: 0,
        book: &mut book,
        depth: 0,
        reusable: Vec::new(),
        reusable_markers: super::BTreeSet::new(),
        templates: &mut templates,
        instance_name: None,
        instance_arguments: super::VecDeque::new(),
        compile_bindings: super::BTreeMap::new(),
        foreign: None,
        constructor_tags: None,
        origin: super::SourceOrigin::Standalone,
    };
    assert!(
        parser
            .expression(0)
            .unwrap_err()
            .message
            .contains("resource limit of 64")
    );
    let legacy = lex_tokens_mode(&source, "<expression>", None, false).unwrap_err();
    assert_eq!(
        super::parse_term(&source).unwrap_err().to_string(),
        legacy.to_string()
    );
}

#[test]
fn restored_lexical_error_retains_the_actual_parser_diagnostic() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-raw-apostrophe/negative");
    for fixture in ["earlier-grammar.bend", "later-grammar.bend"] {
        let path = root.join(fixture);
        let plain = super::load(&path).unwrap_err();
        let tracked = super::load_with_diagnostics(&path).err().unwrap();
        assert_eq!(tracked.error.to_string(), plain.to_string());
        assert!(tracked.diagnostic.is_some());
    }
}

#[test]
fn restored_lexical_error_cannot_reuse_stale_metadata_after_a_guard_failure() {
    let source = format!("law main: Char\ndef main(): {}'''", "+ ".repeat(64));
    let mut book = super::Book::default();
    let mut fresh = 0;
    let mut origins = crate::syntax::diagnostics::DiagnosticOrigins::default();
    origins.source("stale", "stale");
    origins.expected_syntax("stale", 1, "stale", "stale");
    let mut templates = super::Templates {
        diagnostics: Some(origins),
        ..super::Templates::default()
    };
    let error = super::parse_into(
        super::ParseInput {
            source: &source,
            label: "<guard>",
            namespace: "",
            aliases: super::BTreeMap::new(),
            origin: super::SourceOrigin::Standalone,
        },
        &mut book,
        &mut fresh,
        &mut templates,
        None,
        None,
    )
    .unwrap_err();
    let legacy = lex_tokens_mode(&source, "<guard>", None, false).unwrap_err();
    assert_eq!(error.to_string(), legacy.to_string());
    assert!(
        templates
            .diagnostics
            .unwrap()
            .load_failure(error)
            .diagnostic
            .is_none()
    );
}
