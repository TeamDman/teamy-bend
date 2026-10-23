# Bounded CPU task and file comparison

Validated 2026-10-23 at runner commit
`652765d9f5bd0c0f047ed91aab56921f50976cfb`, on a new branch descended from
`23747901bad817dbf43089dd5341b03c4a54c03d`. The original Linux portability
branch stays at that parent commit for independent desktop verification.
This follow-on comparison is separate from the
[Linux portability milestone](linux-cpu-validation.md).

Six unchanged upstream fixtures were executed twice in each of five runtimes:
**54 successful comparisons, six upstream-JavaScript environment failures,
zero timeouts**. Every port execution matches upstream C. No reproducible port
semantic discrepancy was found, so no engine or proof-rule change was made.

## Selection and source identity

The first baseline covers words, closures, arrays and recovered layout/drop
regressions. Existing port tests cover many task and file contracts, but the
bounded runner previously did not execute unchanged upstream task/file programs.
These additions cover lifetime, transfer, ordering, buffering and file results
without a benchmark or a large corpus audit.

The reference remains clean at Bend 2.0.5 revision
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`; fixture bytes are copied unchanged
from its `tests/io` directory. Teamy output/CLI guidance was consulted at docs
revision `50e89d6897ce2a65167ef65b17c5fa83fe849158`, also left clean. Tool and host
versions are unchanged from the Linux receipt: Rust 1.96.0, pinned nightly
formatting, Node 24.19.0, PowerShell 7.5.4 and Clang 19.1.7, with GPU disabled.

| Fixture | Missing differential coverage | SHA256 |
| --- | --- | --- |
| spawn_outlives_main | A detached timer child finishes after main returns | `041007c7cdb78f5d0bf94962e08c6cd1dfed470f53eeafdb2d0f4b3dedd181ab` |
| channel_fork_join | Buffered transfer, close, fork and join | `c0a3a78d313d18686d981ad6a4b11be71390be49cef51623643aafd4a51e5600` |
| chan_rendezvous | Zero-capacity sender/receiver wake order | `72e50d8ba799e7eb6c16d186071186a3a80ac755c964361cba56495ee2a8c163` |
| chan_pipe | Bounded producer backpressure and draining after close | `a06c8009a30fb27dd3b13de145e308a0d8200699424f1bdff2430b0545c1095f` |
| file_roundtrip | Write/read/close plus a missing-file result | `91817af29d872d094e71b00317d9bf909d2cd5cfdfdb8758f6a6d820eb913bf8` |
| file_open_mode | Valid modes and rejection of Node's wider mode vocabulary | `7f5c20a098d06a09a75fe9a039f7bd3b063e57ec2e543a797b0449400fd1ea61` |

## Exact execution outcomes

Upstream C, port C, port JavaScript and native Rust produce every stdout below,
with empty stderr and exit 0, in both repetitions. Upstream JavaScript also
matches all three channel rows.

| Fixture | Complete stdout bytes, escaped |
| --- | --- |
| spawn_outlives_main | `main returns\nlate\n` |
| channel_fork_join | `1122\n` |
| chan_rendezvous | `m0\ns0\ns1\nm1\nm2\ns2\n` |
| chan_pipe | `4\n3\n2\n1\n0\nend\n` |
| file_roundtrip | `No such file or directory\nbend-file\n` |
| file_open_mode | `opened\nopened\nInvalid argument\nInvalid argument\n` |

The remaining three upstream-JavaScript runs fail identically in both repetitions:

| Fixture | Partial stdout | Exit |
| --- | --- | ---: |
| spawn_outlives_main | `main returns\n` | 1 |
| file_roundtrip | empty | 1 |
| file_open_mode | `opened\nopened\n` | 1 |

Their stderr is exactly the following, with the generated file's actual absolute
path in place of the placeholder and a trailing newline:

```text
Error: Cannot find module 'bun:ffi'
Require stack:
- <generated-upstream.js>
```

Pinned upstream `comp.ts` loads `bun:ffi` through `io_sys()` for timer polling
and `strerror`. Node cannot provide that Bun module. These are known failed
executions, not skipped tests or successful equivalence. No Bun installation,
replacement host bridge, or upstream source modification was introduced.
Qualifying those upstream-JS effects under their required host is separate work.

The file fixture payloads are also captured byte-for-byte: `bend-file` (nine
bytes, no newline) and an empty file. Those payloads were the same even in the
failed upstream-JS attempts; this does not turn their partial executions into
successful comparisons.

## Runner and validation

From the repository root, with the pinned tools activated:

```sh
export BEND_GPU=off CARGO_NET_OFFLINE=true
rustup run nightly-2026-05-24 cargo fmt --all -- --check
pwsh -NoProfile -File ./check-all.ps1 -FormatterToolchain nightly-2026-05-24
cargo build --locked --features extended_observability
pwsh -NoProfile -File scripts/compare-cpu.ps1 \
  -Reference '<bend-reference>' -OutputDirectory target/cpu-effects-001 \
  -Suite effects -Repetitions 2
```

`-Suite baseline` remains the default five-case comparison; `all` combines the
two sets. Repetitions are bounded to one through three. Every generation, compile
and execution keeps the existing 60-second deadline and strict port-C flags.
The receipt records exact commands, byte-encoded streams, status, elapsed time,
input hashes and clean source revision. Two observed repetitions agreed on all
streams, status and file bytes; this is bounded determinism evidence, not a proof
of all possible schedules. A single repetition reports determinism as unknown.

Upstream file fixtures use fixed `/tmp/bend_io_file.tmp` and
`/tmp/bend_io_mode.txt` names. The runner refuses pre-existing paths, executes
serially in separate working directories, captures each created file, then
removes it. Both fixed paths were absent after validation. Use fresh output
directories to retain previous receipts; raw captures stay local. Concurrent
effects-suite invocations must not share the same `/tmp` namespace.

The report keeps `matched=false` and lists `environmental_limits` for the three
affected fixtures. `completed_runtimes_match=true` records the successful subset.
Only the exact known exit, diagnostic and partial stdout are classified this way;
timeouts and other differences fail the command. Exit 0 means the comparison
report completed, not that every upstream runtime completed successfully.

Validation results:

- Standard gate: **726 passed, zero failed, 49 ignored**, with formatting,
  strict Clippy, all-feature build and real Node-provider contracts passing.
- Original baseline regression: **25/25 executions match**.
- Effects: **54/60 executions complete and match**, six exact upstream Node FFI
  failures, all 60 repeated observations stable, no timeout.
- Negative runner probes: pre-existing file contents remain intact; an altered
  partial stdout with the known FFI diagnostic is rejected. Both correctly exit 1.

All runtime sources, dependencies, stdout/stderr contracts and GPU behavior are
unchanged. The 47 CUDA hardware tests and two profilers remain ignored; this work
does not add fresh Windows/macOS, GPU, exhaustive checker or Poche qualification.
The next bounded CPU comparison can add close-wakeup and Halt/cancellation cases;
the separate upstream-JS host requirement should remain visible in its receipts.
