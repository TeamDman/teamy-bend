// SPDX-License-Identifier: MPL-2.0
//! Optional loader-owned origins for source diagnostic nodes. No AST or
//! acceptance metadata is changed, and no global pointer registry is used.
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::kernel::origins::TermOrigins;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::rc::Weak;

#[derive(Default)]
pub(crate) struct DiagnosticOrigins {
    pub(crate) tracker: TermOrigins,
    load_failure: Option<LoadDiagnostic>,
    // Loader-wide fresh binder IDs survive Body cloning/substitution unchanged.
    // This index owns source text only; no core term or strong term edge is added.
    local_binders: BTreeMap<usize, (Rc<str>, usize)>,
    sources: BTreeMap<String, Rc<str>>,
    nodes: BTreeMap<usize, TermOrigin>,
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

struct TermOrigin {
    // Retain the allocation: an identity cannot be recycled while this source
    // map is paired with the check result. Identity is never dereferenced.
    _node: Weak<Term>,
    source: Rc<str>,
    line: usize,
}

impl DiagnosticOrigins {
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

    pub(super) fn local_binder(&mut self, id: usize, label: &str, line: usize) {
        if let Some(source) = self.sources.get(label) {
            self.local_binders.insert(id, (Rc::clone(source), line));
        }
    }

    pub(super) fn rejected_local_binding(&mut self, id: usize) {
        self.load_failure = self.local_binders.get(&id).and_then(|(source, line)| {
            source_excerpt(source, *line).map(|excerpt| LoadDiagnostic {
                content: LoadDiagnosticContent::Message("a parameter or field scrutinee (a match cannot scrutinize a local binder: give it its own def)".into()),
                excerpt,
            })
        });
    }

    pub(super) fn expected_token(&mut self, label: &str, line: usize, expected: &str, found: &str) {
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
                    expected: format!("'{expected}'"),
                    observed: found
                        .chars()
                        .next()
                        .map_or_else(|| "end of input".into(), |first| format!("'{first}'")),
                },
                excerpt,
            })
        });
    }

    pub(super) fn load_failure(&mut self, error: super::ParseError) -> DiagnosticLoadError {
        DiagnosticLoadError {
            error,
            diagnostic: self.load_failure.take().map(Box::new),
        }
    }

    pub(crate) fn excerpt(&self, identity: usize) -> Option<String> {
        let origin = self.nodes.get(&identity)?;
        source_excerpt(&origin.source, origin.line)
    }
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
