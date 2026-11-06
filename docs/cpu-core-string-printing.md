# Core String and Char presentation — 2026-11-06

The native pure runner and `eval` formerly printed constructor chains for
complete `String` and `Char` values, while the pinned Bend interpreter and
generated CPU backends print literals. For example, the existing prefix-marker
fixture printed `SCon{Chr{97}, ...}` natively instead of `"abcd"`.

The shared core-term formatter now recognizes complete `SCon`/`SNil` strings
and complete `Chr` values structurally, matching pinned `term_show` escape
rules. It preserves open or malformed constructor structure, including a
literal suffix following a nonliteral head. Quote, backslash, NUL, control,
Unicode scalar and nonscalar code points use the pinned spellings. A single
linear planning pass and iterative prefix printing avoid rescanning every
suffix of an open string; an explicit 4,096-element open-string test exercises
that path. This changes presentation in all callers of `Term` display,
including diagnostics and public consumers, but does not evaluate fields or
alter term graphs, checking, effects, guards, budgets or demand semantics.

The historical `*.native-stdout` and Unicode `port_normalized` values are
retained as evidence of the old presentation. Updated integration assertions
require the upstream expected text and independently compare the complete
normalized core graph, including against the historical constructor spelling.

## Evidence

On the production candidate based on `8e66e8d4d039374056faedc53a848db947eb7514`,
the pinned upstream is `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`.
Four pure programs—the prefix-marker String append, Unicode String, escaped
slash and unchanged upstream `show/string_escape.bend`—matched exactly across
the upstream interpreter, native port, upstream C/JS and port C/JS. Two runs
per lane yielded **48 matching executions**, all exit 0 and empty stderr, plus
16 successful generations and eight successful C builds. C used the pinned
O3 profile, one thread and GPU off; port C used strict warnings. A separate
20-case upstream `term_show` oracle matched the new focused test expectations,
including partial and malformed terms. The selected candidate binary SHA-256
was `93c74b4121971c43edd3e8a8c0aa069cc27c61053e1e852cf87edeeded3ca23d`.
The [compact matrix receipt](cpu-core-string-printing-evidence.json) records
source, binary, fixture and stream hashes and the separate stage statuses.

The focused rendering, Unicode, prefix and reusable-Nat targets passed
11 tests on the earlier candidate, zero failed and zero ignored. A separate
normalization, text, shared-input and rendering run passed 46 tests on that
candidate; three rendering tests overlap the first run. Targeted all-feature
Clippy passed on the final candidate with warnings denied. A preliminary build failed on a Rust binding and was
corrected; a preliminary adjacent run exposed an old Unicode test expectation
and was corrected while retaining its graph assertion. Those failures are not
counted as passes. The full standard-gate outcome is recorded below.

The first standard-gate attempt passed formatting, strict Clippy, the
all-feature build and real Node TCP/UDP provider checks, but failed in the
`cpu_c_escape_printer` group because that test still expected the old native
constructor spelling. Earlier completed groups had 824 passed, zero failed
and 48 ignored; the failing group had two passed and one failed, and later
groups were unrun. The test was updated to require the pinned literal spelling
while retaining its historical graph comparison, and all three tests in that
target subsequently passed. The final-source gate is recorded separately.

The final-source standard gate passed formatting, strict production Clippy,
the all-feature build and the real Node TCP/UDP provider check, then reached
its 1,200-second deadline during tests. Its 136 completed groups contained
952 passed, zero failed and 48 ignored; the active group was interrupted and
later groups were unrun. This is not a gate pass. The complete final-source
offline test set passed separately in bounded batches: 1,382 passed, zero
failed and 49 ignored, including all 200 integration targets plus library,
binary and documentation tests.

The original two-character string-pattern witness remains a separate issue:
twice, upstream returned `True{}` and the port refused at its pattern compiler
guard. This rendering change does not make it execute. The depth limits are
implementation constraints to retire through stack-safe, work- and
memory-bounded paths, not a claim about upstream language semantics.

Individual comparison processes had 30-second deadlines. The outer runner
uses a 1,200-second deadline, at least 11 GiB workspace free at launch, 10 GiB
while active and 1 GiB temporary-storage headroom; no additional OS memory cap
was imposed. GPU and non-Linux hosts were not tested. The unchanged dependencies,
upstream pin and toolchains are recorded in [the progress report](cpu-progress-20261106.md).
