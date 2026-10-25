# Invalid-token source diagnostics

Five `original/` files are unchanged Apache-2.0 fixtures from pinned upstream
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; see NOTICE. Their actual failure is
source loading, regardless of older comments about array/checking semantics.
New MPL-2.0 controls cover term/name positions, EOF without a final newline and
an imported error site. Expected stderr JSON is captured from upstream twice.
The pre-fix binary and compact failure evidence are identified in `before.json`.

The correction annotates existing failures at invalid identifier starts or EOF.
It keeps public ParseError values, tokens, acceptance and guards unchanged; the
CLI now renders the pinned expected category, first observed character and source
excerpt. Keyword-specific errors retain their separate current behavior. It does
not implement the obsolete array spellings used by some rejected originals.

The integration regression in `tests/cpu_pattern_locations.rs` checks exact stderr,
empty stdout and exit 1 for check/eval/run and executable C/JS/native compilation;
no output artifact may appear. No program execution pass is claimed for these
load-rejected inputs. Existing CLI API callers must inspect `CliOutput::exit_code()`
for formatted refusals; external consumers have not been audited.
