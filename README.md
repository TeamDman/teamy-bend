# teamy-bend

A Rust rewrite of the Bend 2 proof language, started from
[teamy-rust-cli](https://github.com/TeamDman/teamy-rust-cli).

Status: working proof-language subset; full rewrite in progress. Native
checking, pure evaluation, persistent typed calls and JavaScript/C generation work.
Native console IO, tasks, timers, channels, environment lookup, files and TCP/UDP
use a separate execution contract checker.
Generated JavaScript supports foreign calls, callbacks, cooperative tasks, timers
and channels with fork/join, plus environment, file and TCP/UDP contracts.
Executable C supports native values, foreign callbacks and console/task/timer/
channel/environment/file effects. All 37 numeric primitives are implemented in native
IO and both executable compilers;
their contracts remain opaque to strict proof checking.
Closed compile-time templates and their specialized instances are supported.
The bundled Base contains 367 selected pure source declarations, including
18 templates, Image quadtrees and Event values. Finite App playback runs through
native IO and generated JavaScript. Window/audio effects,
full C runtime parity, GPU and complete library compatibility remain unfinished.
This is an independent project, not an official Bend release. The full rewrite
and Poche integration remain tracked in the
[implementation plan](docs/implementation-plan.md).

## Commands

```powershell
cargo run -- check examples/laws.bend
cargo run -- check examples/induction.bend
cargo run -- base List
cargo run -- base --types
cargo run -- eval examples/laws.bend --entry main
cargo run -- run examples/console.bend
cargo run -- run examples/tasks.bend
cargo run -- run examples/channels.bend
cargo run -- --output-format json batch examples/laws.bend --entry identity --args-json examples/arguments.json
cargo run -- serve examples/laws.bend
cargo run -- compile examples/induction.bend --output target/induction.cjs
node target/induction.cjs
cargo run -- compile examples/induction.bend --target c --output target/induction.c
cargo run -- compile --executable examples/console.bend --output target/console.cjs
node target/console.cjs
cargo run -- compile --executable examples/console.bend --target c --output target/console.c
cargo run -- compile --executable examples/tasks.bend --output target/tasks.cjs
node target/tasks.cjs
cargo run -- compile --executable examples/channels.bend --output target/channels.cjs
node target/channels.cjs
cargo run -- run examples/app-playback.bend
```

`check` requires complete proofs for every ordinary law and checks ordinary
definitions and instantiated templates. Template bodies are parsed at their
declaration and type-checked when specialized with closed `~` arguments.
Unsupported syntax, unsafe executable definitions, holes in checked terms,
unfilled laws and failed proof checks return errors. `eval` checks the program
before normalizing an entry point. Successful checking is relative to the implemented kernel; the
rewrite has not itself been formally proved sound. The induction example proves
`Nat.add(n, 0n) == n` for arbitrary `n` using structural induction and equality
rewriting, rather than enumerating a finite set of naturals.

`compile` emits standalone JavaScript for a closed data entry (default `main`).
Generated programs print a JSON `value` with recursive `constructor`/`fields`
objects; erased proofs/types use an explicit `erased` marker. Functions work
internally, but function-valued final results are unsupported. Existing output
files require `--force`. Node.js is required to run generated programs and
compiler integration tests, or configure `TEAMY_BEND_NODE` for the tests.

`compile --executable` emits a standalone Node.js program from executable
contracts. IO entries preserve raw console output and exit status; printable
pure entries use Bend text. Synchronous foreign JavaScript uses native values,
curried callbacks and a shared scope for imported sources. Spawned tasks remain
live after main completes; timers and saved continuations use a FIFO scheduler.
Compilation embeds
foreign source; running the generated program executes it with Node's host
permissions. Channel values preserve the reference foreign interface; fork/join
helpers use the same scheduler. TCP/UDP uses the synchronous Rust Node-API
provider built alongside the CLI and packaged beside generated programs. See
[JavaScript networking](docs/executable-network.md) for provider setup, raw
descriptors and limits. [Executable C](docs/executable-c.md) now has a packed
native ABI, foreign callbacks and a bounded CPU effect runtime; its remaining
host effects, reclamation and optimized execution are unfinished.
Environment and file effects use synchronous Node calls. The sealed affine File
type prevents source code from copying handles; reads and writes return the
handle on both success and failure. See
[environment and file effects](docs/native-files.md) for target differences.
See [executable JavaScript](docs/executable-javascript.md).

`App.play` feeds finite lists of events through an App's tick callback and
returns its final state, stopping when tick returns None or halts. It supports
affine state and does not call the view callback or open a window. Image values
and their ordinary drop helpers do not establish GPU execution or parallel
memory reclamation. Interactive window-backed App helpers remain unfinished.

`compile --target c` emits portable C11 with the same output format, lazy
closures and checked proof erasure. Build the emitted file with a C11 compiler.
This baseline uses immutable instruction tables and a bounded runtime; upstream
C optimization and foreign code support remain open. Runtime failures return
an error without a partial JSON result. C integration tests require a compiler
on `PATH` or `TEAMY_BEND_CC`; they can discover Visual Studio's C tools on Windows.

`batch` checks once and applies a function to each row of a JSON array of arrays
of natural numbers. It returns `{"results":[...]}`. This interface requires
the conventional `Nat` datatype with `Zero{}` and `Succ{pred: Nat}` constructors.
The transport ceiling is 4096 per unary natural, and argument files may be at
most 32 MiB. The current kernel's smaller nesting limits also apply, so values
below the transport ceiling can still receive a resource-limit error.
The command rejects negative/fractional inputs, wrong argument types and
non-natural results. This supports independent finite-domain conformance tests
without implementing domain rules in the host application.

`base` prints the exact bundled library source. `base List` selects that name
and its subnames; `base --types` selects datatype declarations and kind laws.
It always emits source text, including when output is redirected.

`run` checks executable contracts and runs `main`. Its execution-only Base adds
IO continuations, pure/bind/die/pass/try, console effects, tasks, timers,
channels, IO.get_env, File.open/read/read_bytes/write/close and the eleven
TCP/UDP/socket effects.
Console output stays raw even with `--output-format json`. `Emit` discards its
payload and exits successfully; `Halt` writes its message to stderr and sets
the exit status. The console example prints `The answer is 42` and exits 0.
Foreign signatures are runtime assumptions, separate from strict proof evidence.
Valid native file open/read/write requests suspend through bounded host workers while other
tasks can run. The collector retains their pending continuations. Halt and
cancellation discard queued work and release owned files; an OS call already
running may finish later without resuming the program. See
[environment and file effects](docs/native-files.md) for ownership and limits.
Native networking uses nonblocking sockets and descriptor readiness on the VM
scheduler. Socket waits leave file workers available, and timers share socket
registration order. See [native networking](docs/native-network.md) for results,
resource bounds and Windows/Unix differences.
Arbitrary foreign source is retained by the loader; the native `run` command
rejects its execution. Executable JavaScript and executable C support foreign
companions for their respective targets. The C effect driver currently covers
console output, tasks, timers, channels, environment lookup and files; see
[executable C](docs/executable-c.md) for remaining effects and runtime limits.

`serve` checks once and reads newline-delimited JSON calls from standard input.
It returns a flushed JSON response for each request, allowing a client to pass
typed constructor trees and receive game states or other structured data.
For example, `{"id":1,"entry":"identity","args":[{"constructor":"Zero","fields":[]}]}`
returns `{"id":1,"value":{"constructor":"Zero","fields":[]},"error":null}`.
IDs must strictly increase. See the [data protocol](docs/data-protocol.md) for
error handling and size limits.

Output defaults to JSON when redirected and text in an interactive terminal.
Use `--output-format json`, `text` or `csv` explicitly. Nested reports may not
have a CSV representation. Logs go to stderr; optional `--log-file` enables
NDJSON logs. `TEAMY_BEND_HOME_DIR` and `TEAMY_BEND_CACHE_DIR` override the
application directories. `RUST_LOG` or `--log-filter` configures diagnostics.

See [compatibility and resource limits](docs/compatibility.md) for the supported
surface and remaining work. Unsupported features return errors.
Current work prioritizes the Bend2 engine and uses existing Poche models for
regression checks. Full Poche formalization remains open, with its application
architecture preserved.

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
