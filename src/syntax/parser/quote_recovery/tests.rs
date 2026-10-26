// SPDX-License-Identifier: MPL-2.0
use super::CopyBudget;
use super::MAX_HEADER_BYTES;
use super::MAX_HEADER_ITEMS;
use super::QuoteRecovery;
use crate::syntax::diagnostics::DiagnosticOrigins;
use crate::syntax::diagnostics::LoadDiagnosticContent;
use crate::syntax::parser::ParseInput;
use crate::syntax::parser::SourceOrigin;
use crate::syntax::parser::Templates;
use crate::syntax::parser::lex_tokens_mode;
use crate::syntax::parser::parse_into;
use std::collections::BTreeMap;
use std::rc::Rc;

fn capture(
    source: &str,
    book: &crate::kernel::Book,
    templates: &Templates,
) -> Option<QuoteRecovery> {
    let mut prefix = None;
    lex_tokens_mode(source, "<quote>", Some(&mut prefix), true).unwrap_err();
    QuoteRecovery::capture(
        prefix?,
        &ParseInput {
            source,
            label: "<quote>",
            namespace: "",
            aliases: BTreeMap::new(),
            origin: SourceOrigin::Standalone,
        },
        book,
        321,
        templates,
    )
}

fn origins(source: &str) -> Templates {
    let mut origins = DiagnosticOrigins::default();
    origins.source("<quote>", source);
    Templates {
        diagnostics: Some(origins),
        ..Templates::default()
    }
}

#[test]
fn quote_recovery_preserves_raw_errors_and_contextual_first_refusals() {
    for (source, expected) in [
        ("def main() -> Char: ''''\n", "'def', 'type' or 'law'"),
        ("def main() -> Char: ('''')\n", "')'"),
        (
            "law main: Char\ndef main(): ''''\n",
            "'def', 'type' or 'law'",
        ),
        (
            "def first(->) -> Char: 'a'\ndef main() -> Char: ''''\n",
            "a name",
        ),
        (
            "def main() -> Char: 'a'\ndef main() -> Char: ''''\n",
            "a fresh name (duplicate declaration: main)",
        ),
    ] {
        let mut templates = origins(source);
        let mut book = crate::kernel::Book::default();
        let mut fresh = 0;
        let error = parse_into(
            ParseInput {
                source,
                label: "<quote>",
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
        .unwrap_err();
        let plain = crate::syntax::parse(source).unwrap_err();
        assert_eq!(
            (error.line, error.column, &error.message),
            (plain.line, plain.column, &plain.message)
        );
        let report = templates
            .diagnostics
            .unwrap()
            .load_failure(error)
            .diagnostic
            .unwrap();
        let LoadDiagnosticContent::ExpectedObserved {
            expected: actual, ..
        } = report.content
        else {
            panic!("expected syntax metadata")
        };
        assert_eq!(actual, expected, "{source}");
    }
}

#[test]
fn quote_recovery_has_independent_origins_ids_and_shared_immutable_terms() {
    let source = "def main() -> Char: ''''\n";
    let book = crate::syntax::parse("law imported: Type\ndef prior(~x: Type) -> Type: x").unwrap();
    let mut templates = origins(source);
    let marker = crate::kernel::term(crate::kernel::Term::Ref("marker".into()));
    let tracker = &templates.diagnostics.as_ref().unwrap().tracker;
    tracker.track(&marker);
    let before = tracker.source(&marker);
    let recovery = capture(source, &book, &templates).unwrap();
    assert_eq!(recovery.fresh, 321);
    recovery
        .templates
        .diagnostics
        .as_ref()
        .unwrap()
        .tracker
        .reusable_marker(99);
    assert!(!tracker.is_reusable_marker(99));
    let crate::kernel::Declaration::Def(original) = &book.declarations[0] else {
        panic!()
    };
    let crate::kernel::Declaration::Def(copied) = &recovery.book.declarations[0] else {
        panic!()
    };
    assert!(Rc::ptr_eq(&original.ty, &copied.ty));
    assert!(recovery.diagnostic().is_some());
    assert_eq!(tracker.source(&marker), before);
    assert_eq!(book.declarations.len(), 1);
    assert_eq!((templates.active, templates.instances), (0, 0));
    let error = crate::syntax::parse("def()").unwrap_err();
    assert!(
        templates
            .diagnostics
            .as_mut()
            .unwrap()
            .load_failure(error)
            .diagnostic
            .is_none()
    );
}

#[test]
fn quote_recovery_preserves_imported_collisions_and_template_cache_identity() {
    let source = "def main() -> Char: prior(~Char)\n''''\n";
    let mut templates = origins(source);
    let mut book = crate::kernel::Book::default();
    let mut fresh = 7;
    parse_into(
        ParseInput {
            source: "def prior(~x: Type) -> Type: x\ndef occupied() -> Char: 'a'",
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
    let tokens = Rc::clone(&templates.declarations["prior"].tokens);
    let mut recovery = capture(source, &book, &templates).unwrap();
    assert!(Rc::ptr_eq(
        &tokens,
        &recovery.templates.declarations["prior"].tokens
    ));
    recovery
        .templates
        .declarations
        .get_mut("prior")
        .unwrap()
        .instances
        .insert("scratch".into(), "scratch-only".into());
    assert!(templates.declarations["prior"].instances.is_empty());
    assert!(recovery.diagnostic().is_none());
    assert_eq!((templates.instances, templates.active), (0, 0));
    assert!(Rc::ptr_eq(&tokens, &templates.declarations["prior"].tokens));

    let collision = "def occupied() -> Char: ''''\n";
    templates.diagnostics = origins(collision).diagnostics;
    // Imported declarations are a deliberate recovery barrier; the primary
    // parser's existing duplicate error must remain authoritative.
    assert!(
        capture(collision, &book, &templates)
            .unwrap()
            .diagnostic()
            .is_none()
    );
}

#[test]
fn quote_recovery_does_not_cross_depth_or_incomplete_atom_guards() {
    for source in [
        format!("def main() -> Char: {}''''\n", "+ ".repeat(64)),
        "def main() -> List(Char): ['''']\n".into(),
        "def main() -> Char: f('''')\n".into(),
    ] {
        let templates = origins(&source);
        assert!(
            capture(&source, &crate::kernel::Book::default(), &templates)
                .and_then(QuoteRecovery::diagnostic)
                .is_none()
        );
    }
    // Existing raw default-stack guard, without a thread-stack override.
    let source = format!("{}''''", "+ ".repeat(64));
    assert!(
        crate::syntax::parse_term(&source)
            .unwrap_err()
            .message
            .contains("resource limit of 64")
    );
}

#[test]
fn diagnostic_quote_recovery_keeps_parser_and_import_depth_on_default_stack() {
    let source = format!(
        "def main() -> Char: {}''''{}\n",
        "(".repeat(32),
        ")".repeat(32)
    );
    let mut templates = origins(&source);
    let mut book = crate::kernel::Book::default();
    let mut fresh = 0;
    let error = parse_into(
        ParseInput {
            source: &source,
            label: "<quote>",
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
    .unwrap_err();
    assert!(error.message.contains("resource limit of 64"));
    assert!(
        templates
            .diagnostics
            .unwrap()
            .load_failure(error)
            .diagnostic
            .is_none()
    );
    let scratch =
        std::env::temp_dir().join(format!("bend-quote-import-depth-{}", std::process::id()));
    std::fs::create_dir(&scratch).unwrap();
    for index in 0..63 {
        std::fs::write(
            scratch.join(format!("f{index}.bend")),
            format!("import f{}.bend as Next\n", index + 1),
        )
        .unwrap();
    }
    std::fs::write(scratch.join("f63.bend"), "def main() -> Char: ''''\n").unwrap();
    let path = scratch.join("f0.bend");
    let plain = crate::syntax::load(&path).unwrap_err();
    for failure in [
        crate::syntax::load_with_diagnostics(&path).err().unwrap(),
        crate::syntax::load_executable_with_diagnostics(&path)
            .err()
            .unwrap(),
    ] {
        assert_eq!(failure.error.to_string(), plain.to_string());
        assert!(failure.error.source.ends_with("f63.bend"));
        assert!(
            matches!(failure.diagnostic.unwrap().content, LoadDiagnosticContent::ExpectedObserved { expected, .. } if expected == "'def', 'type' or 'law'")
        );
    }
    std::fs::remove_dir_all(scratch).unwrap();
}

#[test]
fn quote_recovery_preflights_actual_book_and_template_copy_limits() {
    let source = "def main() -> Char: ''''\n";
    let templates = origins(source);
    let mut book = crate::syntax::parse("law other: Type").unwrap();
    let crate::kernel::Declaration::Def(definition) = &mut book.declarations[0] else {
        panic!()
    };
    definition.completion_parameters = Some(vec![(String::new(), 0); MAX_HEADER_ITEMS]);
    assert!(capture(source, &book, &templates).is_none());
    let crate::kernel::Declaration::Def(definition) = &mut book.declarations[0] else {
        panic!()
    };
    definition.completion_parameters = None;
    definition.name = "x".repeat(MAX_HEADER_BYTES);
    assert!(capture(source, &book, &templates).is_none());

    let mut book = crate::kernel::Book::default();
    let mut templates = origins(source);
    let mut fresh = 0;
    parse_into(
        ParseInput {
            source: "def prior(~x: Type) -> Type: x",
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
    let template = templates.declarations.get_mut("prior").unwrap();
    template
        .instances
        .insert("x".repeat(MAX_HEADER_BYTES), "instance".into());
    assert!(capture(source, &book, &templates).is_none());
}

#[test]
fn quote_recovery_stops_at_semantic_and_imported_law_boundaries() {
    for source in [
        "def prior(): ''''\n",
        "def template(~x: Type) -> Type: x\ndef main() -> Char: ''''\n",
        "def main() -> Char: import \"missing.c\"\n''''\n",
        "def f(x: Char) -> Char: x\ndef main() -> Char: ''''\n",
        "def main() -> Char: 129n\n''''\n",
    ] {
        let book = crate::syntax::parse("law prior: Char").unwrap();
        let templates = origins(source);
        assert!(
            capture(source, &book, &templates)
                .unwrap()
                .diagnostic()
                .is_none(),
            "{source}"
        );
    }
}

#[test]
fn quote_recovery_caps_full_sources_and_every_header_copy() {
    let source = "def main() -> Char: ''''\n";
    let book = crate::kernel::Book::default();
    let within = format!("#{}\n{source}", " ".repeat(4096 - source.len() - 2));
    assert_eq!(within.len(), 4096);
    assert!(capture(&within, &book, &origins(&within)).is_some());
    let over = format!(" {within}");
    assert!(capture(&over, &book, &origins(&over)).is_none());
    // The original display source can be larger than the cleaned parser input.
    assert!(capture(source, &book, &origins(&over)).is_none());
    let late = format!("{source}#{}", "x".repeat(4096));
    assert!(capture(&late, &book, &origins(&late)).is_none());
    let tokens = format!("{}''''", "a ".repeat(64));
    assert!(capture(&tokens, &book, &origins(&tokens)).is_none());
    let mut budget = CopyBudget {
        items: MAX_HEADER_ITEMS,
        bytes: MAX_HEADER_BYTES,
    };
    assert!(budget.string(&"x".repeat(MAX_HEADER_BYTES)).is_some());
    assert!(budget.string("x").is_none());
    let mut budget = CopyBudget {
        items: MAX_HEADER_ITEMS,
        bytes: MAX_HEADER_BYTES,
    };
    for _ in 0..MAX_HEADER_ITEMS {
        assert!(budget.string("").is_some());
    }
    assert!(budget.string("").is_none());
}
