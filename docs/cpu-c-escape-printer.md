# Executable C escape-printer checkpoint

The C language printer now emits the pinned braced hexadecimal escapes. Three
scalar programs whose C output previously failed readback now round-trip.
CPU parity remains incomplete.

Production comparison source: `3d809d30e94e86e7a2b7eea772045c22dbb956d0`.
Final tested source: `d36956122f5b2f27ae429c440ccef3028e13af23`; the follow-up changes only the C test probe's
Windows stream setup. The prior remote checkpoint
`a37fd81f37564b8d1bd11ea4b945c533f75e0c51` remains preserved.
Upstream is frozen at `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.

[Validation](cpu-c-escape-printer-validation.json),
[exact compact outcomes](cpu-c-escape-printer-results.json.gz),
[fixture controls](../tests/fixtures/cpu-c-escape-printer) and
[review](cpu-c-escape-printer-review.json).

## What changed

Pinned [show_chr](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L5526)
prints special escapes first, then braced lowercase hex for remaining C0 controls,
DEL, surrogates and codes above U+10FFFF. The port used unbraced four-digit hex
for C0 controls, raw DEL and invalid UTF-8 for non-scalars.

Only `tb_show_character` changes: the same predicate and braced spelling, plus a
24-byte buffer. Even the full-u64 helper input needs only 20 output bytes and a
terminator. This helper safety probe does not claim language acceptance of a
64-bit Char. Actual Base character codes remain U32.

`tb_show_write` still charges the bytes actually printed against the same 8 MiB
limit. Node, depth and work accounting are unchanged; no budgets are raised.
Special escapes, quote selection and valid UTF-8 remain unchanged. The strict
compiler's JSON encoder, raw console/file encoding and Rust public APIs are
untouched. This is a deliberate correction to executable C pure-result spelling.

Non-scalar constructors already pass checking. Matching their pinned **C**
spelling does not change literal acceptance or JavaScript's refusal policy.
Native pure output continues to show constructors, not quoted Char/String sugar.

## Before and after

| Case | Before port C | After / pinned C |
| --- | --- | --- |
| Control U+0001 | `\u0001` | `\u{1}` |
| DEL | Raw byte 7f | `\u{7f}` |
| Surrogate constructor | Invalid UTF-8 ed a0 80 | `\u{d800}` |
| U32 maximum constructor | Invalid UTF-8 ff bf bf bf | `\u{ffffffff}` |

Unchanged originals: `show/string_escape`, `show/char_unicode`,
`show/string_unicode`. All copies are byte-identical and Apache-attributed.
Authored controls cover all ASCII via three bounded strings, C0/DEL, quotes,
backslashes, empty strings, scalar boundaries, astral values, line separators,
non-scalar constructors and raw IO.

The unchanged 95-character ASCII aggregate hits the existing kernel input-depth
limit. It remains an acceptance/resource difference, with port emission refused and execution unrun. Separate shorter controls do not qualify that failed aggregate.

## Evidence and limits

- Final standard gate: **1313 passed / 0 failed / 49 ignored**,
  194 groups, 950.135 seconds. The earlier production checkpoint also
  passed its full gate; receipts retain both tested commits rather than summing
  repeated passes into new coverage.
- Focused tests: five passed; adjacent JSON, executable C, raw file bytes, sharing
  and cancellation tests: 57 passed; targeted strict Clippy passed.
- Main differential matrix: **495 observations, 167 deterministic pairs and
  161 single observations**, with zero unexpected results or timeouts. The 161
  comprise 101 emission/build commands and 60 canonical-readback commands.
- **64 port C/JS execution observations match their corresponding upstream
  lane exactly:** 56 successes and eight JavaScript non-scalar refusals. There
  are 68 upstream execution observations; four additional upstream successes
  belong to the aggregate that the port refuses. These are not port passes.
- Eleven canonical scalar readbacks pass both checkers and normalize to the
  original values. The integration tests compare complete normalized structures
  for both C and JS outputs. The prior C readback failures were exactly
  `show/string_escape`, `controls/controls` and `controls/characters`.
- Four canonical non-scalar C readbacks pass upstream but still fail the port's
  scalar literal decoder. JavaScript rejects the original constructors at runtime.
  These remain disagreements, not new exceptions or full round-trip claims.
- All 15 successful native pure normal forms retain constructor text distinct
  from upstream sugar. Their baseline bytes are unchanged. Native raw IO executes
  twice in the matrix and matches its unchanged byte contract.
- Sixteen JS artifacts are byte-identical to baseline. Each of sixteen C
  artifacts differs only by the exact reviewed numeric-runtime formatting block.
- Final-source follow-up: 51 observations renew 17 native outcomes, 32 identical
  generated artifacts and two emission refusals. They do not relabel the main
  matrix's source.

The 17-source checking matrix is 15 both accepted and two acceptance differences:
strict-proof IO exclusion and the retained ASCII input-depth failure. The four
non-scalar readback differences are separate derived programs. No full checking
census rerun is claimed. Last measured census remains source
`c5edea8a75022c1654fbf568629422cf73bee1d0`: 140 non-IO residuals (109 acceptance,
31 rejection) after exact renewal of the historical 385-member IO cohort, plus
six oracle failures. No current totals are inferred from this formatter fix.

Coverage linkage becomes 516 / 1302, including failures, with 786
unlinked entries. This adds two original fixture links, not a universal parity
claim. Remaining categories are recorded in the validation receipt.

## Reproduction and provenance

Activate the existing tool environment; no installations or dependency changes:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off PYTHONDONTWRITEBYTECODE=1
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=4
cargo test --locked --features extended_observability --test cpu_c_escape_printer --test cpu_uppercase_unicode -- --nocapture
cargo test --locked --features extended_observability --test compiler_c --test compiler_executable_c_files --test compiler_executable_c --test compiler_open_closures -- --nocapture
cargo clippy --locked --features extended_observability --lib --test cpu_c_escape_printer --test cpu_uppercase_unicode -- -D warnings
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Host debug settings are separate from Bend's C profile. Both C lanes use
`-std=c11 -O3 -lpthread -lm`; port C also uses
`-Wall -Wextra -Werror -pedantic`, then runs with `--threads 1 --gpu off`.
JS runs through installed Node. Each matrix subprocess is bounded to 30 seconds.
Exact generation/run/readback commands and statuses are in the compact receipt,
with sanitized root variables and raw stream hashes. Invalid UTF-8 baseline
outputs remain lossless base64, not replacement-character text.

The probe portability follow-up adds Windows binary-mode setup under `_WIN32`
in the disposable test main, as normal programs already do. It neither invokes
runtime IO initialization nor changes Bend counters. The finding came from source
review, not desktop execution. Windows, GPU and newer upstream remain unrun.

Matrix binary SHA-256: `51bad45bbf6f02b3d856fb510b876c76ba28fdef82983810f0e1a3b4e96e2494`.
Final gate binary SHA-256: `8d37bc3b151067e3290828f6bddec2a3e245d526084eb3a8f0f3358af3b4d769`.
Main matrix SHA-256: `b0c77439af50c299de8c58bee4b02a74f88cd7adbeff368ebf426272f525362f`.
Compact results SHA-256: `27de7e0e06e0cae0c4f7c7b6ce1fc5088ee7496b86f765da8defc7db18866d73`.
Final gate log SHA-256: `bfc4f3117240e2b6940ce41365b583b068e09e081e662cae57368dda84ad5bbc`.
The receipts distinguish build/source identities instead of assuming binary
identity across the test-only metadata rebuild.

Earlier failed launch/harness/lint attempts remain recorded: the exclusive-name
runner refused a colliding output name without overwriting evidence; a probe
initially referenced a CLI symbol omitted under TB_NO_MAIN; test-only Clippy
findings were corrected by naming/extraction. No warnings were suppressed.

## Preservation and remaining work

Workspace floors remain 11 GiB before builds and 10 GiB during work; temporary
storage remains above 1 GiB. Final gate minima: 12.174 GiB workspace and
2.183 GiB temporary. Source pins, lockfile, prior cleanup records, both earlier
backups and blocked Library deliverables retain their verified hashes. No Library
retry, desktop action, main merge or release occurred.

Source and compact receipts are in Git. A fresh private evidence archive verifies
116 records, 770072 logical bytes, SHA-256 `7edf86d5faf0be0791e0bd414db6533e43035512a7c7d1b902b064c5ee339850`.
Generated artifacts/binary snapshots remain local and are not covered by the Git
push. The previously lost historical focused log remains unrecovered.

Native Char/String presentation, non-scalar literal readback and the input-depth
aggregate remain concrete gaps. Pending `nat_first`, unadopted compact Nat/string
lowering, Base coordinates, external invoke audit, supplementary lint failures,
F32/foreign/source-hole/resource and older oracle differences remain explicit.
No new semantic exception was adopted.
