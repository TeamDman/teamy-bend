# Compile-time occurrence provenance controls

These new MPL-2.0 controls exercise pinned Bend
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599` behavior. `negative-cases.json`
contains 14 adjacent checker refusals with exact upstream stderr captures.
The integration test also reuses the unchanged original `comptime/later_def`
and its four adjacent cases from `../cpu-definition-provenance/pending-cases.json`,
plus that directory's qualified-name control: 20 exact cases in total.
The historical manifest name is retained to preserve previous receipts.

`guard-cases.json` contains two macro-rebinding load refusals. Both reject,
but port/upstream diagnostic spelling and location formatting still differ;
they are phase controls, not exact diagnostic matches. The upstream captures
are retained without changing either original parser policy or refusal.

The positive programs cover nested forwarding, lambda macros, template-cache
reuse and repeated shared reference values. Their `.stdout` and complete `.json`
graphs come from the pinned interpreter. Pure C/JS APIs and native strict
evaluation are tested separately from executable CLI emission (which requires
Base). No unexecuted executable artifact receives runtime credit.

Pinned `bend2/bend.ts` lines1635–1644 convert a free simple-name argument into a
reference carrying its first template-use span; forwarded and qualified references
retain the resulting span. Lines1799–1801 preserve unchanged trailing arguments
when `+` changes datatype quantities. The tests require these source rules without
changing core identity, sharing, macro recognition, cache keys or checker work.

Run `cargo test --locked --test cpu_template_occurrences` and
`cargo test --locked --lib occurrence`. Source guards, cancellation, resource
limits and aggregate sharing controls also remain in the standard repository gate.
