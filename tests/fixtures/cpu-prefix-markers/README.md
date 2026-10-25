# Repeated prefix markers

Project-authored MPL-2.0 cases exercise the character-at-a-time prefix grammar in
[pinned Bend](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1788)
and its [adjacent Nat tail](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2239).
`1n++++k` means one successor tail followed by three reusable-prefix markers;
it is not four additions or ordinary numeric unary plus. Declaration quantities
and infix string append use separate grammar positions.

Positive `.json` files capture complete canonical pinned interpreter value graphs;
`.stdout` files record its text. The integration test compares every graph field.
The existing native String constructor display remains explicit in `.native-stdout`.
`io.bend` has executable output `4`, while strict proof checking excludes IO and
the direct upstream Node interpreter lacks its host `require` binding. Generated
C and JS are measured separately. Negative expected stderr is stored as JSON
strings; only the test fixture path is made repository-relative for portability.
Differing upstream and port diagnostics remain differences, not exact matches.

No files are copied from upstream. The original predecessor fixture and receipts
remain in `cpu-reusable-nat`; complete historical command logs and binary captures
are outside Git. See `docs/cpu-prefix-markers.md` for the final qualification.
