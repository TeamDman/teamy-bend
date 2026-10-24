# Hypothesis specialization, Data reuse and residual fallback

Unchanged Apache-2.0 originals from Bend e6676b080f25b1bc1bf5b5b7d7a17e22f8022599:
`tests/check/computed_match_specialize.bend`, `cop_bare_trio_000.bend` and
`fits_residual_fallback.bend`. Derived oracle graphs/text/diagnostics retain
upstream attribution. New tests and adapters use MPL-2.0.

Eleven complete definition graphs preserve hypotheses, named calls, reusable
binders and residual fallback arms. Thirty-four bounded applications have typed
callers: six valid proof calls, four doubling controls, four addself calls,
sixteen addition pairs and four fallback/matcher calls. Nat depths are 0/1/2/4.
Two matcher calls apply the function returned by a zero-parameter definition;
they use typed caller evaluation, separate from the 32 declared-entry calls.
Positive worker scrutinees retain a valid zero hypothesis, without fabricated
proofs. Four negative sources reject impossible hypotheses, a false addself
result and a missing residual constructor case.

Complete constructor lists, scalar projections, six closed proofs and original
main adapters have separate structural and runtime observations. Original builds
need Base. Existing oracle failures remain unresolved; successful projections
cannot qualify complete constructor printers or blocked C execution.
