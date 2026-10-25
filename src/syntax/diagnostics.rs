// SPDX-License-Identifier: MPL-2.0
//! Optional loader-owned origins for source diagnostic nodes. No core AST or
//! acceptance metadata is changed, and no global pointer registry is used.
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::kernel::origins::MatchSite;
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

    pub(super) fn unsupported_pattern(&mut self, value: &TermRef) {
        self.load_failure =
            self.excerpt(self.tracker.source(value))
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
