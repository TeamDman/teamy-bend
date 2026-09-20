# Executable JavaScript

`compile --executable FILE --output program.cjs` checks execution contracts
and generates a standalone CommonJS program. Run it with Node.js. The compiler
is implemented in Rust and consumes typed expressions from `ExecutableBook`;
foreign signatures cannot produce a strict `CheckedBook` or a proof result.
Unsafe declarations remain unsupported.

## Entry points and output

The entry is `main`. Actual Base IO, including aliases, selects the IO driver.
`Emit` discards its payload and returns status zero; `Halt` writes its message
and sets the exit status. Missing `main` prints `All terms check.`. Supported
pure result types print Bend values, including native numerics. Functions and
dependent result shapes that cannot be described by the printer reject during
compilation. Default strict `compile` retains its constructor JSON interface.

Console output uses synchronous UTF-8 writes, preserves embedded NUL and retries
partial/interrupted writes. Pending foreign requests have a private identity.
Ordinary matching, including a fallback arm, rejects a request before its effect
runs. Source Char constructors validate Unicode scalars eagerly, including
discarded values. Native `run` remains lazy and uses C's raw Char encoding for
effect text. The [target differences](native-files.md#target-differences) describe
these separate contracts.

## Foreign calls and cooperative scheduling

The backend selects the first JavaScript import for each reachable foreign
declaration. It deduplicates canonical paths and embeds sources in one shared
lexical scope, ordered by live reference discovery from `main`. Symbols use
their declaring module's local names. Missing selected sources are errors;
later imports do not silently replace them. Unreachable foreign files are not
needed. Compilation reads source but does not execute it. Running the generated
program gives that source ordinary Node host access.

The foreign interface follows the reference JavaScript representations:

| Bend value | JavaScript representation |
| --- | --- |
| Nat | BigInt |
| U32 / F32 | Number, with binary32 rounding for F32 arithmetic |
| Bool | Boolean |
| Char / String | String |
| Array | Array |
| Other datatype | Object with local `$` constructor tag and named live fields |
| Live type or proof | `null` |
| Erased argument or field | Omitted |
| Function | Curried closure that evaluates before returning to host code |

Foreign return values remain raw. The interface deliberately preserves upstream
cases such as truthy numeric Bool values, negative BigInt Nat values and lone
surrogate strings. It does not apply the typed data protocol's validation to
these values. Runtime assumptions about foreign results do not establish proofs.

The driver passes live arguments and the trailing continuation. An undefined
foreign return suspends the current task. A saved continuation resumes through
`io_push(function, argument, fresh)`: `fresh` starts an additional live task,
while false resumes an existing task. Runnable tasks execute in FIFO order.
Main completion waits for every live task; Halt stops the entire scheduler.
If all tasks suspend without a runnable continuation or timer, execution reports
deadlock. Pending requests retain their private identity throughout scheduling.

Bundled IO.spawn, IO.sleep and IO.now implement task creation, timer suspension
and monotonic millisecond time. A foreign `_need()` hook with `time: true` parks
the request until its first argument's millisecond delay expires, then invokes
its implementation. Runnable work precedes timer checks, and multiple overdue
timers resume in registration order. The Node adapter uses bounded synchronous
Atomics waits, preserving upstream's synchronous polling model. It does not
pump event-loop callbacks or await promises; promises still reject explicitly.
Companion helpers `io_bytes`, `io_text`, `io_out`, `io_errs`, `io_done`, `io_tup`
and `io_fail` are provided. `io_sys` honors a supplied globalThis.BEND_SYS.
Otherwise it combines the bounded Node file-read/error adapter with a lazily
loaded Rust Node-API socket provider. TCP/UDP and descriptor readiness preserve
raw handles, callback suspension and mixed timer/socket ordering. See
[JavaScript networking](executable-network.md) for packaging, ownership and
provider adaptation. This does not implement the full Bun/POSIX system interface;
window/audio effects remain unfinished.

## Environment and files

IO.get_env and File.open/read/read_bytes/write/close use sealed executable
contracts. File is an opaque affine Type: source code cannot construct or copy
a handle. Strict proof Base excludes it. Generated JavaScript retains the
upstream raw descriptor representation for trusted foreign code.

Open accepts r/w/a. Reads perform one synchronous host read at the current
cursor; text reads use a fresh TextDecoder and byte reads return List<&2, U32>.
Writes finish successive short writes. Read/write return the handle outside
Result even on failure, and close ignores host close failures. These calls are
synchronous, matching upstream JavaScript. They do not use the native Rust
worker pool or pump the JavaScript event loop.

The default Node adapter supplies common file-error messages with a libuv
fallback. It does not promise arbitrary libc locale equivalence. A supplied
BEND_SYS remains authoritative for descriptor reads and strerror. JavaScript
retains upstream's environment lookup and numeric error policy, including
Windows differences from native execution. See
[environment and file effects](native-files.md) for NUL names/paths, text decoding,
errno and resource bounds.

## Channels and ordinary helpers

Chan is a loader-sealed opaque executable type family. It has no source
constructors or body, and its runtime contracts cannot become strict proof
evidence. Handles are reusable even when their payloads are affine. The
generated backend implements Chan.new/send/recv/close, including FIFO buffering,
zero-capacity rendezvous, suspended senders/receivers, and close wakeups.
Buffered values remain available after close; sending to a closed channel
returns false and receiving from an empty closed channel returns None.

The foreign representation is the reference's raw row with room, ring, wait
and shut fields. Foreign-created rows are accepted with bounded accounting;
rows without ring/wait arrays or exceeding accounting limits fail when used.
Other host fields retain the raw foreign semantics. Private scheduler cleanup does not
rewrite a row retained by foreign code. Pending effect requests keep their
separate private identity.

The reference uses null as its receiver marker. Live type/proof values also
compile to null, so a blocked sender with such a payload has the same marker.
In particular, sending a proof first on a zero-capacity channel deadlocks;
receiving first or using a buffer succeeds. This observable reference behavior
is preserved and tested; channel signatures are runtime assumptions.

IO.fork/join are ordinary checked helpers: fork creates a capacity-one channel
and spawns the action; join receives and closes it. Joining the copied handle
again halts with the reference closed-channel message. List.for_each is an
ordinary template that preserves sequential callback order and stops at Halt.
Native Rust also supports channels and fork/join; its sealed handles and native C erased-payload behavior are described in [native channels](native-channels.md).

## Finite App playback

App.play specializes a closed App value and state type, then passes each frame's
Event list to tick in sequence. An empty frame list returns Some(state); None
stops playback, and Halt stops the IO driver. App.more and App.fold are ordinary
checked continuation helpers. The state type remains affine, and playback does
not invoke view or require a window. The same finite console programs run on
the native Rust IO driver. Interactive App helpers and platform windows remain
unfinished.

Six upstream-generated JavaScript comparisons agree on stdout/stderr/status:
the unchanged App.play fixture, the public example, affine state, foreign
callback order, Image observations and every Event field. Native execution
matches all five applicable playback/event/Image programs after bounded arena
reclamation. The foreign callback program remains outside its supported native
contracts. Scope and remaining limitations are recorded in
[compatibility](compatibility.md).

## Limits and verification

Generated Bend evaluation shares a 2,000,000-transition budget, allows 512
nested non-tail calls and limits native arrays to 131,072 elements. Tail calls
use a trampoline. Pure output limits are 96 levels, 16,384 visited nodes and
8 MiB of text; string construction and each console write also have an 8 MiB
byte limit. These are fail-closed execution limits, not upstream performance
parity. The scheduler shares the transition budget and permits at most 131,072
live tasks and 131,072 queued tasks/timers/descriptors/channel waiters in total. Channel
accounting also caps retained handles and buffered payload slots at 131,072 each.
File reads first clamp to INT32_MAX, then reject requests above the 8 MiB
buffer limit. File writes and decoded text have the same byte limit, and tracked
open file handles are capped at 131,072. A write making zero progress fails
instead of looping. These limits do not bound arbitrary foreign host code.
Each individual timer wait is capped
at one second before rechecking the clock; this does not shorten its deadline.
Arbitrary host JavaScript runs outside Bend's evaluation budget.

Regressions cover compile-time non-execution, unchanged output files after
failed checks, UTF-8/NUL, exit handling, callbacks, erased arguments and fields,
live proof/type nulls, shared source state, import order and request rejection.
Actual-output comparisons use separately generated upstream JavaScript and
capture stdout, stderr and exit status. Strict proof rejection and Poche
conformance checks remain separate gates.

Environment/file regressions cover missing and empty variables, exact sealed
contracts, affine handles, cursor/EOF behavior, raw bytes and TextDecoder
behavior. They also cover failed-operation handle retention, short writes,
zero-progress refusal, byte limits and supplied BEND_SYS adapters. Publication
comparisons and their target differences are recorded in the
[implementation plan](implementation-plan.md); earlier evidence below retains
its original scope.

A fixed release candidate passes 34 unchanged upstream console, foreign,
marshalling, import and numeric programs: 31 agree exactly on stdout, stderr
and status; three expected failures agree on stdout/status with different
diagnostic wording. The earlier synchronous release also passed a separate
34-case numeric matrix exactly, covering
31 generated boundary cases and the three upstream numeric programs already
included above. This includes quiet and signaling NaN payloads. Fourteen focused
emitter tests, 18 foreign/driver tests and nine scheduler tests supplement these
comparisons.

Scheduler comparisons cover 209 deterministic traces against the actual upstream
queue/wait/run implementation, including overdue timers, saved continuations,
task lifetime, deadlock and Halt. Five unchanged upstream spawn/sleep/clock
programs also match stdout, stderr and status. These timer comparisons replace
only upstream's Bun/POSIX host polling with an explicitly identified Node
timer-only adapter; they do not establish descriptor-readiness compatibility.

Channel comparisons cover 509 deterministic scenarios against actual upstream
channel operations and scheduling, including final raw rows. Nine unchanged
channel/fork/join programs also agree under two shared clock adapters: exact
waits and an injected oversleep, for 18 comparisons. Deadlock wording differs;
the other eight programs agree exactly on stdout/stderr/status in both modes.
Real-clock runs exposed variable worker order when multiple timers become
overdue together. The upstream program reproduced this variation, and both
implementations produce the same order with identical injected oversleep.
These comparisons cover timer behavior without claiming wall-clock determinism.

The full rewrite still requires arbitrary native foreign calls,
window/audio effects, executable C, remaining library contracts, GPU
support and the rest of the upstream
CLI. See the [implementation plan](implementation-plan.md).
