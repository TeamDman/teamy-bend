# Contextual diagnostics and blocked normalization

Tested source: `0dec12b6b2261ea82a27da454dda3272b8128a12`. The upstream target remains
[e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
CPU parity remains incomplete. This checkpoint fixes diagnostic presentation and
investigates an independent normalization mismatch; it does not modify reduction.

## Correct contextual names

`page/context_dedup_shadow.bend` rejects a false reflexivity proof under two
binders named `x`. The outer binder is the endpoint being compared. The port
previously printed `x` and `NS{x}`; pinned output is `x^0` and `NS{x^0}`.
Both context rows and the Bend source location were already correct.

The checker now supplies its existing binder-admission order to the borrowed
term formatter. Expected and observed terms use the full context. A context-row
type uses only the prefix before its binder; later shadowing cannot change the
meaning of an earlier row. This follows pinned
[ctx_scope](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L931),
[err_show](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1450)
and [term_show](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1297).
Semantic IDs locate variables; the printed suffix is the lexical level, not that ID.

Only [checker presentation](../src/kernel/check.rs) and the visibility of the
existing private [scope formatter](../src/kernel/term.rs) change. The checker still
rejects before constructing diagnostic metadata, with presentation using only the
remaining allowance on its existing clone. Tests cover nonsequential IDs, repeated
names, shadowed globals, typeless diagnostics, prefix-scoped types, source identity,
unchanged Rc ownership, unchanged checker state and exhausted presentation fuel.
No evaluator, acceptance, sharing, source-hole/structural guard or budget is changed.

The [two original fixtures](../tests/fixtures/cpu-context-scope/README.md) are
unchanged Apache-2.0 upstream bytes. Three new MPL-2.0 cases cover single/triple
shadowing, an earlier proof hypothesis and imported source provenance. The
adjacent `page/arm_leaf_context.bend` retains its already-correct diagnostic.
All five load before rejecting during checking. They have no runtime pass.

The [CLI regressions](../tests/cli_checker_diagnostics.rs) assert exit 1, empty
stdout and exact stderr across check/eval/run in text, JSON and CSV modes, plus
C/JS/native emission refusal with no output artifact. The repeated differential
matrix additionally verifies pinned upstream C/JS emission failures and actual
checking phase. Four earlier false-proof/constructor/rewrite/matched-value controls
remain exact. There is no new public API: the formatter helper is visible only
inside the kernel. Diagnostic text deliberately changes; earlier external-consumer
API caveats remain open rather than being re-audited by this change.

## Partial application: an actual normalized-graph difference

The original
[partial_app_snf](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/eval/partial_app_snf.bend)
still succeeds in both interpreters, but normal forms disagree. Pinned output
retains calls to `snf9` and `wnf`; the port exposes a larger nested match tree.
Direct graph inspection confirms this is not a formatter abbreviation or an
alpha-renaming issue. Bound-variable distances, references, applications,
constructors and match arms are compared without evaluation or text replacement.

| Probe | Pinned graph nodes | Port graph nodes | Exact structure |
| --- | ---: | ---: | --- |
| Original `partial_app_snf` | 27 | 166 | Different |
| Nested constructor field | 5 | 9 | Different |
| Fuel plus nested constructor field | 9 | 16 | Different |
| Blocked first argument | 6 | 6 | Equal |
| Function-returning control | 6 | 6 | Equal |

These are tree-node occurrences in the bounded comparison vocabulary, not heap
allocations or a performance benchmark. Every graph observation repeats twice.
The primary Rust probe links the gate-built library with the same features and
profile as the captured CLI (`default,extended_observability`). A separately
recorded probe of the all-feature library, including `tracy`, gives identical
graphs. Their artifact identities remain distinct.

A valid minimized case is:

```bend
type N is Data:
  Z{}
  S{p: N}
type K is Type:
  Box{n: N}
def f(k: K) -> N:
  match k:
    case Box{n}:
      match n:
        case Z{}: Z{}
        case S{p}: p
def main() -> N -> N:
  x => f(Box{x})
```

Pinned output is `x => f(Box{x})`. Port output is
`x => \{Z: Z{}; S: (p => p)}(x)`. This difference precedes formatting.
The pinned [weak-head contract](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2807)
specifies retaining a named application when it gets stuck. Its
[match machine](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L2975)
carries and reconstructs that call through constructor-field matching. The port's
[consume path](../src/kernel/reduce.rs) returns the exposed field match after the
outer constructor matches; the named-call information is no longer carried.
The normalizer subsequently visits that residual tree. Its printer faithfully
shows that different structure.

The bounded closed-value observations do agree. Applying each of four minimized
functions to Nat depths 0, 1 and 2 gives `[0, 0, 1]` in both lanes, except the
fuel-limited function, which gives `[0, 0, 0]`. Applying the original's two rebuilt
lambdas to `TNum{7}` and `TNum{9}` and observing its resulting application gives
`[1, 7, 9]` (tag, function value, argument value). These observations match direct
upstream interpretation, port pure CLI and generated upstream/port C and JS.
They neither prove general extensional equivalence nor qualify the differing
open normal form. Native executable/IO behavior is not separately tested by these
pure observations. No partial-normalization fixture is added to a passing ledger.

The next bounded repair should carry named-call reconstruction through a blocked
constructor-field match, following the pinned mechanism. It needs direct/nested/
fuel/partial-application regressions, original `partial_app_snf`, and renewed
resource, cancellation, false-proof and demand controls. Do not replace the
printed string, force a match on a free variable, or increase a budget. The pinned
contract supplies the intended behavior; there is no new policy exception or
user decision inferred from these observations. No evaluator change was made here.

## Validation and reproduction

- Full gate: **959 passed, 0 failed, 49 ignored**, 115 result-bearing targets.
  Formatting, strict Clippy, all-feature build and real socket-provider checks pass.
- Focused lib/checker-CLI/scope/normalization run: **288 passed, 0 failed, 1 ignored**.
- Diagnostic matrix: **180 observations**, comprising 162 exact rejection contracts
  and 18 checking-phase probes. All result lanes repeat twice; no timeout or
  nondeterminism. [Compact CSV](cpu-context-scope-evidence.csv).
- Checking audit: **1,302 unchanged statuses**. All 439 expected negatives reject;
  other fixtures are 366 accepted / 497 rejected. Only `context_dedup_shadow` stderr
  changes, to the independently verified pinned result; all other streams match
  the previous port receipt. This is not universal upstream diagnostic parity.
- Fourteen positive controls pass. Eighteen observations retain three unwaived
  hole/TODO discrepancies. The 49 ignores remain unrun: 47 GPU hardware tests and
  two optional profilers.
- Primary graph probe: 20 observations; the separate all-feature graph probe also
  has 20. The original and two nested-field cases mismatch; two controls match.
- Closed-value matrix: **90 observations**: 60 exact outputs and 30 generation/
  build successes. Bounds are 30 seconds per child, 60 per checking-audit child
  and private probe build, and 1,200 per full gate.

```sh
source "$TOOLS_ROOT/activate.sh"
export CARGO_NET_OFFLINE=true BEND_GPU=off CARGO_BUILD_JOBS=4
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
cargo test --locked --features extended_observability --lib --test cli_checker_diagnostics --test kernel_scope_printer --test kernel_normalization --quiet
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
node "$BEND_REFERENCE/bend2/main.ts" "$ORIGINAL"
"$BEND_BINARY" --output-format json check "$ORIGINAL"
"$BEND_BINARY" compile "$ORIGINAL" --executable --target c --output "$WORK/refused.c"
node "$BEND_REFERENCE/bend2/main.ts" "$WRAPPER" -o "$WORK/up.c"
clang-19 -std=c11 -O3 "$WORK/up.c" -lpthread -lm -o "$WORK/up"
"$WORK/up" --threads 1 --gpu off
"$BEND_BINARY" compile "$WRAPPER" --executable --target c --output "$WORK/port.c"
clang-19 -std=c11 -O3 -Wall -Wextra -Werror -pedantic "$WORK/port.c" -pthread -lm -ldl -o "$WORK/port"
"$WORK/port" --threads 1 --gpu off
```

Generated JS uses Node 24.19; the C observations use the pinned Linux O3 profile.
Host Rust debug overrides do not change Bend evaluation policy or budgets. The
private graph probe is compiled with installed Rust against an existing gate-built
library; no dependency is installed or adopted. Exact commands, complete streams,
phase results, source hashes and artifacts are retained in private receipts.

The initial focused build at `c3098c4ab0f2046fad48ba2a83e5a2286754655c` failed with
three Rust borrowing/coercion errors. Explicit `as_ref()` borrowing corrected them;
the successful receipts are tied to the tested source above. Initial authored
wrappers also failed checking because an unsuffixed `+` selected Nat addition;
they were corrected to explicit `U32.add`. A first generated-JS observation failed
because hyphenated helper import paths became invalid upstream identifiers.
The failed sources/artifacts remain preserved. A separate repeat used byte-identical
authored helpers under underscore-only module paths. No upstream source was edited,
and the failed attempt is not counted among the 90 successful observations.

The full gate took 313.38 seconds, starting at 11.012 GiB free and reaching
10.310 GiB. The 11 GiB build preflight and 10 GiB reserve remain unchanged.
After capture, 111 inactive rebuildable integration-test executables were reclaimed;
all source and historical evidence remain. Current free space is 11.014 GiB.
Preflight again before future builds. Emergency floors remain 2 GiB workspace and
1 GiB temporary storage.

Detailed receipts link **325 originals**; **977** lack detailed receipts:
**358 checker negatives, one checking-only module, 68 interpreter cases and 550
compiled candidates**. Only two checking rows are new; prior 140 interpretation,
93 runtime and 90 checking rows retain historical identities. Linked rows include
partial/blocked evidence and do not establish CPU parity. Imported-printer/JS-timeout,
erased-hole/TODO, host/resource/foreign/F32.read/error-order/API gaps and the unfinished
688-vector exploration remain unwaived. Dependencies, references, toolchains,
guards, budgets, cancellation and approved demand policy remain frozen. No installs,
pushes, desktop or GPU work.


The [acceptance criterion](cpu-parity-acceptance.md) still requires applicable
execution, normalization, failure, host/API and resource contracts. Historical
receipts retain their original commits; the earlier 370-observation list matrix
is not relabeled as a new run here.

| Evidence identity | SHA-256 |
| --- | --- |
| Tested binary | `49a0d5c124e3f1057203ae4359b2398b291633ea5495c9663283bda83e13ec16` |
| snapshot.json | `5fbbe726d7b4e4d9bb401e62df884e4b973d963d2a1839d949c64c0b1fd5e536` |
| snapshot/teamy-bend.gz | `e1471ee093fbb0404c83830247166c5e081079d79f84a7cd8de36df6b691f5c5` |
| focused.json | `712ad66fe71b0a252dd006f37d5d6980466018d955419ece5b6853561c948ac2` |
| focused-corrected.json | `151b2ffaa16b4b5aa03cc6a13e838368ddb62e8660dd413e75b57de56579e97d` |
| full-gate-final.json | `e66802679a7e8dc5d7065197af38c01bca852272006a36d3316c025c40730dcb` |
| full-gate-final.log | `8b476c07464f797ddbbdb894e2fae56d1d01e55ae5d595398a6a65a68d5b3332` |
| qualified/receipt.json | `df11e0efec2d47af31a91037b118e4afcdecf2e8ac157a93c54d28663403e433` |
| audit/receipt.json | `a1c262f844728650f9dbd426d67daeab71ea4a3b5191de92df350a6466806e23` |
| audit-delta.json | `e8c12d34cfc0cc2297b7cb8d6a8077cae0d8b6c9db1ecaf51a3e294aaf55ae02` |
| controls.json | `452e32293450890e2f558381708df7d850b5b92b822754267e570e8b220fe786` |
| graphs-matching/receipt.json | `067a230771ccd42fc1cac2acc9ee60a33fca5b5be0543e47f3ae9f47d83d6a79` |
| graphs/receipt.json | `60fb71c054632f43ac5263908ab0f6ae306b14f3d5ca4158eea7f2ad9ea82170` |
| graph-probe.rs | `caa52c67570db2965b42742bcc8cff361a4784edc55b0fc5e07e51d6e7a06117` |
| partial-minimization.json | `0847ea1b33dd0628b97c256899389cb15fcf87d090e17d6c0248532cb6016f56` |
| partial-wrapper-preflight.json | `4ba71ffd34ec0b1a25c836075c65e0b511b5dbe7de8f49dab0e7c705be08a257` |
| partial-runtime/receipt.json | `3ce6adc894f57bf4ab424fcdd93047b5fe86fdbc8581f06fc578dc1f7d204198` |
| partial-module-path-correction.json | `e11daf41a6ae27b7c61db5419d54bce3896b8421dfd4adf674be196d1de2b1ad` |
| partial-runtime-corrected/receipt.json | `ff7231d41ed45295c6f93bc1ddd1895a654263ce1e581ea673f372715b841d7d` |
| coverage-summary.json | `ec88767df742c027b1b3f0193dfce7d1decb562b0e3f7d143ea73e60b417080e` |
| inactive-test-cache-reclaimed.json | `b621fdf86ca2b70af0d179db586af0af91e67a0246d816551cf6fcae29d10227` |
