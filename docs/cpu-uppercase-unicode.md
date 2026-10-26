# Uppercase Unicode escape CPU checkpoint

Tested source: `eedb452e718b10ce8b22a6e5fb42962706aa557b`. Parent checkpoint
`44122522ff38bc98c1ac7d3a3bb4bf7fda704b30` remains reachable and remotely unchanged.
Upstream remains `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.
Valid uppercase Unicode scalar escapes now work in the qualified CPU lanes.
This is bounded compatibility work; CPU parity is incomplete. See the
[validation receipt](cpu-uppercase-unicode-validation.json),
[compact exact outcomes](cpu-uppercase-unicode-results.json.gz) and
[fixture controls](../tests/fixtures/cpu-uppercase-unicode).

## Correction and contract

Pinned [parse_char](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L1588)
uses a case-insensitive regex in an eleven-UTF16-unit window: `u`/`U`, `{`,
one to eight ASCII hex digits, then `}`. The port previously rejected every
uppercase spelling. It now accepts that grammar when the value is a Unicode
scalar, through its existing literal decoder. Cloned-iterator lookahead is
bounded, allocation-free and non-consuming on refusal. Eight shifts cannot
exceed U32. The existing lowercase arm and unsupported-uppercase error path
remain unchanged. No dependency, toolchain, evaluator, budget, guard,
cancellation boundary or public API signature changes.

The earlier authored `"\U{41}"` reproducer now yields the value `"A"`.
Thirteen pure uppercase controls become checking/normalization successes; an
explicit IO control gains native execution while remaining excluded from strict
proof checking. The pinned inventory contains no uppercase-escape original;
the two copied originals are unchanged `show/string_escape.bend` and
`parse/invalid_string_escape.bend`, with Apache attribution. Other controls use
MPL-2.0.

## Validation

- Standard gate: **1310 passed / 0 failed / 49 ignored**,
  193 test-result groups, exit 0, 957.453 seconds.
- Focused: 12 literal tests, two integration tests and targeted strict Clippy
  passed. Added coverage includes valid scalar/digit boundaries, NUL, quotes,
  backslash, astral values, U+10FFFF, 128/129 decoded characters, expression
  depth, malformed/overflow/non-scalar forms, public error fields/Display/Debug,
  tracked load metadata, first-error order and existing cancellation boundaries.
- Differential: **514 observations / 235 deterministic pairs**, 44 separate
  emission/build observations, zero unexpected results or timeouts. This count
  includes retained differences; it is not 514 parity passes.
- Positive compiled evidence: 16 port executions against 16 pinned oracle
  executions. **14 port executions match exactly; two retain the C printer
  difference below.** Actual C uses the pinned O3 profile; port C also passes
  strict warning flags. Two native IO executions match. Sixty pure-normalization
  observations (30 port, 30 upstream) remain separate from execution.
- Tests compare complete normal-form structures for all 15 pure positive
  fixtures. Four fixtures retain native constructor text distinct from upstream
  sugar; eleven fixtures match text. No graph fields are omitted.
- The unchanged original's port C and JS generated artifact hashes match their
  baseline artifacts. Invalid-scalar cases have eight upstream C successes,
  eight upstream JS refusals, and eight port emission refusals with no artifacts;
  port execution for those cases is unrun because loading fails.

The 36-case checking matrix measures **15 both accepted (status), four exact
rejections, nine different rejections and eight acceptance differences**. The
last eight are one strict-IO exclusion, four uppercase non-scalar differences
and three retained lowercase differences. These categories are observations,
not new semantic exceptions. Negative malformed uppercase diagnostics remain
non-exact even where both parsers reject. The valid-U-then-invalid-escape control
now reports the actual later invalid escape, matching pinned text.

Initial integration attempts exposed incorrect test assumptions about native
String presentation and the original C printer; those failed receipts remain
retained. An authored aggregate initially used `List(U32)` instead of the pinned
`List<U32>` family syntax; that control was corrected. No production behavior
was changed to accommodate these exploratory failures.

## Reproduction and identity

Activate the existing environment, then use the frozen host build overrides:

```sh
export CARGO_NET_OFFLINE=true BEND_GPU=off PYTHONDONTWRITEBYTECODE=1
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=4
cargo test --locked --features extended_observability --lib literal_ -- --nocapture
cargo test --locked --features extended_observability --test cpu_uppercase_unicode -- --nocapture
cargo clippy --locked --features extended_observability --lib --test cpu_uppercase_unicode -- -D warnings
cargo build --locked --features extended_observability
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

These debug/incremental settings concern the Rust host build, not Bend's C
optimization. Rust 1.96.0, formatter nightly-2026-05-24, Node 24.19.0, configured
Clang 19.1.7 and PowerShell 7.5.4 remain installed and unchanged. Lock SHA-256:
`d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4`.

For each representative fixture, compare the original commands below; `INPUT`,
`UPSTREAM`, `PORT` and fresh artifact paths are explicit caller parameters:

```sh
node "$UPSTREAM/bend2/main.ts" "$INPUT"
"$PORT" run "$INPUT"
node "$UPSTREAM/bend2/main.ts" "$INPUT" -o upstream.c
node "$UPSTREAM/bend2/main.ts" "$INPUT" -o upstream.js
"$PORT" compile "$INPUT" --executable --target c --output port.c
"$PORT" compile "$INPUT" --executable --target javascript --output port.js
"$TEAMY_BEND_CC" -std=c11 -O3 upstream.c -lpthread -lm -o upstream.bin
"$TEAMY_BEND_CC" -std=c11 -O3 -Wall -Wextra -Werror -pedantic port.c -lpthread -lm -o port.bin
./upstream.bin --threads 1 --gpu off
./port.bin --threads 1 --gpu off
node upstream.js
node port.js
```

The committed compact receipt includes all expanded commands using sanitized
root variables, source hashes, exact streams/statuses, timings and per-command
30-second bounds. Raw scripts/logs and build snapshots are preserved separately.
The existing census phase adapter probes strict and executable checking; these
phase observations do not establish diagnostic or runtime equality.

Matrix binary SHA-256: `ff21a3aa9bc8456a859040586353cb77edc1c216bf8a72e36c10e2ee90826925`.
Gate binary SHA-256: `ff5617a2ee9fe41f5fb05ae2c9bbbf73bebc2315edf6a03ebec21162a025ec52`.
The gate rebuilt the same clean source/features with a new embedded build
timestamp. Exact byte comparison finds only the ELF build ID and three timestamp
digits different (23 bytes total); all other executable bytes match. Record
both identities. The retained phase probe is `fb48912dfc17170a3aa97ce2863ecc96ea6d4d0e3e8876953f411d4faeaf6646`; its historical rlib
build-input hash is not the current rebuilt rlib hash.

Raw matrix SHA-256: `bdf4c986d199cfcdcb8ae6125daf29ed5808caed295ad51b5a14eb270b6c9737`.
Compact results SHA-256: `fe3f9d5d75d46d70a32aacb256a970d9e1b2a8e1d473d61f26cf30afdd047a2e`.
Full gate log SHA-256: `df3a1e34992a5691d89417a7a9eea1225d7d8f07cf2e8c646b6cae2a81bab76b`.

## Remaining work and preservation

Non-scalar character literals need a separate representation/runtime decision:
upstream parsing and C accept U32 codes, upstream JavaScript rejects them, and
the port's literal decoder still uses Rust `char`. No wrapping, replacement
characters or new exception was adopted. Lowercase plus-prefixed and overlong
leading-zero escapes retain their existing disagreement.

The next concrete independent gap is the **C character/string printer**:
`show/string_escape.bend` prints `\u0001` and raw DEL in port C, versus
`\u{1}` and `\u{7f}` upstream and in port JavaScript. This behavior already
exists in the preserved baseline. Its round-trip implications require a scoped
printer correction, not a change to this uppercase parser fix. Native pure
String constructor presentation also remains explicitly distinct.

No full checking-census rerun is claimed. The last measured census is still
source `c5edea8a75022c1654fbf568629422cf73bee1d0`: 405 exact rejections,
366 both accepted, 494 acceptance differences, 31 different rejections and six
oracle failures. The exact 385-member historical IO renewal leaves 140 measured
non-IO residuals (109 acceptance + 31 rejection); this batch does not infer new
counts. The newly linked original brings coverage to 514 linked / 788 unlinked:
256 checker negatives, 519 interpretation/compiled candidates, 12 interpreter-only
and one no-main checking module remain unlinked. Linked receipts can contain
failures and do not certify parity.

Pending `nat_first`, unadopted compact Nat/string lowering, Base coordinates,
external invoke-caller audit, supplementary lint failures, F32/foreign/resource/
source-hole and oracle limits remain open. No desktop/GPU work, installation,
new upstream, main merge, release or Library retry occurred.

The 11 GiB preflight / 10 GiB active workspace / 1 GiB temporary floors remain.
Gate minimum free space: 12.175 GiB workspace / 2.257 GiB temporary.
All 21 cleanup restoration records, the prior evidence backup and both blocked
Library attachments retain verified hashes. The previously lost historical
focused log remains unrecovered; its known limitation is unchanged. Every new
run uses exclusive evidence paths and leaves prior raw files intact.

Source and compact sanitized receipts are preserved in Git. A fresh private
lossless evidence backup verifies 116 files (502786 bytes,
SHA-256 `ae27d132035d83d28fb08debf0982053c82b281ba1cd426e36808b4da712f01c`); raw generated binaries/artifacts remain local and are
not covered by the Git push. Reviewed feature publication does not merge or
release anything.
