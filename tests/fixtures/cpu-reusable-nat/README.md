<!-- SPDX-License-Identifier: MPL-2.0 -->
# Reusable Nat successor patterns

`original/cop_let_descent_escape.bend` is byte-identical to
[Bend's pinned original](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/reg/cop_let_descent_escape.bend).
It remains under the upstream Apache-2.0 attribution in NOTICE. The other Bend
programs are project-authored MPL-2.0 probes. `.json` positive files capture full
constructor graphs from the pinned interpreter; `.stdout` records its text.

The original performs IO, so strict proof checking and executable checking have
different applicability. Its direct upstream Node execution has a host failure;
that does not establish a language failure or qualify an unrun runtime lane.

Negative `.port-stderr.json` and `.upstream-stderr.json` files retain exact
observations separately. Spaced addition and F32 representations still differ;
matching rejection status is not exact diagnostic parity. The pure String probe
has matching complete constructor data but a native text-printer difference,
recorded in `.native-stdout`. No new parity exception is introduced.

`before.json` preserves the compact pre-fix observations at their actual source
revision. Private command roots are represented by logical placeholders.
