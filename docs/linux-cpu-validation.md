# Linux CPU validation

Validated 2026-10-23 at source commit
`9c9e2f5ef833382a506a44a66f098d479cae4213`, descended from preserved commit
`09facb2d969efb16c4fc556f90dac2418b91d2d7`. The standard Linux gate passes,
and five bounded fixtures agree across five runtimes. This closes the first
CPU resumption milestone, not whole-engine parity or GPU qualification.

## Sources and tools

The untouched Bend reference is
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599` (2.0.5). Teamy conventions were
read from `teamy-docs` at `50e89d6897ce2a65167ef65b17c5fa83fe849158`.
Both reference repositories remained clean. The environment restored the
handoff, setup receipts, tool activation script and original failure logs.

The host runs Debian 13 on x86_64 Linux, with a four-core quota and 16 GiB
memory limit. Tools: Rust/Cargo/Clippy 1.96.0, rustfmt 1.9.0-nightly from
`nightly-2026-05-24`, Node 24.19.0, PowerShell 7.5.4 and Debian Clang 19.1.7.
The current memory limit differs from the setup receipt's historical 32 GiB.
`Cargo.lock` and dependency pins are unchanged.

## Root causes and repairs

The initial root-package diagnostic had 541 passes, 183 failures and 49 ignores.
Of the failures, 177 stopped at strict generated-C compilation, five required
the unbuilt Node provider, and one exposed the JavaScript environment-key
contract. These were shared blockers rather than 183 independent semantic bugs.

- Scope the Windows-only `eyre::Context` import and place the Unix
  `OpenOptionsExt` import at module scope, preserving the file mode.
- Cast socket2's initialized address-storage pointers to the libc socket
  address type at the three Unix FFI boundaries. Ownership and lengths stay intact.
- Mark individual optional runtime ABI entry points and paired generated layout
  converters with `TB_OPTIONAL`. Clang/GCC use a declaration-level unused
  attribute; MSVC and shared device code retain their existing behavior.
  Internal helpers and foreign C still receive strict diagnostics. A negative
  regression proves unused foreign functions and locals fail compilation and
  preserve the previous executable. No warning flag or global diagnostic pragma
  was relaxed.
- Reject NUL and `=` in generated-JavaScript environment names, matching native
  Rust, port C and upstream C. The existing Linux expected result is retained;
  the former Windows truncation expectation is intentionally replaced by the
  same rejection contract.
- Emit optional test-probe helpers only when used. Keep the controlled socket
  send's exact 32 KiB parked offset and complete 512 KiB payload assertion,
  with buffers larger than the controlled prefix and wake traffic excluded
  from its byte counter. Linux's smaller buffers previously parked at 4,032
  bytes before reaching the injected limit.

The first unblocked complete test sweep exposed those test-harness issues and
the standalone CUDA host adapter's need for its optional-entry macro. Focused
repairs passed before the final standard gate. CPU/GPU algorithms, proof rules,
resource budgets, raw program/serve output, CLI schemas and logging dependencies
were not migrated.

## Gate result

| Check | Final result |
| --- | --- |
| Pinned formatting check | Passed |
| Strict all-feature Clippy | Passed |
| All-feature workspace build | Passed |
| Real Node provider contract suite | Passed: TCP/UDP, 70 ordered registrations, truncation, handles, views and bounds |
| Standard workspace tests | 726 passed, 0 failed, 49 ignored across 81 result-bearing targets |
| Root-package share of tests | 725 passed, including five expected compile-fail doctests |
| Provider-crate share of tests | 1 passed |
| Five fixtures across five runtimes | 25 successful executions; identical stdout, empty stderr, exit 0 |

The 49 explicit ignores are 47 CUDA hardware tests and two optional profilers.
No ignored case ran. CPU-only CUDA cache and generated GPU-off paths are included
in the passing suite. The earlier Windows result, 728 passed and 49 ignored,
remains separate historical evidence; platform-gated test totals differ.

From the repository root, after activating the pinned environment:

```sh
export BEND_GPU=off CARGO_NET_OFFLINE=true
rustup run nightly-2026-05-24 cargo fmt --all -- --check
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo build --locked --features extended_observability
pwsh -NoProfile -File scripts/compare-cpu.ps1 \
  -Reference '<bend-reference>' -OutputDirectory target/cpu-comparison-001
```

Choose a fresh output directory. The comparison script requires the clean pinned
reference, copies the same fixture into each runtime's input, and records source
SHA256, port revision/dirty state, commands, working directories, status and raw
stdout/stderr bytes. Each generation, compilation and execution has a 60-second
deadline. Captures remain local because commands can contain machine paths.
Upstream generation runs from its `bend2` directory. The complete upstream
POSIX C runtime is compiled with `-std=c11 -pthread -lm -ldl`; port C additionally
uses unchanged `-Wall -Wextra -Werror -pedantic` flags.

## Differential baseline

The two recovered fixtures use identical console wrappers and remain covered by
the original generated-C regression tests. The other three are unchanged copies
of `tests/run/<name>.bend` from the pinned upstream source.

| Fixture | Stdout | Source SHA256 |
| --- | --- | --- |
| drop-chain | `16\n` | `d477b03578a5d1d93fd320a124df1694feac58980c7f35e489aed8d511558050` |
| nested-layout | `17\n` | `f739cea2f97bc825a6800398d2241200086e276fe163887d759495b8fbce4843` |
| word_arithmetic | `63\n` | `73db0427fca1c2c79db2c84141434b92f06b2bc96a84b0563a14ce37a00af9c5` |
| array_boxed_get | `777113\n` | `2f997297d8600b2785a1a05e04ba2013746020403e354daf6fff59f152d1071e` |
| closures_hof | `64180\n` | `995334f3b7925f6d12a19cc84d1353db05736d9ebde158accdf9c9621991f059` |

For each row, upstream C, upstream JavaScript, port C, port JavaScript and native
Rust agree byte-for-byte. These hashes also match the setup comparison inputs.

A separate synthetic environment-name probe deliberately differs: a NUL-suffixed
name returns `error:2\n` in upstream C and all three port runtimes, while upstream
JavaScript on Node 24.19.0 resolves the prefix and prints `synthetic-value\n`.
This is an explicit upstream backend inconsistency; matching the C contract is
not claimed as JavaScript parity for that malformed name.

## Remaining work

Fresh Windows/macOS validation and GPU hardware/sanitizer runs were not performed.
The exhaustive checker and Poche audits were not rerun. Existing unsupported
semantics and proof boundaries in [compatibility](compatibility.md) and the
[implementation plan](implementation-plan.md) remain active. In particular,
finite-layout CUDA qualification, allocation chains, demand backing, sizing,
residency and Metal remain separate work.

The next small CPU step is to extend this bounded differential runner with
unchanged upstream task/channel and file-effect fixtures, classify actual output
divergences, and select one reduced semantic failure. Keep the separate
Facet/logging investigation out of that runtime slice.
