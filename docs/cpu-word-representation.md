# CPU word operator and representation qualification

Target: unchanged [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
Candidate: `8e803a9ecaa5031c675a0bc95ffdd8b232386673`, based on `7712af8`.
Production source, tests, dependencies and toolchains are unchanged from `cc0f3e8`.
Only the comparison runner, classification helper and evidence are extended.

## Exact fixture outcomes

| Unchanged upstream fixture | All five runtimes, two repetitions |
| --- | --- |
| [run/u32_mod_operator](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/run/u32_mod_operator.bend) | stdout `2 17 1.5\n`, empty stderr, exit 0 |
| [reg/word_copy_native](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/reg/word_copy_native.bend) | stdout `22\n`, empty stderr, exit 0 |
| [reg/u32_structural_view](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/reg/u32_structural_view.bend) | stdout `14\n`, empty stderr, exit 0 |

**30/30 executions match** in native Rust, generated port C/JS and upstream C/JS.
No parity defect was demonstrated and no runtime correction was needed.
The structural-view fixture's comment claims C emission refuses; that refusal
was **not reproduced** at the pinned revision. Unchanged source generates C,
compiles and runs successfully. Its lane is required and passed, not excluded.

This agrees with the current pinned C emitter: [emit_match](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L2622)
decomposes U32/F32 words into bit fields; [emit_ctr](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L2213)
packs those fields back into words. The old fixture comment is not the current
implementation contract. This qualifies this structural view, not arbitrary
reflection or every dependent Word representation.

## Validation and preservation

- Full Linux standard gate: **776 passed, 0 failed, 49 ignored**, exit 0;
  formatting, strict Clippy, all-feature build and Node-provider checks passed.
- Strict all-target/all-feature Clippy: exit 0.
- Expanded suite: **308 exact / 320 executions**, 32 cases twice. Twelve existing
  upstream-JS Bun-host failures remain explicitly classified. All port runs pass.
- Every repeated stdout/stderr/status is stable; zero timeouts. C uses
  `--threads 1 --gpu off`; port C retains strict warnings; child deadlines remain
  60 seconds. Final snapshot SHA256 is
  `cbc9983c357d497dac12bd22e3502a7f6f27fe94a90989b53cb64e0dd59f352f`.
- The complete strict acceptance audit was rerun with the preserved prior runtime
  binary; counts are unchanged. This is status-only proof-mode evidence, not an
  executable parity matrix. The [applicability report](cpu-applicability.md)
  explains the modes and exact remaining requirements.

The prior 48 ledger rows and their exact candidate/profile receipts remain
unchanged; the three new rows name this candidate. The negative, comptime,
multiworker and demand/wildcard matrices were **retained, not rerun** in this
checkpoint because production/runtime/test source did not change. Their unresolved
resource, native-foreign, Bun, stack-profile and contract cases remain required.
No pending demand choice or oracle exception was implemented. The earlier public
KernelError/run_main compatibility caveat is unaffected; no API changed here.

The ledger now links 51 upstream fixtures, leaving 1,251 unlinked. The original
1,254 unlinked entries comprise 439 checker negatives, twelve check-only modules,
164 interpreter-only cases and 639 compiled candidates. Three of the latter are
now qualified in five runtime lanes, leaving 636 compiled candidates. The new
rows' direct upstream CLI and broader checker/API cells remain unverified.
Do not equate all unlinked entries with missing CPU runtime tests or claim full
CPU parity from these counts.

## Reproduction and next step

With activated frozen tools, `CARGO_NET_OFFLINE=true` and `BEND_GPU=off`:

```powershell
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo clippy --locked --all-targets --all-features -- -D warnings
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-representation-dir> -Suite representation -Repetitions 2
pwsh -NoProfile -File scripts/compare-cpu.ps1 -Reference <reference> -OutputDirectory <fresh-positive-dir> -Suite all -Repetitions 2
```

The final receipt contains the new family within `-Suite all`; a separate dirty
preflight also passed all 30 observations. Private `cpu-word-representation`
evidence preserves both receipts, exact commands, binary/source hashes, generation
and compilation output, gate logs, strict audit and classification controls.
Every earlier checkpoint remains intact. No upstream/doc reference changes,
installation, dependency/toolchain change, push or main merge occurred.
Fresh Windows/macOS, Bun, GPU/hardware profilers and wider corpus/API cells remain
unrun. Next bounded CPU work can compare a pure interpretation family from the
163 unlinked cases already accepted by the strict checker; the missing evidence
is normalized behavior, not initial acceptance or GPU execution. A concrete
three-case starting point is `flatten/computed_operator_match_002.bend` and its
`_dbl` / `_sel` companions, whose mains normalize to functions.
