# Native `F32.read` NaN payload overflow — 2026-11-06

Source `c3a775c65b5f1dea5fd3536e260a4e2b1706fbee` fixes a native Rust
`F32.read` bit mismatch observed on Linux x86-64 with glibc 2.41. The pinned
[Bend 2.0.5 C implementation](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L492)
passes the text to `strtof`. On this host, a fully numeric payload above the
unsigned 64-bit maximum saturates before its low payload bits are used. The
previous native parser instead returned a zero payload for integer overflow.

The correction is explicitly limited to Linux x86-64 GNU targets. It retains the
existing behavior for other target configurations. It verifies every digit in
the selected decimal, hexadecimal or octal radix before saturating: an invalid
character after overflow remains a nonnumeric payload rather than being accepted
as a number. This adds a bounded linear scan on the overflowing-payload path;
the existing 8 MiB numeric-text ceiling and runtime budgets are unchanged.
No mid-operation cancellation or other-host CRT compatibility is implied.

The [authored witness](../tests/fixtures/cpu-float-text/nan-payload-overflow.bend)
prints `F32.bits` for the largest in-range decimal payload, an overflowing
decimal payload, a negative overflowing hexadecimal payload and a long decimal
payload with a nonnumeric suffix. Before the fix, native returned
`2147483647`, `2143289344`, `4290772992`, `2143289344`; upstream C and port C
returned `2147483647`, `2147483647`, `4294967295`, `2143289344`. The corrected
native result matches those C lanes. Unit regressions also cover octal overflow,
near-boundary values, leading zeroes and invalid trailing digits in all three
radices.

A deterministic host-level sweep compared the native text parser directly with
glibc `strtof` on 828 decimal, hexadecimal and octal payload strings, including
invalid suffixes. The earlier source differed on 128 fully numeric overflows;
the candidate differed on none. These are direct host-parser observations, not
828 Bend runtime executions or a whole-CRT compatibility claim. The same input
list SHA-256 was `4d3aa5dd9fbbafb0474d9e5faf70b987675a47579759909592157179cf1afb99`
for both runs.

Upstream JavaScript and port JavaScript both return `None` for all four
payload spellings. This is an observed cross-backend difference, not a waived
exception or a claim that all `F32.read` inputs have uniform semantics. Other
historical decimal-rounding, hexadecimal and embedded-NUL C/JS differences in
[cpu-float-text.md](cpu-float-text.md) remain unwaived. Other libc versions,
other operating systems and architectures, GPU execution, arbitrary NaN payloads
and the full numerical API remain unqualified.

The unchanged string-pattern `"hi"` witness was rechecked twice: upstream
interpretation returned `True{}` and port checking refused at the 64-level
pattern guard. Port generated runtime artifacts were not run in that recheck.
The preserved prototype fails whole-function normalization at the separate
128-level guard; this slice does not adopt it or weaken either limit. See
[compact-pattern research](cpu-compact-patterns.md).

## Validation

The clean-source five-lane matrix completed 16 command observations, including
ten repeated executions, without timeouts. Native and the two C lanes matched
the C contract; the two JavaScript lanes matched each other and retained the
separate `None` results. The four generated C/JS artifacts were byte-identical
to the before-fix comparison. The [comparison runner](../scripts/compare-float-nan-overflow.py)
requires the clean pinned reference, a clean port, a selected binary, a fresh
output directory and the Linux x86-64 glibc profile.

The complete offline test set passed in bounded batches at the clean source:
**1,371 passed / 0 failed / 49 ignored**, covering all 197 integration targets,
library, binary and documentation tests. Formatting, strict production Clippy,
all-feature build, the Node TCP/UDP provider and targeted strict Clippy passed.
The uninterrupted standard gate timed out at its unchanged 1,200-second bound
after 936 completed passes, zero failures and 48 ignores; its interrupted group
and later groups were not counted in that attempt. The split run does not make
the single-command gate a pass. See the [validation receipt](cpu-float-nan-overflow-validation.json)
for source, binary and fixture hashes, limits, observations and unrun coverage.
