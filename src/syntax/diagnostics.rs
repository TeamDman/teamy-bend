// SPDX-License-Identifier: MPL-2.0
//! Optional loader-owned origins for source diagnostic nodes. No core AST or
//! acceptance metadata is changed, and no global pointer registry is used.
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::kernel::origins::MatchSite;
use crate::kernel::origins::TermOrigins;
use std::collections::BTreeMap;
use std::path::Path;
use std::rc::Rc;
use std::rc::Weak;

#[derive(Default)]
pub(crate) struct DiagnosticOrigins {
    pub(crate) tracker: TermOrigins,
    load_failure: Option<LoadDiagnostic>,
    // Loader-wide fresh binder IDs survive Body cloning/substitution unchanged.
    // This index owns source text only; no core term or strong term edge is added.
    local_binders: BTreeMap<usize, (Rc<str>, usize)>,
    binder_types: BTreeMap<usize, usize>,
    sources: BTreeMap<String, Rc<str>>,
    nodes: BTreeMap<usize, TermOrigin>,
    match_locations: BTreeMap<usize, TermOrigin>,
}

#[derive(Debug)]
pub(crate) struct DiagnosticLoadError {
    pub(crate) error: super::ParseError,
    pub(crate) diagnostic: Option<Box<LoadDiagnostic>>,
}

#[derive(Debug)]
pub(crate) struct LoadDiagnostic {
    pub(crate) content: LoadDiagnosticContent,
    pub(crate) excerpt: String,
}

#[derive(Debug)]
pub(crate) enum LoadDiagnosticContent {
    Message(String),
    ExpectedObserved { expected: String, observed: String },
}

pub(super) struct ImportSite<'a> {
    pub(super) source: &'a str,
    pub(super) line: usize,
    pub(super) from: &'a Path,
    pub(super) relative: &'a str,
}

struct TermOrigin {
    // Retain the allocation: an identity cannot be recycled while this source
    // map is paired with the check result. Identity is never dereferenced.
    _node: Weak<Term>,
    source: Rc<str>,
    line: usize,
}

impl DiagnosticOrigins {
    pub(super) fn binder_type(&mut self, id: usize, source: Option<usize>) {
        if let Some(source) = source {
            self.binder_types.insert(id, source);
        }
    }
    pub(super) fn type_source(&self, id: usize) -> Option<usize> {
        self.binder_types.get(&id).copied()
    }
    pub(super) fn anchor(&mut self, label: &str, line: usize) -> usize {
        // Reserve a weak source identity, never a semantic variable/binder ID.
        let anchor = crate::kernel::term(Term::Ref(String::new()));
        self.node(&anchor, label, line);
        self.tracker.source(&anchor)
    }
    pub(super) fn source(&mut self, label: &str, source: &str) {
        self.sources
            .entry(label.to_owned())
            .or_insert_with(|| Rc::from(source));
    }

    pub(super) fn node(&mut self, node: &TermRef, label: &str, line: usize) {
        if let Some(source) = self.sources.get(label) {
            self.tracker.track(node);
            self.nodes
                .entry(Rc::as_ptr(node) as usize)
                .or_insert_with(|| TermOrigin {
                    _node: Rc::downgrade(node),
                    source: Rc::clone(source),
                    line,
                });
        }
    }

    pub(super) fn match_location(&mut self, value: &TermRef, label: &str, line: usize) {
        if let Some(source) = self.sources.get(label) {
            self.match_locations.insert(
                self.tracker.source(value),
                TermOrigin {
                    _node: Rc::downgrade(value),
                    source: Rc::clone(source),
                    line,
                },
            );
        }
    }

    pub(super) fn application(&mut self, value: &TermRef, label: &str, line: usize) {
        let mut spine = Vec::new();
        let mut head = value;
        while let Term::App(function, _) = head.as_ref() {
            spine.push(head);
            head = function;
        }
        // Like upstream parse_grow, keep the function's starting position,
        // including a parenthesized function whose text starts on another line.
        let position = self
            .nodes
            .get(&self.tracker.source(head))
            .map(|origin| (Rc::clone(&origin.source), origin.line))
            .or_else(|| {
                self.sources
                    .get(label)
                    .map(|source| (Rc::clone(source), line))
            });
        if let Some((source, line)) = position {
            for node in spine {
                self.tracker.track(node);
                self.nodes
                    .entry(Rc::as_ptr(node) as usize)
                    .or_insert_with(|| TermOrigin {
                        _node: Rc::downgrade(node),
                        source: Rc::clone(&source),
                        line,
                    });
            }
        }
    }

    pub(super) fn known_source(&self, value: &TermRef) -> Option<usize> {
        let source = self.tracker.source(value);
        self.nodes.contains_key(&source).then_some(source)
    }

    pub(super) fn occurrence_parent(&mut self, value: &TermRef, source: usize) -> bool {
        let Some(origin) = self.nodes.get(&source) else {
            return false;
        };
        let position = (Rc::clone(&origin.source), origin.line);
        self.tracker.track(value);
        self.nodes
            .entry(Rc::as_ptr(value) as usize)
            .or_insert_with(|| TermOrigin {
                _node: Rc::downgrade(value),
                source: position.0,
                line: position.1,
            });
        true
    }

    pub(super) fn grown(
        &mut self,
        value: &TermRef,
        head: &TermRef,
        explicit: Option<usize>,
        label: &str,
        line: usize,
    ) {
        if let Some(source) = explicit
            && self.occurrence_parent(value, source)
        {
            return;
        }
        let position = self
            .nodes
            .get(&self.tracker.source(head))
            .map(|origin| (Rc::clone(&origin.source), origin.line));
        if let Some((source, line)) = position {
            self.tracker.track(value);
            self.nodes
                .entry(Rc::as_ptr(value) as usize)
                .or_insert_with(|| TermOrigin {
                    _node: Rc::downgrade(value),
                    source,
                    line,
                });
        } else {
            self.node(value, label, line);
        }
    }

    pub(super) fn unknown_pattern(&mut self, name: &str, value: &TermRef) {
        // This parser rejection is not recovered: the private loader carries
        // the structured cause alongside its original public ParseError.
        self.load_failure =
            self.excerpt(self.tracker.source(value))
                .map(|excerpt| LoadDiagnostic {
                    content: LoadDiagnosticContent::Message(format!(
                        "a declared constructor (unknown: {name})"
                    )),
                    excerpt,
                });
    }

    pub(super) fn unsupported_pattern(&mut self, value: &TermRef, source: Option<usize>) {
        self.load_failure = source
            .and_then(|source| self.excerpt(source))
            .or_else(|| self.excerpt(self.tracker.source(value)))
            .map(|excerpt| LoadDiagnostic {
                content: LoadDiagnosticContent::ExpectedObserved {
                    expected: "a pattern (a binder or a constructor)".into(),
                    observed: value.to_string(),
                },
                excerpt,
            });
    }

    pub(crate) fn match_excerpt(&self, site: MatchSite) -> Option<String> {
        match site {
            MatchSite::Expression(identity) => self.excerpt(identity),
            MatchSite::Statement(identity) => {
                let origin = self.match_locations.get(&identity)?;
                source_excerpt(&origin.source, origin.line)
            }
        }
    }

    pub(super) fn local_binder(&mut self, id: usize, label: &str, line: usize) {
        self.binder(id, label, line);
        if let Some(source) = self.sources.get(label) {
            self.local_binders.insert(id, (Rc::clone(source), line));
        }
    }

    pub(super) fn binder(&mut self, id: usize, label: &str, line: usize) {
        // A weakly held source anchor reserves an identity without retaining a
        // semantic term or adding a field to the AST. Generated lambda/let
        // nodes inherit this binder occurrence's span during flattening.
        let anchor = crate::kernel::term(Term::Var {
            name: String::new(),
            id,
        });
        self.node(&anchor, label, line);
        self.tracker.binder_source(id, Rc::as_ptr(&anchor) as usize);
    }

    pub(super) fn rejected_local_binding(&mut self, id: usize) {
        self.load_failure = self.local_binders.get(&id).and_then(|(source, line)| {
            source_excerpt(source, *line).map(|excerpt| LoadDiagnostic {
                content: LoadDiagnosticContent::Message("a parameter or field scrutinee (a match cannot scrutinize a local binder: give it its own def)".into()),
                excerpt,
            })
        });
    }

    pub(super) fn rejected_computed_scrutinee(&mut self, identity: usize) {
        self.load_failure = self.excerpt(identity).map(|excerpt| LoadDiagnostic {
            content: LoadDiagnosticContent::Message("a parameter or field scrutinee (a match cannot scrutinize a computed value: give it its own def)".into()),
            excerpt,
        });
    }

    pub(super) fn rejected_variable_scrutinee(&mut self, identity: usize) {
        self.load_failure = self.excerpt(identity).map(|excerpt| LoadDiagnostic {
            content: LoadDiagnosticContent::Message("match scrutinees in binder order (this variable is unbound, consumed, or out of order: reorder the match)".into()),
            excerpt,
        });
    }

    pub(super) fn rejected_constructor_scrutinee(&mut self, identity: usize) {
        // Upstream reports a constructor at its enclosing match; a virtual
        // destructuring-let match instead inherits its value's source span.
        let excerpt = self
            .match_locations
            .get(&identity)
            .and_then(|origin| source_excerpt(&origin.source, origin.line))
            .or_else(|| self.excerpt(identity));
        self.load_failure = excerpt.map(|excerpt| LoadDiagnostic {
            content: LoadDiagnosticContent::Message("an undestructed scrutinee (this value is already a constructor: bind its fields directly; if an outer match destructed it, fold the pattern into the outer case)".into()),
            excerpt,
        });
    }

    pub(super) fn row_arity(
        &mut self,
        label: &str,
        line: usize,
        colon_column: usize,
        count: usize,
    ) {
        self.load_failure = self.sources.get(label).and_then(|source| {
            // parse_terms leaves the pinned parser immediately after ':'. Use
            // the original source character, including whitespace, CRLF or EOF;
            // tokenization has already skipped that character in the port.
            let observed = source
                .split_inclusive('\n')
                .nth(line - 1)?
                .chars()
                .nth(colon_column)
                .map_or_else(|| "end of input".into(), |c| format!("'{c}'"));
            source_excerpt(source, line).map(|excerpt| LoadDiagnostic {
                content: LoadDiagnosticContent::ExpectedObserved {
                    expected: format!("{count} patterns (one per scrutinee)"),
                    observed,
                },
                excerpt,
            })
        });
    }

    pub(super) fn empty_character(&mut self, label: &str, line: usize, column: usize) -> bool {
        let eligible = self.sources.get(label).is_some_and(|source| {
            let Some(line) = line
                .checked_sub(1)
                .and_then(|n| source.split_inclusive('\n').nth(n))
            else {
                return false;
            };
            let Some(start) = column.checked_sub(1) else {
                return false;
            };
            let mut chars = line.chars().skip(start);
            // The pinned Char parser consumes the second quote as code 39.
            // A third quote closes it; an astral next code cannot be rendered
            // as the single UTF-16 unit observed by pinned parse_fail.
            chars.next() == Some('\'')
                && chars.next() == Some('\'')
                && chars.next().is_none_or(|c| c != '\'' && c.len_utf16() == 1)
        });
        if eligible {
            self.expected_after_token(label, line, column + 1, "a closing '");
        }
        eligible
    }

    pub(super) fn incomplete_character(
        &mut self,
        label: &str,
        line: usize,
        column: usize,
        first: char,
    ) -> bool {
        let eligible = self.sources.get(label).is_some_and(|source| {
            // The direct lexical-prefix path also reaches this barrier. Bound
            // the whole original display source before excerpt construction.
            if source.len() > 4096 || matches!(first, '\\' | '\n' | '\r') {
                return false;
            }
            let Some(line) = line
                .checked_sub(1)
                .and_then(|n| source.split_inclusive('\n').nth(n))
            else {
                return false;
            };
            let Some(start) = column.checked_sub(1) else {
                return false;
            };
            let mut chars = line.chars().skip(start);
            // Pinned parse_char consumes one raw code point; parse_take then
            // requires a quote without skipping trivia. Inspect only that
            // boundary, without decoding any escape or constructing a value.
            chars.next() == Some('\'')
                && chars.next() == Some(first)
                && chars.next().is_none_or(|c| c != '\'' && c.len_utf16() == 1)
        });
        if eligible {
            self.expected_after_token(label, line, column + 1, "a closing '");
        }
        eligible
    }

    pub(super) fn expected_after_token(
        &mut self,
        label: &str,
        line: usize,
        after_token: usize,
        expected: &str,
    ) {
        self.load_failure = self.sources.get(label).and_then(|source| {
            // Pinned parse_fail observes the character immediately after the
            // token, before the next parser step skips whitespace/comments. Use the original
            // display line: tokenization skips trivia and loading normalizes
            // CRLF and missing final newlines. Keep the public parser cursor.
            let observed = source.split_inclusive('\n').nth(line.checked_sub(1)?)?;
            let length = observed.chars().count();
            if after_token > length || after_token == length && observed.ends_with('\n') {
                return None;
            }
            let observed = observed
                .chars()
                .nth(after_token)
                .map_or_else(|| "end of input".into(), |c| format!("'{c}'"));
            source_excerpt(source, line).map(|excerpt| LoadDiagnostic {
                content: LoadDiagnosticContent::ExpectedObserved {
                    expected: expected.to_owned(),
                    observed,
                },
                excerpt,
            })
        });
    }

    pub(super) fn expected_token(&mut self, label: &str, line: usize, expected: &str, found: &str) {
        self.expected_syntax(label, line, &format!("'{expected}'"), found);
    }

    pub(super) fn expected_syntax(
        &mut self,
        label: &str,
        line: usize,
        expected: &str,
        found: &str,
    ) {
        self.load_failure = self.sources.get(label).and_then(|source| {
            // Loader parsing uses a newline-normalized buffer. The public
            // ParseError retains that position; the source display uses the
            // actual EOF, including an original missing final newline.
            let line = if found.is_empty() {
                source.split('\n').count()
            } else {
                line
            };
            source_excerpt(source, line).map(|excerpt| LoadDiagnostic {
                content: LoadDiagnosticContent::ExpectedObserved {
                    expected: expected.to_owned(),
                    observed: found
                        .chars()
                        .next()
                        .map_or_else(|| "end of input".into(), |first| format!("'{first}'")),
                },
                excerpt,
            })
        });
    }

    #[inline(never)]
    #[expect(
        clippy::case_sensitive_file_extension_comparisons,
        reason = "Pinned POSIX normalization uses an exact case-sensitive string suffix; host extension rules differ"
    )]
    pub(super) fn import_syntax(&mut self, source: &str, line: usize, path: Option<&str>) {
        // Failed imports use the original file, including earlier imports.
        // Do not register it: successful loading later records the source with
        // consumed headers blanked, and that source index is first-write-wins.
        // Keep formatting out of the recursive loader's stack frame.
        let Some(authored) = diagnostic_import_line(source, line) else {
            return;
        };
        let (expected, observed) = if let Some(path) = path {
            if path.ends_with(".bend")
                || path.contains('\\')
                || path.split('/').any(|part| matches!(part, "." | ".."))
            {
                // POSIX path normalization and host Path::extension have
                // separate edge cases. This presentation fix does not resolve
                // their acceptance contract.
                return;
            }
            ("an import of a .bend file", path)
        } else {
            (
                "an import ('import Base', or 'import <path> as <Name>')",
                authored.trim(),
            )
        };
        self.load_failure = source_excerpt(source, line).map(|excerpt| LoadDiagnostic {
            content: LoadDiagnosticContent::ExpectedObserved {
                expected: expected.to_owned(),
                observed: format!("'{observed}'"),
            },
            excerpt,
        });
    }

    #[inline(never)]
    pub(super) fn missing_import(&mut self, site: &ImportSite<'_>) {
        let Some(authored) = diagnostic_import_line(site.source, site.line) else {
            return;
        };
        if authored.split_whitespace().nth(1) != Some(site.relative)
            || !simple_import_path(site.relative)
        {
            return;
        }
        // Pinned book_load normalizes only the import token, then concatenates
        // the authored parent's directory. These simple spellings are unchanged
        // by that normalization. Ambiguous spellings keep the original error.
        let target = if site.relative.starts_with('/') {
            site.relative.to_owned()
        } else {
            let Some(from) = site.from.to_str().filter(|from| simple_import_path(from)) else {
                return;
            };
            let prefix = from.rfind('/').map_or("", |at| &from[..=at]);
            format!("{prefix}{}", site.relative)
        };
        self.load_failure = source_excerpt(site.source, site.line).map(|excerpt| LoadDiagnostic {
            content: LoadDiagnosticContent::Message(format!("no such file: {target}")),
            excerpt,
        });
    }

    pub(super) fn load_failure(&mut self, error: super::ParseError) -> DiagnosticLoadError {
        DiagnosticLoadError {
            error,
            diagnostic: self.load_failure.take().map(Box::new),
        }
    }

    pub(super) fn clear_load_failure(&mut self) {
        self.load_failure = None;
    }

    // Quote recovery must not share the mutable origin tracker or scan a large
    // original display source hidden behind the cleaned parser buffer.
    pub(super) fn isolated_load_context(&self, label: &str) -> Option<Self> {
        let source = self.sources.get(label)?;
        if source.len() > 4096 {
            return None;
        }
        let mut context = Self::default();
        context.sources.insert(label.into(), Rc::clone(source));
        Some(context)
    }

    pub(super) fn replace_load_failure(&mut self, diagnostic: LoadDiagnostic) {
        self.load_failure = Some(diagnostic);
    }

    pub(crate) fn excerpt(&self, identity: usize) -> Option<String> {
        let origin = self.nodes.get(&identity)?;
        source_excerpt(&origin.source, origin.line)
    }
}

fn diagnostic_import_line(source: &str, line: usize) -> Option<&str> {
    let authored = source.split('\n').nth(line.checked_sub(1)?)?;
    if authored
        .chars()
        .any(|c| c == '\u{feff}' || c.is_whitespace() && !c.is_ascii_whitespace())
        || authored
            .split_whitespace()
            .nth(1)
            .is_some_and(|token| token.contains('#'))
        || authored.find('#').is_some_and(|at| {
            authored[..at]
                .chars()
                .next_back()
                .is_some_and(|c| !c.is_whitespace())
        })
    {
        // These whitespace/comment tokens differ between Rust and the pinned
        // loader's regex. Preserve the existing error rather than guess.
        None
    } else {
        Some(authored)
    }
}

fn simple_import_path(path: &str) -> bool {
    !path.contains(['\\', ':'])
        && path
            .strip_prefix('/')
            .unwrap_or(path)
            .split('/')
            .all(|part| !part.is_empty() && !matches!(part, "." | ".."))
}

fn source_excerpt(source: &str, at: usize) -> Option<String> {
    let lines = source.split('\n').collect::<Vec<_>>();
    if at == 0 || at > lines.len() {
        return None;
    }
    let begin = at.saturating_sub(1).max(1);
    let end = (at + 1).min(lines.len());
    let width = end.to_string().len();
    Some(
        (begin..=end)
            .map(|line| {
                let marker = if line == at { ">| " } else { " | " };
                format!("{line:>width$}{marker}{}", lines[line - 1])
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn known_sources_require_recorded_positions_and_keep_term_ownership() {
        let mut origins = super::DiagnosticOrigins::default();
        let value = crate::kernel::term(crate::kernel::Term::Qnt);
        assert_eq!(origins.known_source(&value), None);
        origins.source("<quantity>", "Quant");
        origins.node(&value, "<quantity>", 1);
        assert_eq!(
            origins.known_source(&value),
            Some(std::rc::Rc::as_ptr(&value) as usize)
        );
        let unsourced = crate::kernel::term(crate::kernel::Term::Qnt);
        origins.tracker.track(&unsourced);
        assert_eq!(origins.known_source(&unsourced), None);
        assert_eq!(std::rc::Rc::strong_count(&value), 1);
        assert_eq!(std::rc::Rc::strong_count(&unsourced), 1);
    }

    #[derive(facet::Facet)]
    struct ExpectedLoadError {
        case: String,
        line: usize,
        column: usize,
        message: String,
        source: String,
    }

    #[test]
    fn reserved_name_metadata_preserves_public_errors_and_original_sources() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-reserved-name-diagnostics");
        let cases: Vec<ExpectedLoadError> = facet_json::from_str(include_str!(
            "../../tests/fixtures/cpu-reserved-name-diagnostics/public-errors.json"
        ))
        .unwrap();
        assert_eq!(cases.len(), 22);
        for case in cases {
            let path = root.join(&case.case);
            let plain = crate::syntax::load(&path).unwrap_err();
            assert_eq!((plain.line, plain.column), (case.line, case.column));
            assert_eq!(plain.message, case.message);
            assert_eq!(std::path::Path::new(&plain.source), root.join(&case.source));
            let executable = crate::syntax::load_executable(&path).unwrap_err();
            assert_eq!(plain.to_string(), executable.to_string());
            for tracked in [
                crate::syntax::load_with_diagnostics(&path).err().unwrap(),
                crate::syntax::load_executable_with_diagnostics(&path)
                    .err()
                    .unwrap(),
            ] {
                assert_eq!(
                    plain.to_string(),
                    tracked.error.to_string(),
                    "{}",
                    case.case
                );
                if case.case.starts_with("controls/") {
                    assert!(tracked.diagnostic.is_none(), "{}", case.case);
                    continue;
                }
                let report = tracked.diagnostic.unwrap();
                let super::LoadDiagnosticContent::ExpectedObserved { expected, observed } =
                    report.content
                else {
                    panic!("expected the pinned name diagnostic");
                };
                assert_eq!(
                    format!(
                        "Error:\n- expected : {expected}\n- observed : {observed}\nLocation:\n{}\n",
                        report.excerpt
                    ),
                    std::fs::read_to_string(path.with_extension("stderr")).unwrap(),
                    "{}",
                    case.case
                );
            }
        }
    }

    #[test]
    fn import_header_metadata_preserves_public_errors_and_original_sources() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-import-header-diagnostics");
        let cases: Vec<ExpectedLoadError> = facet_json::from_str(include_str!(
            "../../tests/fixtures/cpu-import-header-diagnostics/public-errors.json"
        ))
        .unwrap();
        assert_eq!(cases.len(), 19);
        for case in cases {
            let path = root.join(&case.case);
            let plain = crate::syntax::load(&path).unwrap_err();
            assert_eq!((plain.line, plain.column), (case.line, case.column));
            assert_eq!(plain.message, case.message);
            assert_eq!(std::path::Path::new(&plain.source), root.join(&case.source));
            assert_eq!(
                plain.to_string(),
                crate::syntax::load_executable(&path)
                    .unwrap_err()
                    .to_string()
            );
            for tracked in [
                crate::syntax::load_with_diagnostics(&path).err().unwrap(),
                crate::syntax::load_executable_with_diagnostics(&path)
                    .err()
                    .unwrap(),
            ] {
                assert_eq!(
                    plain.to_string(),
                    tracked.error.to_string(),
                    "{}",
                    case.case
                );
                if case.case.starts_with("controls/") {
                    if case.case == "controls/parser-after-import.bend" {
                        let report = tracked.diagnostic.unwrap();
                        assert!(report.excerpt.starts_with("3 | \n4>| def main()"));
                    } else {
                        assert!(tracked.diagnostic.is_none(), "{}", case.case);
                    }
                    continue;
                }
                let report = tracked.diagnostic.unwrap();
                let super::LoadDiagnosticContent::ExpectedObserved { expected, observed } =
                    report.content
                else {
                    panic!("expected an import diagnostic")
                };
                assert_eq!(
                    format!(
                        "Error:\n- expected : {expected}\n- observed : {observed}\nLocation:\n{}\n",
                        report.excerpt
                    ),
                    std::fs::read_to_string(path.with_extension("stderr")).unwrap(),
                    "{}",
                    case.case
                );
            }
        }
    }

    #[test]
    fn import_header_metadata_preserves_earlier_filesystem_priority() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-import-header-diagnostics/controls/earlier-file.bend");
        let plain = crate::syntax::load(&path).unwrap_err();
        for tracked in [
            crate::syntax::load_with_diagnostics(&path).err().unwrap(),
            crate::syntax::load_executable_with_diagnostics(&path)
                .err()
                .unwrap(),
        ] {
            assert_eq!(plain.to_string(), tracked.error.to_string());
            assert!(tracked.error.source.ends_with("absent.bend"));
            if super::simple_import_path(path.to_str().unwrap()) {
                let report = tracked.diagnostic.unwrap();
                let super::LoadDiagnosticContent::Message(message) = report.content else {
                    panic!("expected missing-file provenance")
                };
                assert_eq!(
                    message,
                    format!(
                        "no such file: {}",
                        path.parent().unwrap().join("absent.bend").display()
                    )
                );
                assert!(report.excerpt.contains("2>| import absent.bend as D"));
                assert!(!report.excerpt.contains("3>| import Wat"));
            } else {
                assert!(tracked.diagnostic.is_none());
            }
        }
    }

    #[test]
    fn import_header_tracking_preserves_successful_complete_books() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-import-header-diagnostics");
        for case in [
            "positive/simple.bend",
            "positive/chain.bend",
            "positive/base-crlf.bend",
            "controls/checker-after-import.bend",
        ] {
            let path = root.join(case);
            let plain = crate::syntax::load(&path).unwrap();
            let (tracked, _) = crate::syntax::load_with_diagnostics(&path).unwrap();
            assert_eq!(format!("{plain:?}"), format!("{tracked:?}"), "{case}");
        }
    }

    #[test]
    fn lambda_binder_metadata_preserves_public_errors_and_sources() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-lambda-binder-diagnostics");
        let cases: Vec<ExpectedLoadError> = facet_json::from_str(include_str!(
            "../../tests/fixtures/cpu-lambda-binder-diagnostics/public-errors.json"
        ))
        .unwrap();
        assert_eq!(cases.len(), 19);
        for case in cases {
            let path = root.join(&case.case);
            let plain = crate::syntax::load(&path).unwrap_err();
            assert_eq!((plain.line, plain.column), (case.line, case.column));
            assert_eq!(plain.message, case.message);
            assert_eq!(std::path::Path::new(&plain.source), root.join(&case.source));
            assert_eq!(
                plain.to_string(),
                crate::syntax::load_executable(&path)
                    .unwrap_err()
                    .to_string()
            );
            for tracked in [
                crate::syntax::load_with_diagnostics(&path).err().unwrap(),
                crate::syntax::load_executable_with_diagnostics(&path)
                    .err()
                    .unwrap(),
            ] {
                assert_eq!(
                    plain.to_string(),
                    tracked.error.to_string(),
                    "{}",
                    case.case
                );
                if case.case.starts_with("controls/") {
                    if case.case == "controls/earlier-syntax.bend" {
                        assert!(matches!(tracked.diagnostic.unwrap().content,
                            super::LoadDiagnosticContent::ExpectedObserved { expected, .. } if expected == "')'"));
                    } else {
                        assert!(tracked.diagnostic.is_none());
                    }
                    continue;
                }
                let report = tracked.diagnostic.unwrap();
                let super::LoadDiagnosticContent::ExpectedObserved { expected, observed } =
                    report.content
                else {
                    panic!("expected a source diagnostic")
                };
                assert_eq!(expected, "a lambda binder (one name: k => body)");
                assert_eq!(
                    format!(
                        "Error:\n- expected : {expected}\n- observed : {observed}\nLocation:\n{}\n",
                        report.excerpt
                    ),
                    std::fs::read_to_string(path.with_extension("stderr")).unwrap(),
                    "{}",
                    case.case
                );
            }
        }
    }

    #[test]
    fn lambda_binder_tracking_preserves_successful_complete_books() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-lambda-binder-diagnostics");
        for case in [
            "parse/uppercase_binder.bend",
            "positive/uppercase.bend",
            "positive/ordinary.bend",
            "positive/reusable.bend",
            "positive/nested.bend",
        ] {
            let path = root.join(case);
            let plain = crate::syntax::load(&path).unwrap();
            let (tracked, _) = crate::syntax::load_with_diagnostics(&path).unwrap();
            assert_eq!(format!("{plain:?}"), format!("{tracked:?}"));
        }
    }

    #[test]
    fn reserved_name_tracking_preserves_successful_complete_books() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-reserved-name-diagnostics/positive");
        for case in [
            "names.bend",
            "qualified.bend",
            "imported.bend",
            "fields.bend",
        ] {
            let path = root.join(case);
            let plain = crate::syntax::load(&path).unwrap();
            let (tracked, _) = crate::syntax::load_with_diagnostics(&path).unwrap();
            assert_eq!(format!("{plain:?}"), format!("{tracked:?}"), "{case}");
        }
    }

    #[test]
    fn unknown_template_metadata_preserves_public_errors_and_imported_sources() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-unknown-template");
        let cases: Vec<String> = facet_json::from_str(include_str!(
            "../../tests/fixtures/cpu-unknown-template/negative-cases.json"
        ))
        .unwrap();
        for case in cases {
            if matches!(
                case.as_str(),
                "negative/missing_close.bend" | "negative/missing_member.bend"
            ) {
                continue;
            }
            let path = root.join(&case);
            let plain = crate::syntax::load(&path).unwrap_err();
            let executable = crate::syntax::load_executable(&path).unwrap_err();
            let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
            let tracked_executable = crate::syntax::load_executable_with_diagnostics(&path)
                .err()
                .unwrap();
            assert_eq!(
                plain.message,
                "~ arguments require a previously declared template head and leading argument position"
            );
            let expected: String = facet_json::from_str(
                &std::fs::read_to_string(path.with_extension("stderr.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(plain.to_string(), executable.to_string(), "{case}");
            for failure in [tracked, tracked_executable] {
                assert_eq!(plain.to_string(), failure.error.to_string(), "{case}");
                assert_eq!(plain.message, failure.error.message);
                assert_eq!(plain.source, failure.error.source);
                assert_eq!(
                    (plain.line, plain.column),
                    (failure.error.line, failure.error.column)
                );
                let report = failure.diagnostic.unwrap();
                let super::LoadDiagnosticContent::ExpectedObserved {
                    expected: want,
                    observed,
                } = report.content
                else {
                    panic!("expected the pinned parser category");
                };
                assert_eq!(
                    format!(
                        "Error:\n- expected : {want}\n- observed : {observed}\nLocation:\n{}\n",
                        report.excerpt
                    ),
                    expected,
                    "{case}"
                );
            }
        }
    }

    #[test]
    fn quantified_binder_metadata_preserves_public_errors_and_imported_sources() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-untyped-binder");
        let cases: Vec<String> = facet_json::from_str(include_str!(
            "../../tests/fixtures/cpu-untyped-binder/negative-cases.json"
        ))
        .unwrap();
        for case in cases {
            if matches!(
                case.as_str(),
                "negative/missing_name.bend"
                    | "negative/missing_type.bend"
                    | "comptime/err_missing.bend"
                    | "comptime/wrong_type.bend"
            ) {
                continue;
            }
            let path = root.join(&case);
            let plain = crate::syntax::load(&path).unwrap_err();
            let executable = crate::syntax::load_executable(&path).unwrap_err();
            let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
            let tracked_executable = crate::syntax::load_executable_with_diagnostics(&path)
                .err()
                .unwrap();
            assert_eq!(plain.message, "expected a type after a quantified binder");
            let expected: String = facet_json::from_str(
                &std::fs::read_to_string(path.with_extension("stderr.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(plain.to_string(), executable.to_string(), "{case}");
            for failure in [tracked, tracked_executable] {
                assert_eq!(plain.to_string(), failure.error.to_string(), "{case}");
                assert_eq!(plain.message, failure.error.message);
                assert_eq!(plain.source, failure.error.source);
                assert_eq!(
                    (plain.line, plain.column),
                    (failure.error.line, failure.error.column)
                );
                let report = failure.diagnostic.unwrap();
                let super::LoadDiagnosticContent::ExpectedObserved {
                    expected: want,
                    observed,
                } = report.content
                else {
                    panic!("expected the pinned parser category");
                };
                assert_eq!(
                    format!(
                        "Error:\n- expected : {want}\n- observed : {observed}\nLocation:\n{}\n",
                        report.excerpt
                    ),
                    expected,
                    "{case}"
                );
            }
        }
    }

    #[test]
    fn duplicate_constructor_metadata_preserves_public_load_errors() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-duplicate-constructor");
        let cases: Vec<String> = facet_json::from_str(include_str!(
            "../../tests/fixtures/cpu-duplicate-constructor/negative-cases.json"
        ))
        .unwrap();
        for case in cases {
            let path = root.join(&case);
            let plain = crate::syntax::load(&path).unwrap_err();
            let executable = crate::syntax::load_executable(&path).unwrap_err();
            let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
            let tracked_executable = crate::syntax::load_executable_with_diagnostics(&path)
                .err()
                .unwrap();
            assert!(plain.message.starts_with("duplicate constructor: "));
            let expected: String = facet_json::from_str(
                &std::fs::read_to_string(path.with_extension("stderr.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(plain.to_string(), executable.to_string(), "{case}");
            for failure in [tracked, tracked_executable] {
                assert_eq!(plain.to_string(), failure.error.to_string(), "{case}");
                assert_eq!(plain.message, failure.error.message);
                assert_eq!(plain.source, failure.error.source);
                assert_eq!(
                    (plain.line, plain.column),
                    (failure.error.line, failure.error.column)
                );
                let report = failure.diagnostic.unwrap();
                let super::LoadDiagnosticContent::ExpectedObserved {
                    expected: want,
                    observed,
                } = report.content
                else {
                    panic!("expected the pinned parser category");
                };
                assert_eq!(
                    format!(
                        "Error:\n- expected : {want}\n- observed : {observed}\nLocation:\n{}\n",
                        report.excerpt
                    ),
                    expected,
                    "{case}"
                );
            }
        }
    }

    #[test]
    fn duplicate_declaration_metadata_preserves_public_load_errors() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-duplicate-declaration");
        let cases: Vec<String> = facet_json::from_str(include_str!(
            "../../tests/fixtures/cpu-duplicate-declaration/negative-cases.json"
        ))
        .unwrap();
        for case in cases {
            let path = root.join(&case);
            let plain = crate::syntax::load(&path).unwrap_err();
            let executable = crate::syntax::load_executable(&path).unwrap_err();
            let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
            let tracked_executable = crate::syntax::load_executable_with_diagnostics(&path)
                .err()
                .unwrap();
            assert!(
                plain.message.starts_with("duplicate declaration: ")
                    || plain.message.starts_with("duplicate definition: ")
            );
            let expected: String = facet_json::from_str(
                &std::fs::read_to_string(path.with_extension("stderr.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(plain.to_string(), executable.to_string(), "{case}");
            for failure in [tracked, tracked_executable] {
                assert_eq!(plain.to_string(), failure.error.to_string(), "{case}");
                assert_eq!(plain.message, failure.error.message);
                assert_eq!(plain.source, failure.error.source);
                assert_eq!(
                    (plain.line, plain.column),
                    (failure.error.line, failure.error.column)
                );
                let report = failure.diagnostic.unwrap();
                let super::LoadDiagnosticContent::ExpectedObserved {
                    expected: want,
                    observed,
                } = report.content
                else {
                    panic!("expected the pinned parser category");
                };
                assert_eq!(
                    format!(
                        "Error:\n- expected : {want}\n- observed : {observed}\nLocation:\n{}\n",
                        report.excerpt
                    ),
                    expected,
                    "{case}"
                );
            }
        }
    }

    #[test]
    fn duplicate_declaration_metadata_does_not_disguise_earlier_errors() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-duplicate-declaration/exploratory");
        for case in ["missing-paren.bend"] {
            let path = root.join(case);
            let plain = crate::syntax::load(&path).unwrap_err();
            let executable = crate::syntax::load_executable(&path).unwrap_err();
            assert_eq!(plain.to_string(), executable.to_string());
            for tracked in [
                crate::syntax::load_with_diagnostics(&path).err().unwrap(),
                crate::syntax::load_executable_with_diagnostics(&path)
                    .err()
                    .unwrap(),
            ] {
                assert_eq!(plain.to_string(), tracked.error.to_string());
                assert!(matches!(tracked.diagnostic.unwrap().content,
                    super::LoadDiagnosticContent::ExpectedObserved { expected, .. }
                    if expected == "'('"));
            }
        }
    }

    #[test]
    fn duplicate_constructor_metadata_requires_an_actual_source_position() {
        let mut origins = super::DiagnosticOrigins::default();
        origins.expected_after_token("missing", 1, 0, "A");
        assert!(origins.load_failure.is_none());
        origins.source("known", "A");
        for (line, column) in [(0, 0), (2, 0), (1, 2)] {
            origins.expected_after_token("known", line, column, "A");
            assert!(origins.load_failure.is_none());
        }
        origins.source("multiline", "A\nB");
        origins.expected_after_token("multiline", 1, 2, "A");
        assert!(origins.load_failure.is_none());
        origins.source("trailing-newline", "A\n");
        origins.expected_after_token("trailing-newline", 1, 2, "A");
        assert!(origins.load_failure.is_none());
        origins.expected_after_token("known", 1, 1, "A");
        assert!(matches!(origins.load_failure.unwrap().content,
            super::LoadDiagnosticContent::ExpectedObserved { observed, .. } if observed == "end of input"));
    }

    #[test]
    fn template_rebind_metadata_preserves_public_load_errors_and_per_use_sources() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-comptime-rebind");
        let cases: Vec<String> = facet_json::from_str(include_str!(
            "../../tests/fixtures/cpu-comptime-rebind/negative-cases.json"
        ))
        .unwrap();
        for case in cases {
            let path = root.join(case);
            let plain = crate::syntax::load(&path).unwrap_err();
            let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
            assert_eq!(plain.to_string(), tracked.error.to_string());
            assert_eq!(
                plain.message,
                "a template argument cannot be rebound as a pattern"
            );
            assert!(matches!(tracked.diagnostic.unwrap().content,
                super::LoadDiagnosticContent::ExpectedObserved { expected, .. }
                if expected == "a pattern (a binder or a constructor)"));
        }
        let value = crate::kernel::term(crate::kernel::Term::Ref("inc".into()));
        let mut origins = super::DiagnosticOrigins::default();
        origins.unsupported_pattern(&value, Some(usize::MAX));
        assert!(origins.load_failure.is_none());
        origins.source("shared", "old\ninc\nnext\nuse\nend");
        origins.node(&value, "shared", 2);
        let occurrence = origins.anchor("shared", 4);
        let owners = std::rc::Rc::strong_count(&value);
        origins.unsupported_pattern(&value, Some(occurrence));
        assert!(
            origins
                .load_failure
                .as_ref()
                .unwrap()
                .excerpt
                .contains("4>| use")
        );
        origins.unsupported_pattern(&value, Some(usize::MAX));
        assert!(
            origins
                .load_failure
                .as_ref()
                .unwrap()
                .excerpt
                .contains("2>| inc")
        );
        assert_eq!(std::rc::Rc::strong_count(&value), owners);
    }

    #[test]
    fn unsupported_pattern_metadata_preserves_public_load_error() {
        for name in ["unsupported_pattern", "invalid_assign_target"] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("tests/fixtures/cpu-match-failures/{name}.bend"));
            let plain = crate::syntax::load(&path).unwrap_err();
            let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
            assert_eq!(plain.to_string(), tracked.error.to_string());
            assert_eq!(plain.message, "a pattern must be a binder or constructor");
            assert!(matches!(tracked.diagnostic.unwrap().content,
                super::LoadDiagnosticContent::ExpectedObserved { expected, .. }
                if expected == "a pattern (a binder or a constructor)"));
        }
    }

    #[test]
    fn row_arity_metadata_preserves_public_load_error_and_location() {
        for name in [
            "arity_mismatch_000",
            "arity_mismatch_001",
            "as_pattern_error",
            "kplus_missing_case",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("tests/fixtures/cpu-matched-values/{name}.bend"));
            let plain = crate::syntax::load(&path).unwrap_err();
            let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
            assert_eq!(plain.to_string(), tracked.error.to_string());
            assert_eq!(plain.message, "a match needs one pattern per scrutinee");
            assert!(
                matches!(tracked.diagnostic.unwrap().content, super::LoadDiagnosticContent::ExpectedObserved { observed, .. } if observed == "'\n'")
            );
        }
    }

    #[test]
    fn scrutinee_metadata_preserves_public_load_errors() {
        for name in [
            "error_window_substituted_scrutinee",
            "consumed_column_error_001",
            "order_ban_002",
            "framed_cell_capture",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("tests/fixtures/cpu-scrutinee-origins/{name}.bend"));
            let plain = crate::syntax::load(&path).unwrap_err();
            let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
            assert_eq!(plain.to_string(), tracked.error.to_string());
            assert_eq!(
                plain.message,
                "match scrutinees must be parameters or fields, in binder order"
            );
            assert!(tracked.diagnostic.is_some());
        }
    }

    #[test]
    fn computed_scrutinee_metadata_preserves_public_load_errors() {
        for name in ["motive_suffix_join", "computed_match_split"] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("tests/fixtures/cpu-computed-scrutinee/{name}.bend"));
            let plain = crate::syntax::load(&path).unwrap_err();
            let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
            assert_eq!(plain.to_string(), tracked.error.to_string());
            assert_eq!(
                plain.message,
                "match scrutinees must be parameters or fields, in binder order"
            );
            assert!(matches!(tracked.diagnostic.unwrap().content,
                super::LoadDiagnosticContent::Message(message)
                if message.contains("a match cannot scrutinize a computed value")));
        }
    }

    #[test]
    fn token_metadata_preserves_public_parse_error() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-rewrite-checking/dead_motive_syntax.bend");
        let plain = crate::syntax::load(&path).unwrap_err();
        let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
        assert_eq!(plain.to_string(), tracked.error.to_string());
        assert_eq!((plain.line, plain.column), (13, 26));
        assert!(matches!(tracked.diagnostic.unwrap().content,
            super::LoadDiagnosticContent::ExpectedObserved { expected, observed }
            if expected == "':'" && observed == "'e'"));
    }

    #[test]
    fn syntax_category_metadata_preserves_public_load_errors() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-token-expectations");
        for (case, expected, observed) in [
            ("controls/term-punctuation.bend", "a term", "'!'"),
            ("controls/name-punctuation.bend", "a name", "'!'"),
            ("controls/term-eof.bend", "a term", "end of input"),
            ("controls/imported/main.bend", "a term", "':'"),
        ] {
            let path = root.join(case);
            let plain = crate::syntax::load(&path).unwrap_err();
            let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
            assert_eq!(plain.to_string(), tracked.error.to_string(), "{case}");
            assert!(plain.message.starts_with("expected a name, found"));
            let report = tracked.diagnostic.unwrap();
            assert!(
                matches!(report.content,
                super::LoadDiagnosticContent::ExpectedObserved { expected: want, observed: found }
                if want == expected && found == observed),
                "{case}"
            );
            if case.contains("imported") {
                assert!(report.excerpt.contains("def bad() -> U32: :"));
                assert!(!report.excerpt.contains("H.bad()"));
            }
        }
    }

    #[test]
    fn declaration_context_preserves_public_errors_and_successful_terms() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-declaration-boundary");
        let cases: Vec<String> = facet_json::from_str(include_str!(
            "../../tests/fixtures/cpu-declaration-boundary/cases.json"
        ))
        .unwrap();
        for case in cases {
            let path = root.join(&case);
            let plain = crate::syntax::load(&path).unwrap_err();
            let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
            assert_eq!(plain.to_string(), tracked.error.to_string(), "{case}");
            assert!(
                plain.message.starts_with("expected a name, found")
                    || plain.message == "expected type, law, or def"
            );
            let diagnostic = tracked.diagnostic.unwrap();
            assert!(matches!(diagnostic.content,
                super::LoadDiagnosticContent::ExpectedObserved { expected, .. }
                if expected == "'def', 'type' or 'law'"));
            if case.contains("imported") {
                assert!(diagnostic.excerpt.contains("On{} : U32"));
                assert!(!diagnostic.excerpt.contains("H.On{}"));
            }
        }
        // Small, fully retained declaration trees include an empty datatype,
        // ordinary rows and a decorator boundary. Do not normalize away changes.
        let path = root.join("controls/unsafe-boundary.bend");
        let plain = crate::syntax::load(&path).unwrap();
        let (tracked, _) = crate::syntax::load_with_diagnostics(&path).unwrap();
        assert_eq!(format!("{plain:?}"), format!("{tracked:?}"));
    }

    #[test]
    fn lambda_arm_context_preserves_public_errors_and_colon_provenance() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-lambda-arm-diagnostics");
        let cases: Vec<String> = facet_json::from_str(include_str!(
            "../../tests/fixtures/cpu-lambda-arm-diagnostics/cases.json"
        ))
        .unwrap();
        for case in cases {
            let path = root.join(&case);
            let plain = crate::syntax::load(&path).unwrap_err();
            let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
            assert_eq!(plain.to_string(), tracked.error.to_string(), "{case}");
            if matches!(
                case.as_str(),
                "controls/missing-close.bend" | "controls/missing-body.bend"
            ) {
                continue; // Separate existing parser errors, checked exactly by the CLI test.
            }
            assert_eq!(plain.message, "a lambda-match arm needs a constructor name");
            let diagnostic = tracked.diagnostic.unwrap();
            assert!(matches!(diagnostic.content,
                super::LoadDiagnosticContent::ExpectedObserved { expected, observed }
                if expected == "'}'" && observed == "':'"));
            if matches!(
                case.as_str(),
                "controls/multiline.bend" | "controls/imported/main.bend"
            ) {
                assert_eq!(plain.line, 5); // The unchanged public error is after the colon.
                assert!(diagnostic.excerpt.contains("4>|     :"));
                assert!(!diagnostic.excerpt.contains("H.bad()"));
            }
        }
        // A shared compile-bound Ref stays a named arm during parsing. Its
        // bare constructor argument is deliberately not a valid checked value.
        let path = root.join("controls/shared-ref-head.bend");
        let plain = crate::syntax::load(&path).unwrap();
        let (tracked, _) = crate::syntax::load_with_diagnostics(&path).unwrap();
        assert_eq!(format!("{plain:?}"), format!("{tracked:?}"));
        assert_eq!(
            crate::kernel::check_book(&plain).err().unwrap().to_string(),
            crate::kernel::check_book(&tracked)
                .err()
                .unwrap()
                .to_string()
        );
    }

    #[test]
    fn erased_binder_context_preserves_public_errors_and_successful_terms() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-erased-binder-diagnostics");
        let cases: Vec<String> = facet_json::from_str(include_str!(
            "../../tests/fixtures/cpu-erased-binder-diagnostics/cases.json"
        ))
        .unwrap();
        for case in cases {
            let path = root.join(&case);
            let plain = crate::syntax::load(&path).unwrap_err();
            let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
            assert_eq!(plain.to_string(), tracked.error.to_string(), "{case}");
            if matches!(
                case.as_str(),
                "controls/argument.bend" | "controls/missing-value.bend"
            ) {
                continue;
            }
            assert_eq!(plain.message, "expected a name, found \"-\"");
            let diagnostic = tracked.diagnostic.unwrap();
            assert!(matches!(diagnostic.content,
                super::LoadDiagnosticContent::ExpectedObserved { expected, .. }
                if expected == "a name"));
            if case == "controls/multiline.bend" {
                assert_eq!(plain.line, 4);
                assert!(diagnostic.excerpt.contains("5>|     5"));
            }
        }
        for case in ["erased", "plus", "subtraction", "template"] {
            let path = root.join("positive").join(format!("{case}.bend"));
            let plain = crate::syntax::load(&path).unwrap();
            let (tracked, _) = crate::syntax::load_with_diagnostics(&path).unwrap();
            assert_eq!(format!("{plain:?}"), format!("{tracked:?}"));
        }
        let path = root.join("controls/deep-body.bend");
        let plain = crate::syntax::load(&path).unwrap_err();
        let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
        assert_eq!(plain.to_string(), tracked.error.to_string());
        assert!(plain.message.contains("expression nesting exceeds"));
        assert!(tracked.diagnostic.is_none());
    }

    #[test]
    fn local_binding_metadata_preserves_the_public_parse_error() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/cpu-destructure/destructure_let_001.bend");
        let plain = crate::syntax::load(&path).unwrap_err();
        let tracked = crate::syntax::load_with_diagnostics(&path).err().unwrap();
        assert_eq!(tracked.error.to_string(), plain.to_string());
        assert_eq!(
            plain.message,
            "a match cannot scrutinize a local binding; pass it to a definition"
        );
        assert!(tracked.diagnostic.unwrap().excerpt.contains("14>|   q = p"));
    }
}

#[cfg(test)]
#[path = "missing_import_tests.rs"]
mod missing_import_tests;

#[cfg(test)]
#[path = "literal_diagnostic_tests.rs"]
mod literal_diagnostic_tests;

#[cfg(test)]
#[path = "empty_character_diagnostic_tests.rs"]
mod empty_character_diagnostic_tests;
