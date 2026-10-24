# Checking-only list reverse and sort qualification

Source `c900e3fb1766733cb7033a8b779f3b0fa10122fb` qualifies the unchanged
[`eval/list_reverse_sort.bend`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/eval/list_reverse_sort.bend).
No engine source, dependency, toolchain, safety budget or demand policy changed.
The changes add an attributed fixture, two regression tests and `--entry NAME`
to the existing structural comparison example. No push occurred.

## Applicable observations

This file has no `main`. The pinned gate selects checking, and upstream
[`book_run`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L454)
reports successful checking when no entry is present. Running that CLI path does
not execute a Bend program. Neither native program execution nor generated C/JS
lanes are required or claimed for this fixture. We did not add a synthetic main.

Upstream CLI and the port compatibility command both return `All terms check.`
with final LF, empty stderr and exit 0, twice. The separate port strict `check`
command also exits 0 with empty stderr and a typed `checked: true` report. Its
counts include the port's Base; they are not a cross-implementation library count
comparison. Every original definition and proof is checked before normalization.

The three proofs concern concrete lists: reverse `[1, 2, 3]`, reverse it twice,
and insertion-sort `[3, 1, 2, 3]` while retaining its duplicate. This is not a
general theorem that every list is reversed or sorted correctly.

| Named definition | Normal form, both implementations |
| --- | --- |
| `l123` | `[1, 2, 3]` |
| `l321` | `[3, 2, 1]` |
| `rev`, `rev2`, `srt` | `{==}` |

All five have exact text and independently compared actual core, twice. The
oracle adapter uses the unmodified pinned `book_load`, `book_valid`,
[`term_snf`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.ts#L3033),
`term_lower` and `term_show`, and rejects incomplete books or a present main.
The port probe strictly checks and normalizes the selected definition directly;
its structural vocabulary is unchanged. A reflexivity token's matching spelling
alone does not establish a valid proof; full checking and false-proof controls
provide the relevant additional evidence.

Two authored controls each change one expected list. One gives reverse the wrong
order; the other removes the expected sort duplicate. Both engines reject both
at the intended `rev` or `srt` equality, with exit 1 and empty stdout, twice.
Their raw diagnostics differ: upstream shows normalized expected/observed lists
and the Bend source location; the port shows the reflexivity constraint and host
diagnostic context. Those differences remain unresolved diagnostic observations,
not exact negative parity or a newly accepted policy exception. The controls add
no upstream inventory rows. Captures retain all streams, statuses and locations.

## Validation and provenance

| Check | Final result |
| --- | --- |
| Original checking | Exact compatibility stdout/stderr/status twice; separate strict check succeeds twice |
| Five named definitions | Ten upstream normal forms, ten port strict eval reports and ten actual-core comparisons; all exact |
| Two authored false-proof controls | Twelve expected rejections, same proof/phase; raw diagnostics differ |
| Full normalization | 32 fixtures: 250 successes, 68 printer refusals, six F32 exclusions; runner exit 0 |
| Focused checker/collection/CLI tests | **58 passed, 0 failed, 0 ignored** |
| Standard gate | **825 passed, 0 failed, 49 ignored**; exit 0 |
| Strict all-target Clippy and build | Both exit 0 |
| Remaining runtime/demand/comptime/read/audit matrices | Not rerun; historical receipt identities preserved |

The 49 ignores remain 47 CUDA checks and two optional profilers.

All comparison children exited; no timeout occurred. Repeated streams/statuses
were stable. The checking-only comparison contains 48 commands: 36 successful
checking/definition observations and twelve expected false-proof rejections.
The full normalization suite retains its strict F32 proof exclusions and separate
executable-output/core-regression mode. No proof contract was broadened.

The initial focused run passed. Strict Clippy then caught an assertion-style
issue in the new missing-main check; the final source uses an explicit expected
error. The first private comparison harness incorrectly expected an error keyword
that upstream does not print. Its captured rejection was correct; the final
harness verifies the actual expected/observed and proof-location fields. Both
candidate receipts remain intact. These were test/harness corrections, not engine
defects or changed expected acceptance.

The original bytes have SHA256
`a38a348ae9f8d1e11f7b4514a5656ae22fff2d7c06c7713e126c30ebee08cd3f` and retain
[Apache-2.0 attribution](../tests/fixtures/cpu-checking/README.md). Frozen upstream
and docs checkouts remain clean; Cargo.lock is unchanged. Receipt hashes below
bind the final source/binaries and exact commands. Raw evidence remains private.

After activating the existing tools with `CARGO_NET_OFFLINE=true` and
`BEND_GPU=off`, the final commands are:

```sh
cargo test --features extended_observability --test kernel_normalization --test kernel_collections --test kernel_core --test cli_proofs --quiet
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features --bin teamy-bend --example compare_normalization
node <clean-pin>/bend2/main.ts <unchanged-fixture>
<port-binary> run <unchanged-fixture>
<port-binary> --output-format json check <unchanged-fixture>
<port-binary> --output-format json eval <unchanged-fixture> --entry l123
<structural-probe> <unchanged-fixture> <oracle-stdout> --entry l123
pwsh -NoProfile -File scripts/compare-normalization.ps1 -Reference <clean-pin> -OutputDirectory <fresh-evidence> -Suite all -Repetitions 2 -RequireExactText
pwsh -NoProfile -File check-all.ps1 -FormatterToolchain nightly-2026-05-24
```

Repeat named observations for `l321`, `rev`, `rev2` and `srt`. The saved bounded
adapter performs the following operations on the unchanged source (using Node's
existing TypeScript support):

```js
const Bend = await import(pathToFileURL(modulePath));
const book = Bend.book_nil();
await Bend.book_load(book, source, "", new Map());
Bend.book_valid(book);
if (book.hols + book.open !== 0 || book.tlds.main !== undefined) throw Error("scope");
const normal = Bend.term_snf(book, book.tlds[name].v);
process.stdout.write(Bend.term_show(Bend.term_lower(normal)) + "\n");
```

The full saved adapter also verifies the named definition is filled; no upstream
file is edited. Each child has a 30-second bound in this comparison, and the
existing normalization runner retains its 60-second bound.

## Build headroom and remaining work

Initial workspace headroom was **5.28 GiB**. Bounded cleanup
of inactive rebuildable test executables restored **14.70 GiB**.
The gate started with **13.34 GiB** and kept at least
**4.83 GiB** workspace / **4.84 GiB** temporary-space
headroom over 154.68 seconds. No disk/timeout stop triggered.
After verified immutable snapshot deduplication, workspace headroom was
**5.32 GiB**. Four duplicate CLI/probe copies now share their
respective immutable bytes, retaining all paths/hashes and saving
0.49 GiB. No mutable target output was hard-linked.

The [previous disk investigation](cpu-nat-literal-roundtrip.md#disk-cause-and-prevention)
still applies: large retained test executables, dependency-fingerprint variants
and Git-metadata-triggered relinking cause growth. This checkpoint reused the
bounded cache helper, protected CLI/probes and all source/evidence, and required
12 GiB before broad builds. The standard gate monitored two-second samples,
2 GiB workspace and 1 GiB temporary-space floors, and a 1,200-second deadline.
No source/evidence deletion or Bend budget change was used.

| Receipt / input | SHA256 |
| --- | --- |
| comparison | `24d6e61987814d5fe6e5ee9dfd558f0985800c4513214b5ae070e8a41ca24b14` |
| normalization | `ecf038e5fdd598e24892a366f0d17f340c8c5bbc00daf93c39d798f6c2b6ab37` |
| gate-monitor | `837d1c21108920375ed3580a152d8bd47057c5286ff69fec4922a0a254feab4d` |
| cache-before-tests | `3586c664ad7d70ed93aa658f440bf28d6e83095058dcf2f35b4ad6e221f3c1f2` |
| cache-before-build | `91ba735764c869d8f0429dda0e85b8d1b0153b9470c3131a853bccbec8b638b7` |
| cache-before-gate | `322b0803b0a7b83ca0f30049d1323bad69dd4afcd6ae96d14ed6eb9247bc7ce2` |
| snapshot-dedup | `af1569208e3d7465da8cf41a27c19563f1a7a520e3f3bb999524423cb6e42792` |
| oracle-adapter | `3c1f262b95ef84c08ddc1255207b9c78c8b4d90ab07d58c9c8e43037511cb9ac` |
| comparison-runner | `730432a94f6e3a93224d05bc5e879a61657000f4a7d9b40e71a87e9775ea530c` |
| Cargo.lock | `d2ef7a9647a04bafdb30afa09f204403fff983abe2b70b3c0a21617f53302bc4` |
| Port binary | `8ab4a96161c2c0d9f5d22957700168f395a86240d8128113dca554825dc24374` |
| Structural probe | `e8766fdf9a268090786cce5da2b7cf90e614a51f1901e8ec7f84b75c99930f96` |

The checking ledger now has five originals: two exact positive checking receipts
and three older phase/construct agreements with differing raw diagnostics. The
new original retains its checking-only category. It is not added to the runtime
or main-interpretation ledger just because named values were inspected.
The 28 existing main-interpretation rows were renewed here; all runtime/demand,
broader audit and other checking rows retain their actual historical identities.
Their earlier pass counts do not become new runs at this source.

The runtime/interpretation union remains 103 originals, with 1,199 lacking those
receipt modes. The twelve no-main entries in that number include two now covered
by explicit positive checking receipts; they are not missing runtime tests.
The acceptance criterion remains unsatisfied. F32.read, competing-error order,
resource-policy, native foreign, unavailable oracle-host, API and platform gaps
remain open. GPU and fresh Windows/macOS runs were not performed.

Next independent family: `parse/chain_default_nat_f` and `_g`, whose original
goldens normalize Nat arithmetic precedence and comparison functions. They still
need actual upstream printer-applicability verification and strict actual-core
comparison. Their sources were inspected here; qualification was not run.
