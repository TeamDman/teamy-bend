// SPDX-License-Identifier: MPL-2.0
//! Bounded, optional grammar context for an already-rejected lexical prefix.
//! The original public error always wins; no partially parsed book can escape.
use super::ParseError;
use super::Parser;
use super::Token;

// These cap extra presentation work, not accepted language input or any existing
// parser/evaluator budget. Include trivia bytes and the terminal fault token.
const MAX_PREFIX_TOKENS: usize = 64;
const MAX_PREFIX_BYTES: usize = 4096;

#[derive(Clone, Copy)]
pub(super) enum LexicalFault {
    DecimalSuffix { after: usize },
    Backtick,
}

enum PrefixOutcome {
    Active,
    Unsupported,
}

pub(super) struct LexicalDiagnostic {
    pub(super) initial_declarations: usize,
    start: usize,
    fault: LexicalFault,
    outcome: PrefixOutcome,
}

impl LexicalDiagnostic {
    pub(super) fn unsupported(&self) -> bool {
        matches!(self.outcome, PrefixOutcome::Unsupported)
    }
}

pub(super) struct LexicalPrefix {
    pub(super) tokens: Vec<Token>,
    pub(super) diagnostic: LexicalDiagnostic,
}

impl LexicalPrefix {
    pub(super) fn eligible(tokens: usize, end: usize) -> bool {
        tokens < MAX_PREFIX_TOKENS && end <= MAX_PREFIX_BYTES
    }

    pub(super) fn capture(
        target: &mut Option<&mut Option<Self>>,
        tokens: &[Token],
        source: &str,
        mut fault_token: Token,
        fault: LexicalFault,
    ) {
        let Some(target) = target else { return };
        if !Self::eligible(tokens.len(), fault_token.end) {
            return;
        }
        debug_assert!(source.is_char_boundary(fault_token.end));
        fault_token.text = source[fault_token.start..fault_token.end].into();
        let diagnostic = LexicalDiagnostic {
            initial_declarations: 0,
            start: fault_token.start,
            fault,
            outcome: PrefixOutcome::Active,
        };
        let eof = Token {
            text: String::new(),
            line: fault_token.line,
            column: fault_token.column + fault_token.text.chars().count(),
            start: fault_token.end,
            end: fault_token.end,
        };
        let mut tokens = tokens.to_vec();
        tokens.extend([fault_token, eof]);
        **target = Some(Self { tokens, diagnostic });
    }
}

impl Parser<'_> {
    // Stop before expansion, nontrivial lowering or host-path inspection. The
    // failed Loader owns any temporary prefix declarations and binder IDs.
    pub(super) fn require_semantic_prefix(&mut self) -> Result<(), ParseError> {
        if let Some(diagnostic) = &mut self.templates.lexical_diagnostic {
            diagnostic.outcome = PrefixOutcome::Unsupported;
            return Err(self.error("unsupported lexical diagnostic prefix"));
        }
        Ok(())
    }

    pub(super) fn require_valid_numeric_token(&mut self) -> Result<(), ParseError> {
        let Some(diagnostic) = &self.templates.lexical_diagnostic else {
            return Ok(());
        };
        if diagnostic.start != self.current().start {
            return Ok(());
        }
        let LexicalFault::DecimalSuffix { after } = diagnostic.fault else {
            return Ok(());
        };
        let line = self.current().line;
        if let Some(origins) = &mut self.templates.diagnostics {
            origins.expected_after_token(
                &self.label,
                line,
                after,
                "a numeric literal (NUMBER is U32, NUMBER n is Nat)",
            );
        }
        Err(self.error("invalid numeric literal suffix"))
    }
}

#[cfg(test)]
mod tests;
