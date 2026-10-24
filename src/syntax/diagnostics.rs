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
    pattern_failure: Option<PatternDiagnostic>,
    sources: BTreeMap<String, Rc<str>>,
    nodes: BTreeMap<usize, TermOrigin>,
}

#[derive(Debug)]
pub(crate) struct DiagnosticLoadError {
    pub(crate) error: super::ParseError,
    pub(crate) pattern: Option<PatternDiagnostic>,
}

#[derive(Debug)]
pub(crate) struct PatternDiagnostic {
    pub(crate) constructor: String,
    pub(crate) excerpt: String,
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

    pub(super) fn unknown_pattern(&mut self, name: &str, value: &TermRef) {
        // This parser rejection is not recovered: the private loader carries
        // the structured cause alongside its original public ParseError.
        self.pattern_failure =
            self.excerpt(self.tracker.source(value))
                .map(|excerpt| PatternDiagnostic {
                    constructor: name.into(),
                    excerpt,
                });
    }

    pub(super) fn load_failure(&mut self, error: super::ParseError) -> DiagnosticLoadError {
        DiagnosticLoadError {
            error,
            pattern: self.pattern_failure.take(),
        }
    }

    pub(crate) fn excerpt(&self, identity: usize) -> Option<String> {
        let origin = self.nodes.get(&identity)?;
        let lines = origin.source.split('\n').collect::<Vec<_>>();
        let at = origin.line;
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
}
