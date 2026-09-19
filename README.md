# teamy-bend

A Rust rewrite of the Bend 2 proof language, started from
[teamy-rust-cli](https://github.com/TeamDman/teamy-rust-cli).

**Status: working proof-language subset; full rewrite in progress.** Native
checking, pure evaluation, batch conformance and JavaScript generation work.
The bundled Base contains 114 selected pure upstream declarations. C, GPU,
effects, templates and complete library compatibility remain unfinished.
This is an independent project, not an official Bend release. The full rewrite
and Poche integration remain tracked in the
[implementation plan](docs/implementation-plan.md).

## Commands

```powershell
cargo run -- check examples/laws.bend
cargo run -- check examples/induction.bend
cargo run -- eval examples/laws.bend --entry main
cargo run -- --output-format json batch examples/laws.bend --entry identity --args-json examples/arguments.json
cargo run -- compile examples/induction.bend --output target/induction.cjs
node target/induction.cjs
```

`check` requires definitions and complete proofs for every law in the loaded
program. Unsupported syntax, unsafe definitions, holes, unfilled laws and failed
proof checks return errors. `eval` checks the program before normalizing an
entry point. Successful checking is relative to the implemented kernel; the
rewrite has not itself been formally proved sound. The induction example proves
`Nat.add(n, 0n) == n` for arbitrary `n` using structural induction and equality
rewriting, rather than enumerating a finite set of naturals.

`compile` emits standalone JavaScript for a closed data entry (default `main`).
Generated programs print a JSON `value` with recursive `constructor`/`fields`
objects; erased proofs/types use an explicit `erased` marker. Functions work
internally, but function-valued final results are unsupported. Existing output
files require `--force`. Node.js is required to run generated programs and
compiler integration tests, or configure `TEAMY_BEND_NODE` for the tests.

`batch` checks once and applies a function to each row of a JSON array of arrays
of natural numbers. It returns `{"results":[...]}`. This interface requires
the conventional `Nat` datatype with `Zero{}` and `Succ{pred: Nat}` constructors.
The transport ceiling is 4096 per unary natural, and argument files may be at
most 32 MiB. The current kernel's smaller nesting limits also apply, so values
below the transport ceiling can still receive a resource-limit error.
The command rejects negative/fractional inputs, wrong argument types and
non-natural results. This supports independent finite-domain conformance tests
without implementing domain rules in the host application.

Output defaults to JSON when redirected and text in an interactive terminal.
Use `--output-format json`, `text` or `csv` explicitly. Nested reports may not
have a CSV representation. Logs go to stderr; optional `--log-file` enables
NDJSON logs. `TEAMY_BEND_HOME_DIR` and `TEAMY_BEND_CACHE_DIR` override the
application directories. `RUST_LOG` or `--log-filter` configures diagnostics.

See [compatibility and resource limits](docs/compatibility.md) for the supported
surface and remaining work. Unsupported features return errors.

## Development

```powershell
./check-all.ps1
cargo run -- --help
cargo run -- --version
```

The quality gate formats the source, checks Clippy, builds all features and runs
tests without the Tracy transport. Profiling is opt-in:

```powershell
./run-profiler.ps1 check examples/laws.bend
```

This uses the shared `teamy-profiler` harness. The `extended_observability` and
`tracy` features are disabled by default. Human logs, NDJSON logs and Tracy have
independent filters.

## Licensing and provenance

New project code and the CLI scaffold use [MPL-2.0](LICENSE). The language port
derives from Bend revision `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`;
translated files retain [Apache-2.0](licenses/Apache-2.0.txt) licensing and
upstream attribution. See [NOTICE](NOTICE) for source provenance.
