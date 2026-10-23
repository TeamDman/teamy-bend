# CPU fixture applicability and existing evidence

Target: [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
This classifies the **1,254 entries unlinked at checkpoint 7712af8**, then shows
the three newly linked representation fixtures separately. The reproducible
[per-fixture table](cpu-applicability.csv) includes all 1,302 inventory rows,
source hashes, required checking/interpretation, compiled candidates, effect
references, both ledger snapshots and existing evidence. Its
[metadata](cpu-applicability.json) records input hashes and aggregate counts.
Classification is not qualification or a compatibility exception.

**Follow-up:** the [interpretation ledger](cpu-interpretation-evidence.csv) now
links nine originals, renewed at `208b343`, all exact twice. Eight are interpreter-only;
the ninth imports Base but actual upstream C/JS generation refuses its function
result. The historical CSV/table below retain their original runtime-ledger inputs.
Across runtime and interpretation ledgers, 1,242 rows lack those receipts,
including 156 interpreter-only cases and 635 compiled candidates. The separate
[checking ledger](cpu-checking-evidence.csv) adds one exact check-only acceptance
and three phase/construct agreements whose raw diagnostics remain unqualified.
The [approved-demand checkpoint](cpu-approved-demand.md) renews those runtime
and interpretation profiles without changing applicability or inventory counts.
See also the [rewrite checkpoint](cpu-rewrite-printing.md); no missing implementation
or unavailable host is waived by a classification hint.

## What the unlinked entries actually require

These exclusive categories follow the pinned
[gate selection](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/gates/test.ts#L48),
including its exact `main`, `import Base`, foreign-twin and `Error:` rules.
Every case also needs checking; main-bearing positive fixtures then need
interpretation, including the IO driver where applicable. Compiled lanes remain
candidates until their actual applicability and printability are reviewed.

| Requirement | Unlinked at 7712af8 | Unlinked after this checkpoint | Existing strict-check evidence for the baseline group |
| --- | ---: | ---: | --- |
| Checker rejection (`Error:` expectation) | 439 | 439 | All refused, status only; phase/offending construct/exact diagnostics unqualified. |
| Checking without main | 12 | 12 | Eleven accepted; `io/far_types` refused by the proof-only checker because it declares foreign effects. Needs executable-module checking. |
| Interpretation only under the upstream gate | 164 | 164 | 163 accepted; `reg/framed_cell_dep` refused by the proof-only checker because it uses IO. Normalized results remain unqualified. |
| Interpretation plus compiled candidates | 639 | 636 | 190 accepted and 449 refused by the proof-only checker. These refusals are not 449 demonstrated executable defects. |
| Examples/other outside those gate categories | 0 | 0 | All inventoried files are gate-selected tests. |
| **Total unlinked** | **1,254** | **1,251** | Different kinds of required evidence, not one execution-test denominator. |

The original 639 compiled candidates comprise 624 with both C/JS selected and
15 with only JS selected. The three new fixtures have both lanes, leaving 621
C/JS plus 15 JS-only candidates. The interpreter-only category is not synonymous
with pure proof normalization: `reg/framed_cell_dep` has a deliberately spaced,
commented Base import that the upstream gate's exact compiled-selection regex
does not select, but its main is IO. It needs the appropriate interpreter/IO
lane, not a successful strict-proof check. Do not rewrite the source to make the
selection easier.

Of the 163 accepted interpretation-only cases, 129 are in `flatten`. For example,
`flatten/computed_operator_match_002.bend` and its `_dbl` / `_sel` companions
have function-valued mains; their goldens are normalized lambdas/match functions,
not IO output or a machine executable result. They were only source-reviewed at
the classification checkpoint; the follow-up above records actual normalized
results and the correction of their raw rendering gaps. The three original
computed-recursion mains also now match exactly. A separate authored rewrite-term
reproducer exposes the next independent rendering gap; it adds no inventory row.

The upstream gate [enumerates every test directory](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/gates/test.ts#L210).
Names such as `page`, `rfc` and `gfx` do not make their fixtures optional examples.
The twelve no-main programs include eleven proof/checking cases and the foreign
helper module `io/far_types`; they are not missing main executions. Language/API
behavior outside this test inventory remains a separate acceptance requirement.

## Host and platform applicability is a second axis

The classification removes strings/comments before collecting direct API
references and follows the one runnable non-Base module import at this pin,
`reg/far_import` to `io/far_types`. These are source-reference profiles, not a
complete reachability or permission analysis. Counts overlap and must not be
added to the exclusive table above.

Among the original unlinked cases, **48 distinct execution fixtures** reference
one or more of these profiles; the three newly qualified fixtures have none:

| Profile | Referencing unlinked fixtures | Qualification needed |
| --- | ---: | --- |
| Window surface | 1: `gfx/window` | Named display/headless host and its supported/refusal contract. |
| Audio surface | 1: `io/audio_open` | Device or headless/stub branch and cleanup; the source deliberately accepts either open outcome. |
| Filesystem | 9 | Isolated paths, exact artifacts/errors and cleanup. |
| Network | 14 | Reviewed bounded/local host profile, port lifecycle and cleanup. No network fixtures were executed by classification. |
| Environment | 3 | Explicit child environment and platform-sensitive errors. |
| Clock/timer | 8 | Required ordering and named permitted outcomes, not brittle elapsed-time goldens. |
| Foreign companion, including imported module | 19 | Matching C/JS companion and native support review. This does not automatically mean Bun is required. |

Eleven unlinked fixtures contain CPU-offload syntax. Their CPU fallback remains
required independently of separate GPU qualification. No offload marker is a
reason to defer CPU execution automatically.

The earlier inventory's broad `device-review` hint matched 38 unlinked rows,
including comments about syntax `App`, checker errors, ordinary collection code
and scripted `App.play`. It cannot establish 38 hardware requirements.
`gfx/app_play` uses scripted frames with no window; `gfx/app_linear` combines
scripted frames with file effects. Only `gfx/window` and `io/audio_open` directly
reference device operations among the unlinked runtime fixtures. They remain
unqualified, not waived; headless refusal/stub behavior can itself need CPU tests.

## What existing suites establish

The complete [strict acceptance audit](../examples/audit_upstream.rs) was rerun
against the preserved `cc0f3e8` runtime binary: 1,302 fixtures, 364 accepted
positives, 489 refused positives and 449 refused nonzero-exit expectations,
zero abnormal exits. The 449 include **439 checker negatives and ten runtime
negatives**. The latter belong to the existing runtime ledger. This audit invokes
`check`, not executable checking or evaluation, and does not compare diagnostics.
Its current results equal the earlier audit, but are not 1,302 parity passes.

| Existing evidence | Applies to | What it does not establish |
| --- | --- | --- |
| `tests/parser*.rs`, syntax unit tests | Source syntax, imports, annotations, numeric/template parsing and rejection controls. | Exact coverage of every upstream parse/page/printer expectation. |
| `tests/kernel_*.rs`, kernel unit tests, `cli_proofs` | Strict types, quantities, descent, equality, normalization and false-proof refusal. | Executable IO/foreign acceptance or every upstream negative diagnostic. |
| `tests/executable_*.rs` and executable checker unit tests | Sealed numeric/effect/foreign assumptions, signatures, erasure and executable safety boundaries. | A complete executable acceptance matrix for all 639 baseline compiled candidates. |
| `compiler_c`, `compiler_javascript`, `runtime_data*`, `serve_protocol` | Pure values, data evaluation, compiled results and protocol/ownership bounds. | Per-fixture upstream normalized results for the 164 interpreter-only cases. |
| `tests/runtime_*.rs`, `compiler_executable_*.rs`, `cli_effects` | Native/C/JS arithmetic, IO, scheduler, file/network, failure/cleanup and resource controls. | A passed original fixture merely because the same API has an authored regression. |
| `kernel_image_event`, `compiler_executable_app`, host-storage/network/file tests | Headless image/event/App behavior and platform adapter contracts. | Window/audio device or fresh cross-platform qualification. |
| `compiler_executable_c_gpu_off`, GPU mark tests | Explicit CPU fallback and bounded failure behavior. | Hardware GPU passes or all annotated upstream fixtures. |
| `compare-cpu.ps1`, negative runner and retained comptime/profile probes | Exact named runtime receipts in the 51-row ledger, including explicit failures. | Checker/normalizer/API coverage for unlinked entries. |

A hash-based scan found **nine exact upstream source fixtures** referenced by
literal `include_str!` in repository integration tests; all nine already appear
in the runtime ledger. It found zero such exact references in the 1,254 baseline
unlinked rows. This is deliberately a narrow mapping: it neither denies coverage
from inline/adapted tests nor treats those as exact upstream-fixture qualification.
The CSV names every detected integration-test source. The full standard gate
passed **776 tests, zero failed, 49 ignored**; it exercises the related suites
above, not all 1,302 original fixtures as full differential cases.

## Actual remaining acceptance work

1. Qualify checker rejection phase, offending construct and required diagnostics
   for the 439 negatives. A proof-mode refusal before the intended error is not
   agreement. Use the matching executable or strict-checking surface as required.
2. Close the twelve check-only cases using their proper checking modes, including
   the foreign helper module. Eleven existing acceptances reduce uncertainty;
   they do not create runtime work where upstream selects none.
3. Compare interpreter/normalizer results for the 164 interpretation-only cases,
   using executable IO for the spaced-Base outlier. The 163 accepted strict cases
   are a useful next bounded evaluation pool, not 163 missing implementations.
4. Review and execute the 636 remaining compiled candidates in the applicable
   C/JS/native profiles; preserve actual printability refusals and companion
   exclusions with source or oracle evidence. Use host review before effects.
5. Resolve the already recorded resource/foreign/oracle gaps, pending demand
   choices and broader language/library/CLI/serve/API inventory. Complete the
   declared worker and platform profiles. New coverage cannot erase existing
   failures or turn blocked lanes into passes.

These refine the existing [acceptance criterion](cpu-parity-acceptance.md);
no gate was relaxed. A useful next small CPU step is a bounded pure
interpretation family drawn from the 163 already accepted strict cases, with
actual normalized-output comparison. It does not require Bun or GPU hardware.

## Reproduction

Use the frozen reference and existing Python standard library; no package is
installed. Preserve the 48-row baseline via Git, supply the retained strict audit
JSON, and generate into a fresh directory:

```bash
git show 7712af8:docs/cpu-runtime-evidence.csv > <baseline-ledger.csv>
python3 scripts/classify-cpu.py --reference <reference> --baseline-ledger <baseline-ledger.csv> --audit <strict-audit.json> --output <fresh-directory>
```

The helper verifies the clean pin, inventory/source/expectation hashes and ledger
consistency; it executes no Bend source. Input and output hashes plus raw audit
logs remain in private `cpu-word-representation` evidence. No private diagnostics
or machine paths are included in the published classification files.
