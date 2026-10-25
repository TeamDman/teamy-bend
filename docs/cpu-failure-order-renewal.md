# Competing-error contract: renewed, still unresolved

Clean candidate: `d8122bf54fb0879a3307ee1873f5ec607a824e40`. No runtime reordering or new
oracle exception was implemented. Independent review found no demonstrated port
traversal defect. The five-control experiment and ten-control demand suite were
renewed at this source; the independent parser diagnostic correction is described
[separately](cpu-token-expectations.md).

## The approval already in force

The prior approved choices are exactly the three in the
[approved-demand checkpoint](cpu-approved-demand.md):

1. Evaluate ordinary live executable arguments even when the function ignores them.
2. Validate the runtime Nat bound before shallow observation.
3. Continue refusing private effect requests, including wildcard matches, with
   a narrow recorded exception for the conflicting upstream C wildcard behavior.

They preserve erasure, unselected branches, IO ordering and safety limits. They do
not approve copying C's source-shape-dependent competing-error order, treating all
error order as unspecified, or accepting any further conflicting-oracle exception.

The **two primary disagreements are one `nat_first` fixture repeated twice**.
The **six approved wildcard observations are three different fixtures repeated
twice** (`unused_rhs_error`, `ordinary_rhs_error`, `forced_request_wildcard`). `nat_first`
uses explicit constructors to reject a private request; it is not a wildcard
case. Their statuses remain separate in [the renewed demand receipt](cpu-demand-renewal-results.json).

## Exact result and cause

In `pair(overflow(), invalid())`, `pair` ignores both arguments, but both remain
live. `overflow()` exceeds the maximum Nat; `invalid()` tries to inspect a private
IO request. All lanes exit 1, stdout is exactly `before` without a newline, and
neither the request payload nor `kept`/`after` appears.

| Expression / control | Native, port C/JS, upstream JS | Upstream C O3 |
| --- | --- | --- |
| `pair(overflow(), invalid())` | `bend: a Nat past the largest immediate 2^48-1\n` | `bend: runtime fail-stop\n` — unresolved |
| `pair(invalid(), overflow())` | Request diagnostic | Request diagnostic |
| First argument through `apply(u => overflow())` | Nat diagnostic | Nat diagnostic |
| First complete `IO.pure(Nat, overflow())` | Nat diagnostic | Nat diagnostic |

The pinned [C ANF pass](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1281)
leaves [flat calls](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1462)
inline while binding non-flat calls into earlier continuations. Generated C jumps
to the invalid request before the later inline Nat check. [CPU err_post](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L3604)
prints and immediately exits. This is a Bend backend-lowering difference, not an
observed Clang optimization-level difference or demonstrated C operand-order issue.
The same result occurs at O0/O1/O2/O3 with the installed compiler. Different
compiler versions and multiworker/GPU profiles remain untested.

The [Lean core strategy](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.lean#L9347)
is explicitly left-to-right, but its [scope](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.lean#L98)
and the [runtime account's limitations](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/docs/BendRT/main.typ#L602)
do not establish a cross-backend rule selecting between these two fail-stop
messages. They also do not establish that arbitrary failure order is permitted.
The Lean source was inspected, not built or newly proved.

## Decision needed from Teamy

Recommended narrow decision, **pending approval**:

> Keep the port's existing left-argument-first failure for ordinary sequential
> live arguments, and accept the demonstrated pinned-C `nat_first` lowering
> discrepancy as a separate, explicitly bounded oracle exception. Both failures
> remain fatal; IO sequencing, erasure, unselected branches and limits stay unchanged.

This would preserve current native/port-C/port-JS behavior and upstream-JS agreement.
Its compatibility cost is giving up exact diagnostic bytes for this particular
pinned-C case. It would not authorize arbitrary C mismatches, reordering effects,
weakening private-request refusal, or extending the old wildcard exception.
Without approval, retain these two primary cells as unresolved until the contract
or upstream implementation is settled. Imitating C's hoisting in the port would
instead break the current corresponding JS agreement and lacks an approved basis.
No decision has been inferred from elapsed time or earlier demand approval.

## Qualification and reproduction

The fresh four-profile research receipt contains 123 commands: 20 successful
generations, 24 successful C builds, one failed C build and 78 executions. Seventy
executions match their exact port contract; eight reproduce the order discrepancy:
**two primary O3 observations plus six supplemental O0/O1/O2 observations**.
These six supplemental observations are unrelated to the six wildcard observations.
All available repeated outcomes are deterministic; zero timeouts.

`word_live_tail` still fails Clang register allocation at O0: its two executions
are unrun. O1/O2/O3 execute and match. O3 is the actual pinned Linux CLI profile;
O0 remains an oracle build limitation. Generated C and JS hashes for all five
controls are identical before and after this parser-only change.
[Exact current outcomes, commands and hashes](cpu-failure-order-renewal-results.json).

The ten-control primary-O3 demand suite records **92 exact executions, six approved
wildcard observations and two unresolved order observations**; its exit 1 remains
a required-parity failure. Its failure-order, live/erased values, Word-tail and
unselected-branch controls are retained. The full regression gate separately
covers laziness, cancellation, work/depth limits and sharing.

With the frozen environment and new output directories:

```sh
python3 scripts/compare-failure-order.py --reference "$REFERENCE" --output "$EVIDENCE/order"
pwsh -NoProfile -File scripts/compare-cpu-negatives.ps1 \
  -Reference "$REFERENCE" -OutputDirectory "$EVIDENCE/demand" -OracleOptimization O3 -Suite demand
```

The first runner exits 0 when research finishes; it does not certify parity. Both
use 60-second per-child limits. The fresh reference remains clean at
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. Historical receipts in
[cpu-failure-order.md](cpu-failure-order.md) remain tied to their original source.
No dependencies, toolchains, reference, guard, budget, cancellation policy, main
branch or release changed. Compact-pattern research remains unadopted.
