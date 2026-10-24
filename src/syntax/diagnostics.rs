// SPDX-License-Identifier: MPL-2.0
//! Optional loader-owned origins for source reflexivity nodes. No AST or
//! acceptance metadata is changed, and no global pointer registry is used.
use crate::kernel::TermRef;
use std::collections::BTreeMap;
use std::rc::Rc;

#[derive(Default)]
pub(crate) struct DiagnosticOrigins {
    sources: BTreeMap<String, Rc<str>>,
    proofs: BTreeMap<usize, ProofOrigin>,
}

struct ProofOrigin {
    // Retain the allocation: an identity cannot be recycled while this source
    // map is paired with the check result. Identity is never dereferenced.
    _proof: TermRef,
    source: Rc<str>,
    line: usize,
}

impl DiagnosticOrigins {
    pub(super) fn source(&mut self, label: &str, source: &str) {
        self.sources
            .entry(label.to_owned())
            .or_insert_with(|| Rc::from(source));
    }

    pub(super) fn proof(&mut self, proof: &TermRef, label: &str, line: usize) {
        if let Some(source) = self.sources.get(label) {
            self.proofs.insert(
                Rc::as_ptr(proof) as usize,
                ProofOrigin {
                    _proof: Rc::clone(proof),
                    source: Rc::clone(source),
                    line,
                },
            );
        }
    }

    pub(crate) fn excerpt(&self, identity: usize) -> Option<String> {
        let origin = self.proofs.get(&identity)?;
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
