# Contextual empty-character diagnostics

Authored MPL-2.0 controls against pinned Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. Existing original-empty and
raw-quotes-2 files, and the five positive controls, are referenced unchanged.

Pinned Char parsing consumes the second quote in `''` as apostrophe code 39,
then requires an immediate closing quote. Optional diagnostic attribution now
uses the original source immediately after the two quotes. Another apostrophe
or an astral observed scalar withholds this metadata: a longer quote run has a
different parse history, and pinned diagnostics observe one UTF-16 code unit.
The existing cardinality rejection, raw ParseError and parser cursor survive.

The manifest records 22 refused inputs and five accepted executable controls.
At its original checkpoint, eighteen rejection diagnostics matched upstream; four remained different: earlier
grammar, empty-followed-escape, four-quotes and raw-before-grammar. The baseline
had two exact diagnostics, so this closes 16 differences without an acceptance
change. Subsequent quote/grammar attribution and bounded isolated recovery now
match all 22 diagnostics. The separate `cpu-four-quote-recovery` cohort has five
exact diagnostics and three retained differences; it does not change this
cohort's source bytes. Strict IO proof exclusion remains distinct from executable acceptance.

The CRLF fixture and expected stream are marked `-text` locally so Git preserves
those exact bytes on checkout. Trailing trivia in the negative fixtures and
expected diagnostics is intentional comparison data.
