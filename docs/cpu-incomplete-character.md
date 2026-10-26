# Terminal incomplete-character diagnostic metadata

Tested source: `1a135e437d91c99072bbd880b9a6d43ceb984d2a`. Standard gate: **1354 passed / 0 failed / 49 ignored**,
197 groups, 1043.121 seconds. Formatting, standard strict Clippy, build and
real TCP/UDP provider checks passed. The source remained clean and post-gate CLI
hash matches the differential snapshot. Focused receipts passed 105 syntax tests,
one direct terminal-state test and 15 CLI/runtime/template tests.

Final-source matrix: **908 observations / 142 repeat pairs / zero timeouts**.
[Validation receipt](cpu-incomplete-character-validation.json) and
[compact streams/runners](cpu-incomplete-character-results.json.gz) preserve exact commands,
hashes and classifications. Matrix SHA-256 `c044fce28e86fa0b52ec735b89edba287abf1044847161994a59000b2cab9025`;
full-gate log SHA-256 `1cab3a1129106be0c74a936d4caac0c6c0eed94ca1f2fd862c53624ca53e974f`. Minimum free space:
12.159 GiB workspace / 1.450 GiB temporary.

This checkpoint qualifies the unchanged list fixture. The unchanged argument
fixture remains different: its preceding parameterized definition reaches the
existing nontrivial-lowering barrier before recovery reaches the malformed atom.
The requested two-case family is therefore **one exact / one unresolved**.
The depth-refusal neighbor remains a separate, unwaived resource-policy difference.

## Mechanism and pinned contract

Pinned [parse_char](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1588)
consumes one raw code point, and the [Char term parser](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1930)
then demands a quote without skipping whitespace. At an existing incomplete-token
barrier, the port now checks the original source's quote and first unescaped
scalar against the fault token, emits optional closing-quote metadata, and returns
the same internal refusal immediately. It constructs no character value and
advances no token. An explicit Terminal outcome distinguishes this from continuing
recovery. Raw public loader errors and rejection remain authoritative.

The original display source is capped at 4096 bytes before excerpt generation,
including the direct lexical-prefix path. Existing 64-token/4096-byte prefix,
4096-header/64 KiB copying ceilings, 64-level expression/import guard, literal,
checker and runtime limits are unchanged. Backslash/newline first codes,
astral observed UTF-16 units, mismatched source coordinates and oversized sources
retain refusal without this metadata. Earlier grammar, duplicate, template,
imported-law and nontrivial-lowering barriers still win. There is no new second
parser pass, evaluation, cancellation capability or policy exception.

The implementation adds only private diagnostic methods and an internal outcome.
Newly formatted CLI failures use the existing invoke Ok(exit_code=1) boundary;
raw ParseError fields remain unchanged. Repository regressions inspect status;
external invoke consumers remain unaudited.

## Exact observations and limits

The original 22-negative cohort and preceding 12 grammar controls remain exact.
The eight four-quote contexts improve 5/3 to 6/2; the new 12-case cohort improves 1/11
to 5/7; two older unterminated/cross-line cases improve 0/2 to 2/0. These are seven
observed diagnostic closures, not seven semantic repairs or whole-census gains.
All 56 negatives still refuse loading; check/eval/run and C/JS/native emission
retain status 1, empty stdout and absent artifacts. All 112 negative and 10 accepted
raw-loader streams match baseline. CRLF fixture and golden bytes have explicit
-text attributes and were verified against staged and committed Git blobs.

Five accepted controls retain four native pure outputs and one actual native IO
output. Forty generated C/JS executions produce 20 exact corresponding-backend
comparisons; ten generated port artifacts remain byte-identical to baseline.
C uses upstream's O3 profile and port C retains strict warnings. Four upstream
pure normalization rows establish successful exit only, not full graph equality.
Strict IO proof exclusion is separately retained.

The nine current diagnostic differences include the original argument and depth
controls plus seven new f-first neighbors. The retained trace establishes the
lowering barrier for the original argument; the f-first controls remain
unqualified. Unicode cases additionally lose the corrected prefix when legacy
lexing fails, and escaped/astral observations remain outside the helper's
supported metadata contract. These mechanisms are not conflated. Direct argument controls deliberately
use an unresolved name: the malformed literal fails before name checking. They
are not successful runtime programs.

One possible route for the original argument is a separately reviewed design
that reuses already-completed primary declarations at a proven identical
source/token boundary. Its safety has not been established. Skipping a function, cloning post-failure state or replaying
its lowering is not implemented. The retained temporary trace demonstrates the
barrier at flatten_body_occurrence while finishing f; instrumentation was removed
before the clean-source build and all final validation.

## Parity inventory reconciliation

[Inventory reconciliation](cpu-incomplete-character-gap-inventory.json) separates
historical full measurements from current subsets. The last full 1,302-original
census at c5edea8 measured 366 both accepted, 405 exact rejections, 494 acceptance
differences, 31 differing rejections and six upstream stack failures. Outside the
same 385 historical IO rows it measured 140 differences (109 acceptance/31
rejection). Those are historical counts; this checkpoint does not silently
subtract authored diagnostic improvements or renew that IO cohort.

The 31 historical differing rejections were rerun on this clean source:
**31 remain different**, with 17 load/load, 11 check/check and three check/load
phase pairs. Symptom triage gives 11 proof-mode preemptions, three resource
refusals, one TODO-contract difference and 16 rejection-content/provenance
candidates requiring individual audit. No category waives its observations.

The earlier 109-row acceptance classification found 92 executable-check acceptances,
15 resource load refusals and two missing Window/Audio check refusals. It was not
rerun here, and 92 is not a runtime-pass count. Its cause breakdown remains in
the reconciliation JSON. Existing evidence links cover 516/1,302 originals; 786
unlinked entries comprise 256 checker rejections, 517 interpretation/compiled
candidates, 12 interpretation-only and one checking-without-main. Linked receipts
include failures; unlinked entries are not all missing CPU runtime tests.

Next substantive family: unchanged run/array_fill_fold and run/array_poly_elem.
Fresh baseline native executions produce 140375 and 102122; actual generated
C/JS execution, reusable ownership, enum/F32 layouts and adjacent bounds/COW
controls still require qualification. No defect is invented from that missing
coverage. Fresh baseline base/num_kit also succeeds with 31 lines, showing that
an older kernel-depth note is historical. Those selection probes are explicitly
bound to f64eb0a/02155ff source, not claimed as final-source runtime qualification.

nat_first, compact Nat/string lowering, Base coordinates, F32.read, source holes,
foreign/host/Bun/oracle limitations, external invoke consumers and supplementary
all-target Clippy failures remain open. No whole census was run here. Standard
Clippy is not the supplementary all-target lane. Full CPU parity remains incomplete.

## Reproduction and preservation

Use the frozen activation environment, Rust/Cargo 1.96.0, formatter
nightly-2026-05-24, Node 24.19.0, Clang 19.1.7 and PowerShell 7.5.4. Upstream remains
e6676b080f25b1bc1bf5b5b7d7a17e22f8022599, mounted docs remains
50e89d6897ce2a65167ef65b17c5fa83fe849158 and Cargo.lock hash remains
d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4.
Host build overrides remain CARGO_PROFILE_DEV_DEBUG=0,
CARGO_PROFILE_TEST_DEBUG=0, CARGO_INCREMENTAL=0, CARGO_BUILD_JOBS=4,
CARGO_NET_OFFLINE=true and BEND_GPU=off. These do not change Bend's generated O3 profile.

```sh
cargo test --locked --offline --features extended_observability --lib syntax::
cargo test --locked --offline --features extended_observability --lib terminal_character_metadata_refuses
cargo test --locked --offline --features extended_observability --test cpu_empty_character_context --test cpu_raw_apostrophe --test cpu_template_occurrences --test cpu_malformed_literal_diagnostics
cargo build --locked --offline --features extended_observability
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Compact receipts include runnable differential and subset scripts, exact commands,
statuses, hashes and before/after streams. Public copies replace only explicit
machine roots; original stream hashes remain. Runs compare original bytes.
The first focused runs exposed stale no-metadata assertions and an incorrect
assumption that the f-first argument could reach the new helper. Those failed
receipts and the barrier trace are preserved. Updated expectations follow renewed
upstream output; the original argument was not forced to pass.

Workspace floors remain 11 GiB prebuild/10 GiB active, with 1 GiB temporary. Source
bundles and unique evidence have local verified backups; Git preserves compact
receipts but not large binary snapshots/raw logs. Publication follows independent
final review and the standing non-force feature/main authorization, with remote
identities recorded separately. No desktop/GPU work, installation, dependency
change, cleanup or release was required. Blocked Library uploads were not retried.
