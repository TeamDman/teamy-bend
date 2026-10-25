# Compile-time argument occurrence diagnostics

Tested source: `59f04c46f6b3a503367c95884338f9404216f7d1`. The subsequent evidence checkpoint changes only
documentation. Target: `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.
The five retained `comptime/later_def` source-location failures are corrected.
**Full CPU parity remains incomplete.** Dependencies, toolchains, evaluation
semantics and safety limits are unchanged.

## Source contract and implementation

Pinned [parse_var](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1635)
converts a free simple-name argument into a reference carrying its first
compile-time parameter-use span. An already qualified reference keeps its caller
span. Forwarding the converted reference through another template preserves that
first span rather than assigning the second template's location.

For example, both implementations reject the original's forward reference to
`later`, but the port previously pointed to `app(~later, 3n)`. It now points to
`f(x)` inside `app`, matching upstream's complete stderr and exit status 1.
The original fixture bytes and four prior reproductions remain unchanged in
`tests/fixtures/cpu-definition-provenance`. Their historical `pending-cases.json`
name and old failure receipts are retained, not rewritten into past successes.

The parser now carries a private occurrence wrapper beside the **same shared
Rc reference**. It records source anchors on actual parent edges and surface
roots, separately from fallback substitution origins. No core term fields,
reference nodes, binder IDs or template cache keys change. Bare-name syntax is
tracked only to implement the pinned diagnostic rule; it does not change lookup,
macro rebinding refusal or evaluation. Qualified and forwarded controls verify
the distinction.

Checker error mapping keeps the rejected node identity separate from its display
source identity. A deeper rejected node cannot steal its parent's occurrence span
merely because both share a transformed source root. Existing fallback origins
retain their behavior. Macro substitution and checker beta substitution carry
metadata only across the actual replaced edges, respecting binder shadowing.
Metadata is discarded before evaluation. This adds no mid-normalization
cancellation mechanism and changes no existing cancellation boundary.

Independent review found and corrected three missing propagation paths before
acceptance: `do` local assignment child order, captured/beta-reduced lambda root
provenance, and unchanged trailing arguments of quantified datatype `+` syntax.
The last follows pinned [parse_term_atom](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1799),
which replaces leading quantity arguments while preserving trailing objects.
Review of the clean tested commit found no remaining source-review blocker;
the reviewer did not run tests or claim test results.

## Host-stack regressions caught by the full gates

The initial candidate `2a5188663d6dfdc28e72b660818f58f09ea1d4ee` passed its
focused comparisons but failed the full gate: the unchanged 90-application
`kernel_core` regression overflowed the default Rust test-thread stack.
That failed gate completed 812 passes and 48 ignores before one test target
aborted; subsequent tests were unrun. Its log, captured CLI, failing test binary
and source commit remain preserved. It is not a passing checkpoint.

Unoptimized x86-64 disassembly measured `infer_inner` growing from 10,072 to 10,872
bytes of host frame storage and `check_inner` from 11,528 to 11,912. The combined
recursive frames crossed the host stack boundary before Bend's unchanged depth
guard. The correction extracts the existing reference-inference rule verbatim
into a private helper, reducing the recurring inference frame to 9,176 bytes.
The helper has no new `step`, `enter` or inference transition; reference lookup,
structural-decrease checks, unfilled-law errors and diagnostic ordering remain
identical. Independent review verified the rule body differs only in indentation.

The reference extraction passed all 19 core tests, but candidate
`c45cfec3d4350debd1557635039d40c7d15ee65c` then aborted in the unchanged
`array_rendering_preserves_complete_trees_and_partial_boundaries` parser guard
control. That gate completed 870 passes and 48 ignores before the normalization
test target aborted; subsequent tests were unrun. Its failing binary and logs
are also retained as failed evidence.

Here the occurrence wrapper grew `atom_inner` from 20,664 to 23,512 bytes and
`expression_inner` from 6,696 to 8,424. Extracting the existing rewrite and quantified
atom rules into helpers reduces the recurring atom frame to 18,824 bytes.
Independent review verified unchanged token consumption, spans, scope restoration,
fresh IDs and parser depth accounting; only whitespace and final tail returns
changed within the extracted bodies. The unchanged 64-deep array input now returns
the original parser-limit error instead of aborting. A new regression also requires
64 nested plus prefixes, binders and rewrite forms to reach that same guard.

The final clean source passes 103 focused core/normalization/parser tests, eight
occurrence units and four new-target integration tests, then the renewed full gate
and matrices. No stack size, Bend guard, work allowance, compiler optimization
level or semantic policy was raised to obtain these passes.
These frame sizes describe this pinned host build, not a cross-platform guarantee.

## Qualified scope and results

Twenty cases match exact rejection status, stdout, stderr and absent output
artifacts across repeated upstream/port check and executable compiler boundaries:
the five retained failures, the qualified-name control and 14 adjacent cases.
The additions cover nested forwarding, ordinary arguments, constructor field 4,
nonfunction application, multiline parentheses, multiple shared uses, macro and
ordinary beta substitutions, `do` assignment and quantified type arguments.
All 20 load successfully and reject during checking. No program executes after
checking refusal; C/JS generation, build and runtime are distinct phases.

Two further macro-rebinding cases still reject during loading in both
implementations, but their diagnostic spelling/location formatting differs.
Their upstream captures and port internal-error assertions are retained as
**unqualified diagnostic cells**. No acceptance policy was changed to fix them.

Two positive programs cover shared values, nested templates, lambda macros and
cache reuse. Their complete normalized graphs and text agree with the pinned
interpreter; native strict evaluation and pure C/JS constructor values agree.
Actual executable emission refuses these no-Base programs consistently. This is
pure API coverage, not invented upstream executable-C/JS coverage.

| Check on the tested source | Result |
| --- | --- |
| Standard gate | **1,135 passed / 0 failed / 49 ignored** |
| Format, strict production Clippy, all-feature build, Node provider | Pass |
| Focused integration Clippy | Pass |
| Focused core/normalization/parser targets | 103 passed on the default test-thread stack |
| New integration target | 4 passed; included in full gate |
| Occurrence-filtered unit controls | 8 passed, including 5 new units |
| Target differential matrix | 450 process observations; 24 cases |
| Renewed prior runtime/diagnostic matrix | 246 observations; historical failures retained |
| Combined matrices | **696 observations**, zero timeouts; repeated groups deterministic |
| Phase-only probes | 20 checking refusals; 2 loading refusals |
| Full inventory census | All 1,302 acceptance/rejection statuses unchanged |
| Changed census stderr streams | Exactly 1: `comptime/later_def`, now exact upstream match |
| Renewed prior 489-case checking subset | 32 exact / 385 strict IO exclusions / 72 unresolved |
| Prior ten diagnostic/acceptance cells | Six labels exact; three pattern anchors and `unsafe_many` unresolved |
| Positive and hole/TODO controls | 14 positive observations; 18 retain three unwaived differences |

The new units compare tracked/plain parsed structure, internal error equality,
remaining fuel, depth and fresh IDs for 14 negative cases and two positives.
Pointer tests establish that three parameter uses and the caller retain one
reference allocation with four strong owners, unchanged after provenance is
removed. Cache tests keep one instance per specialization. Other controls cover
shadowing, arbitrary field indices, direct-child versus deeper-source remapping,
legacy fallback mapping and exhausted diagnostic presentation allowance.
The full gate retains aggregate composition, lazy/demand ordering, resource,
structural, source-hole and cancellation regressions.

## Remaining differences and compatibility

The 72 renewed differences comprise 64 upstream-accept/port-reject outcomes and
eight differing rejections/preemptions. Strict IO proof exclusions are separate
from executable success and are not missing-implementation waivers. The two new
rebinding diagnostic controls and four prior diagnostic/acceptance cells are
separate, potentially overlapping scopes; do not sum ledgers as disjoint defects.

The runtime renewal retains two repeated upstream C-generation failures for
forged IO and two direct-IO Node host failures. Failed generation still leaves C
build/execution **unrun**. Imported-JS printer timeouts, dependent-pair/minimal-IO
C failures, F32.read disagreements, competing-error order, host/foreign/resource
and API gaps, and the incomplete 688-vector exploration remain unwaived.
Historical supplementary all-target Clippy failures were not rerun; a passing
standard/focused lint gate does not claim an all-target pass.

No new public signature or term-serialization field changes. Existing
`CliOutput::exit_code()` handling and emitter/`DefDecl::completion_parameters`
compatibility caveats remain. External consumers were not audited: a successful
host `Result` alone still must not be treated as a successful command status.

Detailed links now cover 407 originals; 895 remain unlinked: 344 checker negatives,
one checking-only module, 13 interpreter-profile cases and 537 compiled candidates.
Runtime/interpretation links remain unchanged. Linked evidence includes partial,
blocked and historical outcomes; these counts do not measure completed parity.
The [acceptance criterion](cpu-parity-acceptance.md) remains unchanged.

The next independent diagnostic work is the three retained pattern-anchor cells.
A substantive applicability-first candidate batch remains
`parse/datatype_declaration_form`, `empty_datatype` and `empty_param_group`.
No future work has qualification credit, and no new user decision is required
for the correction completed here.

## Reproduction and preserved evidence

```sh
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo test --locked --features extended_observability --test kernel_core --test kernel_normalization --test parser --test parser_numeric --test parser_templates
cargo test --locked --features extended_observability --test cpu_template_occurrences
cargo test --locked --features extended_observability --lib occurrence
cargo clippy --locked --features extended_observability --test cpu_template_occurrences -- -D warnings
node "$UPSTREAM/bend2/main.ts" "$FIXTURE"
node "$UPSTREAM/bend2/main.ts" "$FIXTURE" -o "$ARTIFACT"
"$BEND" check "$FIXTURE"
"$BEND" compile "$FIXTURE" --executable --target c --output "$ARTIFACT"
```

Paths are caller-selected. The committed manifests, original fixture, expected
stderr and complete positive graphs/text make the bounded comparisons reproducible.
The compact [case ledger](cpu-template-occurrences-evidence.csv) links cases and
lanes to private receipts by hash. Those receipts retain exact commands, bytes,
statuses and cleanup outcomes; they are not stored in Git. Renewed baseline
cases reuse existing fixtures and are summarized rather than duplicated.
Matrix children have 30-second deadlines; the gate has a 1,200-second deadline.
Integration child processes retain their 20-second bounds. Upstream C renewal
uses the CLI's `-std=c11 -O3 -lpthread -lm` and runs
`--threads 1 --gpu off`; pure port C uses O3 with
`-Wall -Wextra -Werror -pedantic -pthread -lm -ldl`.

Frozen host: Rust/Cargo 1.96.0, fmt nightly-2026-05-24, Node 24.19.0,
Clang 19.1.7 and PowerShell 7.5.4. Host build overrides are
`CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`
and `CARGO_BUILD_JOBS=4`, with `CARGO_NET_OFFLINE=true BEND_GPU=off`.
These host settings are distinct from Bend runtime optimization profiles.
Work/depth limits remain 2,000,000/128. Cargo.lock SHA-256 remains
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

The gate started with 11.41 GiB workspace headroom and reached
10.37 GiB minimum, above the 10 GiB stop floor; preflight required 11 GiB.
The separate temporary floor remains 1 GiB. Rebuilt test binaries explain the
transient growth; only completed, inactive rebuildable caches were reclaimed.
Final post-cache workspace headroom is 11.46 GiB.
All source, unique failure evidence and historical captures remain intact.

Raw logs, generated artifacts, command/stream receipts and captured binaries are
**outside Git preservation**. Only reviewed source, compact fixtures and ledgers
are pushed. No main merge or release. Captured tested binary SHA-256:
`e37d64c378bd19fa8a753256c5b204d0430c09bd35c985f412928c74301ec2ec`.
Prototype compile failures, the two initial full-gate stack overflows and an early
runner start before snapshot creation are preserved separately; none is counted
as successful validation.

| Private receipt | SHA-256 |
| --- | --- |
| `full-gate.json` | `70b12dc2abbb3c1d32b78f4b6295804068cab8f727a80bcbe14de613c7837d1d` |
| `focused-structure-current.json` | `ed803df8fc96a0aa4602fa34e5b0230fffd69491ed2090ba1ccad1be455bc7c4` |
| `stack-environment.json` | `3587055f20289c5538403abd3f6e57dc859d40deabe46ce26210789071c961b6` |
| `parser-stack-frame-final.json` | `62324de462035bfad70c3cbfdb7708b825f73e6ad9418ba4d5b9abba59ee90af` |
| `../cpu-template-occurrences-final/stack-frame-probe-final.json` | `22d3e2ac27e4c2eac489ff9f7dd0efac0775dc57d43656d96e6bdc20354641ac` |
| `../cpu-template-occurrences-final/full-gate.json` | `b30cb863c3ebc6a2ca42d0f08d01767afacef33c872ddda0a228b6b6dff7b77e` |
| `../cpu-template-occurrences-final/failed-kernel-normalization.json` | `e33fa60e554646468e0db8eb2db29f9ccd9c311c83338a1087e49b72a8ffe0d2` |
| `../cpu-template-occurrences/full-gate.json` | `a6a7fc3075afbb24f50376ff7b00b0b4c1758aa2118828e01daf5958d630dfa6` |
| `../cpu-template-occurrences/failed-kernel-core.json` | `24cdbad12d8d9fe85f6c51732a51ceb68e5683de209a3f1323c2047050378013` |
| `focused-clippy-current.json` | `90d8656aa0f7171c82446363e3f7a68aa115cb6ea9205afb7d20fce143f726b3` |
| `focused-integration-current.json` | `bd7cbec97879ba13b0b3dc523ecd9c3eb8d635505c0adf44167eb839a0560a0e` |
| `focused-unit-current.json` | `21b2c79371c8ab00ad2c9777522bb4c218610a69054e228beb4fefc4aa377591` |
| `snapshot.json` | `a29341ce12ffe71c88e25b3d16c08ea03c8a0a98909757ce21498a1c63e94e8e` |
| `matrix/receipt.json` | `a51f202fedac9183efb85eeb26c912e21293f2db4213656bdee24141110e72ee` |
| `baseline-matrix/receipt.json` | `b195d731b0e7d1d372a215dca8fc1f8a8df2decf2d014885d66803c713069eae` |
| `phase-controls.json` | `d4c744719fb8e5d39216ecbb09241c5b7a6948be761746a6679426b2dacbfe1b` |
| `audit/receipt.json` | `ee8663ec22282d796e0f6a2994cd995ae4edf8897cea2029967391a8d7b57c84` |
| `audit-delta.json` | `c683cb7aa3af37fc6412613126d2af738473434f937672773ea17a0a87387888` |
| `remediation-audit-comparison.json` | `e344f600e87372e53eecbec9cb704f6bf99ea0d3e062b59fdff8a00e16e70f9f` |
| `audit-upstream-delta.json` | `eb37c202a5667dd1b79b9f025a3538682e65a9097bc391ad4f685fc090aae7f1` |
| `renewed-checking.json` | `104b6a7bc4e7cda15cd5a819b66b77a358e7de8ef93fe210081ad6e9d6413748` |
| `checking-renewal-summary.json` | `80bb483b210ca4c694532c8bf709506d64908d15c933c47f676aa6ad10b361f5` |
| `retained-controls.json` | `3be7ccbd6aaed0befa85fd0ffbaf32cc02e51f1049c09a187231a0da8ab46938` |
| `controls.json` | `7020c895cb8cc0b95757bfd76dbd85285e6b58aab29f4747aa603283eccda4d2` |
| `independent-review.json` | `d142b698ace42a1d52601aeac056d890e6b94e94dc6a88c3cb4e6a457f00651a` |
| `frozen-inputs.json` | `80599a5e0cfeee8abb6d015ebd0da8e1a4dba0e580d27eb97d67226c0e7a02f7` |
