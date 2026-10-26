# Reviewed CPU backlog consolidated onto main

Remote main was fast-forwarded, without force, from
`969561f5214fa494507ba07bd5740eea18d6620e` to
`c99e74a1556539c187bef7803a252919e19d835e` on 2026-10-26.
GitHub branch reads and an independent Git remote read verified the exact result.
All 68 other remote heads were unchanged; no branch was deleted or force-updated.

The pre-update audit found 69 remote heads: 67 ancestors of the target, one
equal target, and one divergent archive. Main had no unique commits and was
347 commits behind. These branches mostly recorded successive cumulative
checkpoints; merging each one would add no missing work.

The only divergent branch,
`archive/cpu-runtime-negatives-pre-correction-20261023`, retains commit
`3089177b24c4dce48ace830039dcd2214f12461f`. Its one unique commit is superseded
by `3b9739b3746bed0d505e62f7c31a6d1c1c6296ed`: the corrected runner retains mixed
repetition outcomes instead of letting a later repetition overwrite a failure.
The five other documentation/evidence files are identical. The archive remains
remotely preserved and was not merged.

Independent review covered ancestry, cumulative diff inventory, risk-relevant
source/contracts, content hygiene and exact test identity. It was not an
exhaustive line-by-line re-review of all 347 historical commits. The changed
4,417 blobs, 14 compressed receipts and 7,458 decoded fields had no actionable
credential, private-path or oversized-binary finding. Cargo.toml and Cargo.lock
are unchanged from the old main. No CI workflow or dependency was introduced.

The integrated tree has the same code/build/test inputs as tested source
`b46f9570a2174b6f475b4f58836d960534a967a8`; only six documentation files differ.
Its fresh standard gate passed **1,322 tests, zero failures, 49 ignores**, across
195 result groups in 941.061 seconds. Post-gate executable identity matched the
differential snapshot. This reviewed checkpoint, with identical code/build/test
inputs, was integrated; there was no new
merge result needing a separate qualification run.

Before updating, GitHub reported main unprotected, no required
status checks, no rulesets and no open pull requests. The update supplied both
the expected old SHA and `force=false`. No protection was bypassed. No release
or package publication occurred.

At audit time, the local checkout was clean on the separate malformed-diagnostic
branch at `d79486efef4cb6baf6d94515c9ee764b2fedeb77`, with no stashes. Of 150
local heads, 147 were target ancestors, one equaled the target, one was this new
descendant and one was the same archived variant. That current diagnostic work
was excluded from the main update and retains its independent validation.

The repository remains prominently incomplete WIP. Full CPU parity, GPU hardware
qualification, external API consumer audits, supplementary all-target lint
failures and the recorded oracle/resource/semantic disagreements are not waived
by main consolidation. The experimental GPU candidate already in cumulative
history remains unqualified for hardware. Compact Nat/string lowering and the
pending competing-error-order decision remain unchanged.

Public destination: https://github.com/TeamDman/teamy-bend/tree/main
Exact integrated commit: https://github.com/TeamDman/teamy-bend/commit/c99e74a1556539c187bef7803a252919e19d835e
Preserved feature checkpoint: https://github.com/TeamDman/teamy-bend/tree/fix/cpu-lowercase-escape-grammar-20261026

The complete branch census, API metadata, operation receipt and both independent
reviews are retained with this local audit. The report describes the verified
update, not a claim that every branch represents finished parity.
