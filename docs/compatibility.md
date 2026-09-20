# Compatibility and verification scope

The reference is Bend 2.0.5, revision
`e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. This tool currently implements a
supported subset. It is not a drop-in replacement for the full upstream CLI.

## Supported surface

- Dependent functions, algebraic datatypes, kinds and quantities; erased,
  affine and reusable binders; structural recursion; equality, reflexivity and
  equality rewriting. Every ordinary law must have a proof. Unsafe/foreign
  assumptions and holes in checked terms (including holes discarded by
  reduction) are refused.
- Source `type`, `def`, `law`, closed compile-time templates, nested/multiple
  constructor patterns, parallel lets, reusable binders, local module
  imports/aliases, literals and operators,
  pure do notation and array syntax lowering. Syntax support alone does not
  provide an absent library operation or runtime intrinsic.
- Pure normalization, typed Nat batch calls, persistent constructor calls,
  and Rust-authored JavaScript/C11 generation with closures, constructors,
  pattern matches and simultaneous let scope. Persistent calls use a separate
  lazy native data runtime after checking their arguments. No TypeScript
  interpreter implements these operations.
- 367 selected pure Base source declarations: dependent pairs/existentials, sums,
  equality helpers, Bool/Cmp, Nat arithmetic, Maybe/Result/List, word structure
  and Map/Set, Char/String operations, Bool/Maybe formatting, List templates, Array operations and pure
  Word/U32 helpers, decimal U32/Nat formatting and parsing, and checked addition
  commutativity, Image quadtrees and Key/Mouse/Move/Close events. The four ordinary
  Image cleanup helpers preserve checked structural recursion; they do not
  provide GPU offload or parallel reclamation. Array creation uses a power-of-two
  depth, indexing wraps,
  and clone/get require reusable elements; swap/set/map retain affine ownership.
  `src/syntax/base.bend` is the exact inventory (322 definition forms including
  18 templates, 25 laws and 20 datatypes). Templates enter the checked book only
  when instantiated, so check-report counts differ from source-form counts.

GPU calls, hub fetch/publish, JavaScript readiness/networking, window/audio effects,
general large native Nat computation, optimized C and GPU
backends, and upstream CLI parity remain unfinished. F32 syntax/representation
does not establish floating-point proof support. Native IO execution additionally
supports [all 37 numeric primitive contracts](numeric-execution.md), also implemented
by executable JavaScript. Strict checking and
the pure compilers do not admit their opaque implementation assumptions.

## Executable checking and native IO

`run` loads a separate execution-only Base and checks ordinary terms with the
same proof/resource rules. Foreign declarations need loader-owned origin and a
direct return reference to actual Base IO. `ExecutableBook` is a distinct API
with no conversion to `CheckedBook` and no proof-evaluation method. Its foreign
signature metadata describes runtime assumptions. Unsafe definitions remain
unsupported even on this executable path.

Native execution supports IO.pure/bind/die/pass/try, console effects, tasks,
timers, channels, environment lookup, files and TCP/UDP. Requests are private runtime
values: matching them as ordinary IO.OP
constructors fails without executing the requested effect. UTF-8, NUL, output
ordering, cancellation, write failures, discarded Emit payloads and Halt exit
codes have executable regressions. IO aliases select the driver; a user-defined
type named IO does not. Pure main output still uses canonical core syntax.

At the initial console milestone, ten unchanged upstream IO fixtures were compared to actual upstream-generated
JavaScript. Eight matched stdout, stderr and exit status exactly. The two
expected refusals matched stdout/status and absence of unintended effects;
diagnostic wording differed. Those results describe that milestone's behavior.
Native effect text now follows C's io_utf8 encoding for all U32 Char codes,
including surrogates and values outside the Unicode scalar range. JavaScript
still validates source Char constructors eagerly, including discarded values;
native evaluation remains lazy. Numeric parsing has separate scalar validation.
See [environment and file effects](native-files.md) for the current text and
error contracts. The earlier console comparison does not certify arbitrary C FFI.

Execution uses the existing 2,000,000-step, 131,072-thunk/environment and
4,096-frame limits, with an 8 MiB ceiling per decoded console string. Unreachable
thunks and environments are reclaimed at evaluator safe points using stable IDs
and explicitly retained caller values. Collection consumes the same step budget;
the arena caps now bound retained slots rather than cumulative allocation.
A full live graph still fails. U32.show uses
ordinary checked decimal doubling over bits to avoid repeated division exhausting
that arena; its upstream helper names retain truncation/accumulator behavior.
Boundary/sample and helper tests cover zero, every bit boundary and u32::MAX.

Arbitrary C/JS import descriptors are retained and deduplicated, but this native
backend rejects their execution explicitly. The separate
[executable JavaScript compiler](executable-javascript.md) supports synchronous
foreign imports, native representations and callbacks. The C effect driver,
JavaScript network, window/audio Base effects and unsafe execution remain
required work in [the design](effects-design.md).

Executable JavaScript additionally supports IO.spawn, IO.sleep and IO.now with
a cooperative FIFO scheduler. Undefined foreign returns suspend; saved
continuations can resume through io_push. Main completion waits for spawned
tasks, while Halt stops all pending work. Timers use a monotonic Node clock and
synchronous waits; JavaScript event-loop callbacks and promises are not pumped.
Channels support buffered and zero-capacity transfer, closure and ordinary
fork/join. A separately sealed opaque Chan family grants executable handle
types; strict checking still rejects that unfilled law. List.for_each executes
callbacks sequentially. Descriptor readiness remains unsupported.

Native `run` supports IO.spawn, IO.sleep and IO.now with cooperative FIFO tasks.
Spawn continues its parent; sleeping yields even for zero milliseconds; ready
tasks run before timers and all overdue timers resume in registration order.
Children outlive main. Halt, cancellation and errors discard remaining tasks.
Task/timer roots participate directly in the native garbage collector. Native
channels add sealed handles, buffered and zero-capacity transfer, FIFO waiters,
close/drain behavior and ordinary fork/join. Buffered payloads and waiting
continuations are traced directly; all channel state is discarded on driver exit.
See [native channels](native-channels.md) for target differences and limits.
Arbitrary native foreign callbacks remain unfinished.

Both targets implement IO.get_env and File.open/read/read_bytes/write/close.
File is a loader-sealed opaque affine Type, absent from strict proof Base.
Open accepts exactly r/w/a. Reads and writes return the handle outside Result,
including on failure; close consumes it. Byte reads return List<&2, U32> octets.
Environment lookup distinguishes missing names from existing empty values.

Native open/read/write park through bounded host workers. The collector traces
pending continuations, and only the VM thread packs replies into Bend values.
Halt and cancellation remove queued work and close owned files. Already-running
OS calls may finish later; their results are discarded without resuming the VM.
JavaScript file calls remain synchronous, matching upstream JS. The Node adapter
honors a supplied BEND_SYS and otherwise provides bounded descriptor reads and
common error messages with a libuv fallback. See
[environment and file effects](native-files.md) for text, errno, NUL behavior,
resource bounds and the limits of cancellation. Unix host code still needs
runtime validation on Unix.

Native execution also implements all eleven TCP/UDP/close effects with sealed
affine Socket and Listener types. Accept and receive park before their first
syscall; connect and send attempt immediately. Nonblocking sockets share the
VM readiness poller, leaving file workers available. Ready descriptors and due
timers follow their common registration order after worker completions. Pending
continuations are collector roots; all live and parked sockets close on driver
exit. See [native networking](native-network.md) for error/handle behavior,
partial sends, datagrams, cancellation and bounds. Generated JavaScript still
rejects reachable network calls until its host readiness provider is implemented.

Native IO.now retains the OS monotonic clock origin: Windows performance-counter
nanoseconds or Unix CLOCK_MONOTONIC, floored to milliseconds. It is not rebased
per invocation or narrowed to U32. Waiting polls cancellation at intervals of at
most 100 ms without charging evaluation steps for idle time. Compact executable
Nat values support these timestamps up to the reference native 48-bit bound;
subtraction, comparison, decimal display and U32 conversions have guarded fast
paths. Ordinary constructor matching remains available, and proof normalization
and output limits are unchanged. This does not establish general large-Nat
arithmetic parity. Platform clock validation currently covers Windows; the
Unix adapter still needs execution on a Unix host.

App.more/fold/play run through native IO and executable JavaScript. Finite
playback consumes frames in order, preserves affine state, stops at None or
Halt and skips the view callback. App and its helpers live in executable Base;
Image and Event are also available to strict checking and the pure compilers.
The window-dependent App.next/turn/draw/step/loop/run helpers remain unfinished.

Compact executable words and bounded reclamation now pass all 21 Image probes
using the exact upstream build/fold definitions. Native execution completes
the depth-six sum of 8,386,560 over 4,096 pixels and the corresponding leaf
count of 4,096. Following one path and forcing Image.free also pass through six.
The original full Image observation program matches upstream on native and
generated JavaScript execution. No workload was reduced or limit raised.
These are CPU data-runtime results; window effects and GPU execution remain
unfinished.

Imported standalone models may use Nat literals and default Nat operators for
their own locally declared Nat type. The loader resolves the generated names in
the same module scope as explicit constructors; separate modules' types remain
distinct. This is a deliberate extension: the reference rejects a namespaced
custom Nat's `0n` even where its equivalent explicit `Zero{}` checks. Modules
using the global bundled Base retain global literal and operator identities.

## Deliberate resource limits

Compile-time template arguments must be closed. Specialization uses structural
keys that preserve binder identity and share alpha-equivalent arguments.
Instances retain ordinary erased-argument type checks, affine/reusable resource
checks, source ordering and structural descent. Like upstream, unused template
bodies are parsed but not type-checked until instantiated. The complete module
graph allows at most 256 instances and 16 nested specializations; each key is
limited to 2,048 bytes, 128 term levels and 4,096 visited term nodes.

The kernel uses a per-definition/evaluation step budget and nesting limits.
Its current AST input and dynamic recursion limits are 128; the step budget is
2,000,000. Parsing caps expression/body/do nesting, pattern compilation and
import nesting at 64; telescopes, argument lists and strings at 128; Nat literals
at 128; expression spines and intermediate term depth at 256. A combination of
constructs can reach a stricter limit before any individual limit is reached.

Limits return failure, never a successful proof. They deliberately restrict
large programs until compact representations and iterative traversals are
implemented. Evaluation performance and sharing are not at upstream parity.

The persistent data runtime has separate arena, continuation and output limits
documented in [the protocol](data-protocol.md). It does not replace the proof
normalizer. The baseline C backend limits instruction generation to 250,000
nodes and defaults to 2,000,000 runtime steps, 64 MiB of tracked allocations,
nesting depth 1,024 and 8 MiB of output. Generated programs release their arena
after either success or failure. C and JavaScript emit an explicit erased marker
for proof/type results; the constructor-only data protocol rejects such results.

## Reproduce the upstream audit

The completed native-network candidate audit on 2026-09-20 covered all 1,302 fixtures: 362 expected-positive programs
checked, 491 expected-positive programs were rejected, and all 449 expected
failures were rejected. There were zero abnormal exits and zero accepted
expected-failure fixtures. These are acceptance counts, not a parity percentage.
The numeric helper slice adds `proof/word_add_comm.bend` to the previous 360
accepted positives. Transparent let aliases in structural descent add
`proof/rewrite_type_family.bend`; no previous positive was lost. This follows
only aliases and annotations, without unfolding computed recursive arguments.
The Image/App, compact-word and reclamation slices preserve those acceptance decisions.
The native-network implementation passes 487 tests, including 5 compile-fail API
boundary examples, with 2 optional
local profilers ignored. Strict Clippy checking covers the library and
integration tests. Its frozen candidate records 97 compiled-source fingerprints.
[Native networking](native-network.md) distinguishes real loopback, forced-GC
and adapted upstream C evidence. The [implementation plan](implementation-plan.md)
tracks publication validation and retained comparison receipts.
Generated executable behavior has
separate actual-output comparisons in
[executable JavaScript](executable-javascript.md) and
[numeric execution](numeric-execution.md); the strict audit does not test it.

Native scheduler validation includes nine checked-program tests, nine private
virtual-clock/collection tests, and 13 complete programs matched against actual
upstream generated JavaScript. The upstream comparison runs each program with
two clock origins, including one above U32. All 256 integer comparisons and 21
Image workloads continue to pass. Native channels have separate validation
in [native channels](native-channels.md). Arbitrary native FFI and readiness
remain unsupported.

The latest pure-library comparison matches 19 complete programs against upstream.
The unchanged `base/string_kit.bend` and `base/num_kit.bend` still hit the kernel
nesting limit in their long IO chains. All 67 print expressions match in nine
shorter chunks, which does not establish whole-program compatibility. The limits
remain unchanged.

The template/collection slice added five positives over the 344-positive
audit, and Array support added 11 more without losing any previous positives.
All 32 upstream comptime fixtures still reject because the
positive programs require other unfinished features; this category does not
establish template parity. The 13 focused template tests, import tests and
native/JavaScript/C execution comparisons provide the supported evidence.

The earlier numeric/parser expansion exposed a debug Windows stack overflow in `run/fuel_loops.bend`
was reduced to a large unary Nat, fixed with a bounded CLI worker stack, and
rerun through this complete audit. Kernel proof limits remain unchanged.

Use an existing separate upstream checkout at the reference revision:

```powershell
cargo build --locked
cargo run --example audit_upstream -- <bend-reference>
```

The auditor runs each of the 1,302 upstream fixtures with `#|` expectations in a
separate native CLI process. It compares check acceptance only. A rejected
negative fixture can fail for a different reason; that does not establish
diagnostic parity. A runtime-error fixture can legitimately pass checking.
Runtime values, exact diagnostics, compiled behavior and performance require
separate comparisons. Abnormal exits are counted separately from rejections.

The complete audit output can be stored under ignored `target/`. Do not publish
raw diagnostics containing local paths. A snapshot executable may be supplied
as the second argument to keep development builds free during a long audit.

## Poche integration

Engine work remains the current priority. Existing Poche checks provide
regression coverage while full formalization remains open; this work preserves
the application's existing architecture.

The Poche integration is maintained in that project's `models/bend/`,
`crates/poche-conformance/src/bend.rs` and `docs/bend-conformance.md`. The native
gate checks seven equality terms and compares 15,503 scalar cases against
independent Rust rules, including the conventional scoring/trick oracle.
Both a false equality and a well-typed scoring mutant must expose their defects.

This establishes the documented finite scalar domains and two universal
definitional equalities. The entire game state machine, full deck, network
protocol, privacy, UI and money-transfer implementation remain outside that
receipt. Full state-model integration remains in the active implementation plan.

The separate `micro-v1` model now covers the complete six-card, two-seat 1/2/1
state schema. Its trajectory gate passes 21 transitions, 22 inspections, all
300 deal partitions and eight rejection controls. A distributed sample passes
1,000 states and 1,000 edges. Seven compiled witnesses and two well-typed model
mutants also pass their expected-value checks.

The final native comparison with the clean `6802c1e` release completed on
2026-09-19: all 431,800 reachable states, 549,896 labeled edges and 863,600
player observations matched. It also checked all 300 deal partitions and
eight rejection controls. Its 981,720 requests completed in 1,658.848 seconds.
This is exhaustive conformance for the registered six-card, two-seat 1/2/1
graph, not an inductive proof or full-deck/network verification.

The receipt fingerprints the actual executable, frozen model, compiled Rust
adapter/oracle sources and transcript. The checked model BLAKE3 is
`3798ecaad4f3bff64d9648452c0f7f2090bde1294a5beb8c505d8bce4fedf226`;
the transcript BLAKE3 is
`a273a13c52d7e36f956c30b0675cfefeb60ace620dece023b0beb361b0bbed39`.
The Poche integration remains in its local checkout; these results do not
imply that its changes have been published.

A separate imported privacy file now proves seven symbolic observation laws.
With the viewer's own hand, public fields and opponent hand count fixed,
changing hidden opponent cards leaves that observation equal. Phase corollaries
also hide undealt/captured identities as specified in Poche's privacy document.
A well-typed mutation selecting the other player's hand is rejected by the
unchanged theorem. These laws quantify over typed micro-model values; they do
not establish protocol, trace or full-deck confidentiality.

The clean `9da68e5` release checks those laws and passes the scalar and trajectory
regressions. The micro model and all 13 compiled conformance-source fingerprints
remain unchanged. The exhaustive graph evidence is still attributed to `6802c1e`.

The subsequent clean `06f0808` release also passes all seven privacy laws and
the mutant, 15,503 scalar comparisons and the complete diagnostic trajectory
(22 states, 21 transitions, 44 observations, 300 partitions and eight controls).
The trajectory uses 67 requests in 312 ms. All 13 compiled source fingerprints,
model bytes and privacy proof/runner bytes are unchanged; the exhaustive graph
was not rerun or reattributed to this release.

The clean channel release, `f27f7ba`, passes the same privacy, scalar and
trajectory gates. Its 67 trajectory requests complete in 301 ms. All 13 compiled
conformance fingerprints and 17 source fingerprints remain unchanged from the
timer release. The retained executable BLAKE3 is
`0e535bb8516581f31fd16788f52879abe7056992af9c1ad29d83b5ef03413b3a`.
The exhaustive graph remains attributed solely to `6802c1e`; Poche edits remain
local.

The clean Image/App release, `39f3dc5`, also passes all seven privacy laws,
15,503 scalar comparisons and the micro trajectory, with their negative controls.
Its 67 trajectory requests complete in 300 ms. All 13 compiled conformance
fingerprints and 17 source fingerprints are unchanged from the channel release.
Its executable BLAKE3 is
`1a413d37286879e272957003f4c004c3f5e107460a9d7baf764bda92e6786507`.
This regression does not rerun or reattribute the exhaustive graph.

The clean compact-word release, `a7ddfcc`, passes those same privacy,
scalar and trajectory gates, including all negative controls. Its 67 trajectory
requests complete in 482 ms. All 13 compiled conformance fingerprints and 17
source fingerprints remain unchanged from the Image/App release. Its executable
BLAKE3 is `5e2acc14b82e2335fc07fcb180dc9452d2dd373d7c8b7b4f215311eabf057f90`.
Poche changes remain local. The exhaustive graph retains its original attribution.

The latest clean reclamation release, `6356963`, passes the same seven privacy
laws, 15,503 scalar comparisons and micro trajectory, including all negative
controls. Its 67 trajectory requests complete in 319 ms. All 13 compiled
conformance fingerprints and 17 source fingerprints are unchanged from the
compact-word release. Executable BLAKE3 is
`24f13a9421a18f51892e4402d0ddf1cc55462a3dc4082876f86c6d7ea676dd1c`.
The original exhaustive graph was neither rerun nor reattributed.
