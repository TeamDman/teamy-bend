// SPDX-License-Identifier: MPL-2.0
use super::super::ParseInput;
use super::super::SourceOrigin;
use super::super::Templates;
use super::super::lex;
use super::super::parse_into;
use crate::kernel::Book;
use crate::syntax::diagnostics::DiagnosticLoadError;
use crate::syntax::diagnostics::DiagnosticOrigins;
use crate::syntax::diagnostics::LoadDiagnosticContent;
use crate::syntax::load;
use crate::syntax::load_executable_with_diagnostics;
use crate::syntax::load_with_diagnostics;
use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

fn diagnostic(source: &str) -> DiagnosticLoadError {
    diagnostic_with_seed(source, false)
}

fn diagnostic_with_seed(source: &str, seed: bool) -> DiagnosticLoadError {
    let mut templates = Templates {
        diagnostics: Some(DiagnosticOrigins::default()),
        ..Templates::default()
    };
    let mut book = Book::default();
    let mut fresh = 0;
    let mut foreign = BTreeMap::new();
    let mut tags = BTreeMap::new();
    if seed {
        let origins = templates.diagnostics.as_mut().unwrap();
        origins.source("<stale>", "stale");
        origins.expected_syntax("<stale>", 1, "stale", "stale");
    }
    let error = parse_into(
        ParseInput {
            source,
            label: "<lexical-test>",
            namespace: "",
            aliases: BTreeMap::new(),
            origin: SourceOrigin::Standalone,
        },
        &mut book,
        &mut fresh,
        &mut templates,
        Some(&mut foreign),
        Some(&mut tags),
    )
    .unwrap_err();
    let original = lex(source, "<lexical-test>").unwrap_err();
    assert_eq!(format!("{error:?}"), format!("{original:?}"));
    assert!(templates.lexical_diagnostic.is_none());
    assert!(templates.declarations.is_empty());
    assert_eq!((templates.instances, templates.active), (0, 0));
    assert!(foreign.is_empty());
    templates.diagnostics.as_mut().unwrap().load_failure(error)
}

#[test]
fn lexical_context_keeps_public_errors_and_exact_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/cpu-contextual-lexical-diagnostics");
    let cases: Vec<String> = facet_json::from_str(include_str!(
        "../../../../tests/fixtures/cpu-contextual-lexical-diagnostics/negative-cases.json"
    ))
    .unwrap();
    assert_eq!(cases.len(), 22);
    for case in cases {
        let path = root.join(&case);
        let plain = load(&path).unwrap_err();
        for failure in [
            load_with_diagnostics(&path).err().unwrap(),
            load_executable_with_diagnostics(&path).err().unwrap(),
        ] {
            assert_eq!(
                format!("{plain:?}"),
                format!("{:?}", failure.error),
                "{case}"
            );
            let report = failure.diagnostic.unwrap();
            let LoadDiagnosticContent::ExpectedObserved { expected, observed } = report.content
            else {
                panic!("expected syntax context")
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
fn lexical_context_stops_before_expansion_and_host_paths() {
    for source in [
        "def template(~x: Type) -> Type: x\n0x",
        "law main: U32\ndef main(): import \"missing.c\"\n0x",
        "law f: for x: U32 U32\ndef f(x): x\n0x",
        "type B is Data: T{}\nlaw f: for x: B B\ndef f(x):\n  match x:\n    case T{}: T{}\n0x",
        "law main: U32\ndef main(): 1nx",
        "law main: U32\ndef main(): 1.0x",
        "law main: U32\ndef main(): 129n\n0x",
    ] {
        assert!(diagnostic(source).diagnostic.is_none(), "{source}");
    }
    let long_string = format!("law main: U32\ndef main(): \"{}\"\n0x", "a".repeat(129));
    assert!(diagnostic(&long_string).diagnostic.is_none());
}

#[test]
fn lexical_context_caps_include_trivia_and_terminal_token() {
    let input = "law main: U32\ndef main(): 0x";
    assert!(diagnostic(input).diagnostic.is_some());
    let within = format!("#{}\n{input}", " ".repeat(4096 - input.len() - 2));
    assert_eq!(within.len(), 4096);
    assert!(diagnostic(&within).diagnostic.is_some());
    assert!(diagnostic(&format!(" {within}")).diagnostic.is_none());
    assert!(
        diagnostic(&format!("law main: U32\ndef main(): {}x", "0".repeat(8192)))
            .diagnostic
            .is_none()
    );
    // 12 tokens in the final program; four per preceding simple law.
    let mut prefix = String::new();
    for index in 0..20 {
        prefix.push_str(&format!("law p{index}: U32\n"));
    }
    assert!(diagnostic(&format!("{prefix}{input}")).diagnostic.is_none());
    assert!(
        diagnostic(&format!(
            "law main: U32\ndef main(): {}x\n0x",
            "+ ".repeat(64)
        ))
        .diagnostic
        .is_none()
    );
}

#[test]
fn lexical_context_preserves_guard_refusals_and_clears_stale_metadata() {
    for expression in [
        "129n".to_owned(),
        format!("\"{}\"", "a".repeat(129)),
        "128n+128n+1n".to_owned(),
    ] {
        let original = crate::syntax::parse_term(&expression).unwrap_err();
        assert!(original.message.contains("resource limit"), "{original}");
        let source = format!("law main: U32\ndef main(): {expression}\n0x");
        let mut prefix = None;
        super::super::lex_tokens(&source, "<guard>", Some(&mut prefix)).unwrap_err();
        assert!(
            prefix.is_some(),
            "must reach parser, not the presentation cap"
        );
        assert!(diagnostic_with_seed(&source, true).diagnostic.is_none());
    }
    for count in [63, 64] {
        let source = format!("{}0x", "a ".repeat(count));
        let mut prefix = None;
        super::super::lex_tokens(&source, "<cap>", Some(&mut prefix)).unwrap_err();
        assert_eq!(prefix.is_some(), count == 63, "fault token counts toward64");
    }
}

#[test]
fn lexical_context_barriers_stop_before_mutating_shared_templates() {
    use super::super::Parser;
    use super::super::lex_tokens;
    use std::collections::BTreeSet;
    use std::collections::VecDeque;
    use std::rc::Rc;

    for source in [
        "def new_template(~x: Type) -> Type: x\n0x",
        "law main: U32\ndef main(): import \"missing.c\"\n0x",
        "law f: for x: U32 U32\ndef f(x): x\n0x",
        "type B is Data: T{}\nlaw f: for x: B B\ndef f(x):\n  match x:\n    case T{}: T{}\n0x",
        "law main: Type\ndef main(): prior(~Type)\n0x",
        "def imported(): Type\n0x",
    ] {
        let mut templates = Templates::default();
        let mut book = Book::default();
        let mut fresh = 0;
        parse_into(
            ParseInput {
                source: "law imported: Type\ndef prior(~x: Type) -> Type: x",
                label: "<import>",
                namespace: "",
                aliases: BTreeMap::new(),
                origin: SourceOrigin::Standalone,
            },
            &mut book,
            &mut fresh,
            &mut templates,
            None,
            None,
        )
        .unwrap();
        let shared = Rc::clone(&templates.declarations["prior"].tokens);
        let mut origins = DiagnosticOrigins::default();
        origins.source("<prefix>", source);
        templates.diagnostics = Some(origins);
        let mut prefix = None;
        lex_tokens(source, "<prefix>", Some(&mut prefix)).unwrap_err();
        let mut prefix = prefix.expect("eligible prefix");
        prefix.diagnostic.initial_declarations = book.declarations.len();
        templates.lexical_diagnostic = Some(prefix.diagnostic);
        let mut foreign = BTreeMap::new();
        let mut tags = BTreeMap::new();
        let error = Parser {
            tokens: prefix.tokens,
            at: 0,
            label: "<prefix>".into(),
            namespace: String::new(),
            aliases: BTreeMap::new(),
            scope: Vec::new(),
            fresh: &mut fresh,
            display_fresh: 0,
            book: &mut book,
            depth: 0,
            reusable: Vec::new(),
            reusable_markers: BTreeSet::new(),
            templates: &mut templates,
            instance_name: None,
            instance_arguments: VecDeque::new(),
            compile_bindings: BTreeMap::new(),
            foreign: Some(&mut foreign),
            constructor_tags: Some(&mut tags),
            origin: SourceOrigin::Local(PathBuf::from("unused-foreign-root")),
        }
        .declarations()
        .unwrap_err();
        assert_eq!(
            error.message, "unsupported lexical diagnostic prefix",
            "{source}"
        );
        assert!(templates.lexical_diagnostic.as_ref().unwrap().unsupported());
        assert_eq!(templates.declarations.len(), 1);
        assert!(Rc::ptr_eq(&shared, &templates.declarations["prior"].tokens));
        assert!(templates.declarations["prior"].instances.is_empty());
        assert_eq!((templates.instances, templates.active), (0, 0));
        assert!(foreign.is_empty());
    }
}
