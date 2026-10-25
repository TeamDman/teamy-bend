<!-- SPDX-License-Identifier: MPL-2.0 -->
# Reusable-let diagnostic locations

These small project-authored fixtures exercise the pinned Bend check-let rule:
only a failure whose expected object is the exact let-kind expectation receives
the let's source span. Inference and unrelated nested errors keep their own
locations. Upstream: `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`,
`bend2/bend.ts`, check-let (lines 3452–3461).

`negative-cases.json` names six exact rejection controls. The imported helper is
included unchanged from `reusable-field.bend`. The unsourced-let expected report
is a programmatic Rust regression: fresh unsourced lambda/let nodes retain the
original children and therefore fall back to the original parameter type span.
It is not a separate upstream source fixture.

`before-receipt.json` preserves the compact original mismatch on source
`3b144f16f1a6dd5344a668dc0bd31a488c602aeb` (checkpoint `6d6a03ea88188523361858abdb5d383e6bd723fd`).
Its old stderr remains evidence, not a current expected result. Command roots are
placeholders; diagnostic bytes are unchanged.

`exploratory/observations.json` retains four adjacent probes without qualification:
`deeper-kind` lacks port source provenance; `inner-inference` exposes an annotation
parse difference; `parallel-let` uses invalid comma syntax (both reject, different
load diagnostics); `reusable-lambda` exposes differing rejection phases for the
probed syntax. Their original source and streams remain available. They are not
added to the exact-case manifest or counted as upstream-inventory coverage.
