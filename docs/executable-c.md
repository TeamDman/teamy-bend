# Executable C

`compile --executable --target c` generates a standalone C program from the
execution-only checked book. The compiler and type-directed lowering run in
Rust. The generated program uses a CPU runtime with Bend's packed native value
and foreign-effect interfaces, with CUDA offload for eligible marked calls.
The first CUDA execution milestone has Windows hardware validation; broader
GPU parity remains in progress. Strict proof checking remains a separate path;
foreign results and `@unsafe` definitions are runtime assumptions, not proofs.
Annotated definitions use the same generated dispatcher and runtime limits;
their checking exceptions are described in [compatibility](compatibility.md).

```text
teamy-bend compile example.bend --executable --target c --output example.c
cl /nologo /TC /std:c11 /W4 /WX example.c
```

`--target native` invokes the host C toolchain and prepares an emitted GPU
program before installing a binary. Generated executables accept GPU/worker
controls and prebuild without running main. This integration is validated on Windows;
see [native builds and caching](native-builds.md).

The validated toolchain is MSVC on Windows. Generated source selects the Winsock
link library through an MSVC pragma and uses binary standard streams for exact
UTF-8/NUL output. Unix code needs a C11 compiler, libm and pthread; its source
implementation has not been executed on a Unix host. C generation neither runs
foreign source nor invokes a C compiler, and it does not package the JavaScript
Node-API provider.

## Values and generated code

U32 and F32 cross the C interface as their raw 32-bit words, including F32's bit
representation. Nat uses the upstream immediate range through 2^48-1 and refuses
overflow. Other datatypes use constructor-tagged Terms. In particular Bool and
Char cross the foreign boundary as tagged values; a String node stores its Char
field as a raw word because that constructor's field layout is flattened.

Finite datatype fields use type-directed inline layouts. Constructor nodes use
their declaration with open parameters: a generic Tuple stores two Terms even
when a particular pair's inline layout would contain two numeric words. Arrays
use the corresponding packed-u32 or Term block layout, including power-of-two
element strides. Clone preserves the original array, and indexes wrap as in
upstream. Closure captures and unary callback application preserve those boxed
boundaries. The runtime also provides the upstream closure task bridge for C
companions that use `task_node` and `corpus_eval`.

Saturated direct ordinary calls carry finite values as vectors of layout words
through generated bodies, arguments, results and joins. Definition signatures
follow upstream's raised parameter telescope, including erased parameters.
Unit arguments occupy no words; an empty result layout uses one boxed word so
each child still has a distinct result destination. Dynamic closures, foreign
callbacks and heap nodes keep their explicit boxed representation boundaries.
Native scalar tail crossings compose their cast and ownership changes in the
dispatcher without retaining conversion frames. Finite datatype conversions
at dynamic callable boundaries still use tracked frames; broader conversion
and callable optimization remains unfinished.

Generated calls use an explicit dispatch loop. Direct typed calls return a
function ID and borrowed argument/ownership spans to that loop, which validates
both spans and copies the argument words before releasing the source frame. A self-tail call reuses its
frame and capture storage, including when argument order or active sum variants
change. Other tail calls release the current frame before dispatch continues.
Boxed applications retain their closure-task boundary. A non-tail call saves its caller's
program counter, result slot, captures, argument and scratch values in a tracked
heap frame. After the child finishes, the caller resumes immediately after the
call. Definition thunks and applications introduced by pattern matching use the
same mechanism. Function heads, arguments and constructor fields remain strictly
evaluated, without repeating earlier evaluation on resume.
Generated recursion no longer grows the native C call stack. Pending work still
uses bounded storage, and dispatch and resumption consume the evaluation budget.
Every direct transition also rechecks worker eligibility before invoking the
target. Forks retain their task graphs and result destinations. This removes
intermediate task allocation from ordinary typed calls; it does not yet provide
upstream's full native-body fusion or borrowed-parameter optimization.

Foreign C callbacks keep their synchronous ABI. Each nested evaluation owns its
own pending stack, and a callback's returned root tasks finish before the caller
receives its result. Calls made recursively by foreign C code remain subject to
the native call-depth limit. Conversion and printing helpers remain synchronous
with separate bounds.

Simultaneous lets discard unused or erased bindings. When at least two remaining
right-hand sides are calls, the generator evaluates their heads and arguments
first, then publishes child tasks for the final applications. A join holds the
live outer captures and resumes the body after every child result arrives.
Mixed groups, partial applications and native intrinsics retain sequential
lowering. Raised definition arity and dynamic calls follow upstream's call
classification. Each child has its own private sequential continuation frames.

Tasks use upstream's payload followed by a continuation and packed index/remaining
footer. `FID_CLO_APPLY` has two payload words; registered closure tasks have their
captures and final argument. `FID_IO_EMIT` preserves its single payload in an
Emit constructor. Task destruction releases payload owners without following
the weak continuation link. Canonical roots use `TERM_HOLE` and index zero.

The coordinator validates returned fork graphs before running them,
detaches embedded children and delivers each result exactly once. A ready
non-root task can also enter through a closed chain of one-child continuations.
Missing external siblings, cycles, duplicate nodes, invalid destinations and
dependency counts fail explicitly. Nested foreign evaluations have separate
root results. Generated sibling bodies can execute concurrently on CPU workers.
Registered boxed callbacks return one Term; generated word segments
return an explicit vector outcome. Raw word bits cannot become task controls.
Argument and result vectors carry exact ownership masks, including finite sums
whose active variant changes a slot between a reference and a raw word. Graph
validation reserves each child's complete result span before execution, rejects
overlap, and decrements the dependency count once per completed child. Saved
callers resume into checked vector destinations. `corpus_eval_words` exposes
multiword results to trusted C with an explicit output capacity and ownership
buffer; `corpus_eval` retains its one-word result contract. Broader callable
specialization and upstream optimization parity remain open.

## CUDA offload

Named calls such as `walk!(depth, value)` preserve their GPU mark through typed
lowering. Generated C embeds device source for eligible functions and their
dependencies. The host dynamically loads CUDA and NVRTC when an eligible task
first runs. Native `run`, proof evaluation and JavaScript retain the call's
value semantics without GPU offload.

Set `BEND_GPU` in the generated program's environment to `auto` (the default),
`off` or `on`. Auto mode falls back to CPU only when CUDA is unavailable. Off
mode never initializes CUDA; on mode requires CUDA for an eligible marked task.
Compilation, allocation, transfer and execution failures terminate the
invocation without replaying consumed arguments on the CPU.

The coordinator drains CPU workers before exporting a ready graph. Device
execution shares the CPU value representation and uses persistent frames,
bounded queues and an iterative dispatcher. The module, stream and buffers
remain available across offloads in one invocation. Completion restores value
storage, ownership and allocator state before resuming parked CPU callers.

Actual Windows/CUDA execution covers recursive forks, self-tail calls, closures,
owned multiword results and shared arrays. Full GPU parity remains open,
including broader numerical and effect behavior, compilation caching, upstream
CLI controls, Unix CUDA, Metal and performance qualification. See
[generated CUDA execution](executable-gpu.md) for requirements, limits and
validation scope.

## CPU workers

The runtime creates persistent workers when a generated fork has ready siblings.
By default, it uses the detected CPU count, capped at 128, and creates only as
many threads as the ready work needs. Set `BEND_CPU_WORKERS` when compiling the
C output to select a limit, for example `/DBEND_CPU_WORKERS=4` with MSVC or
`-DBEND_CPU_WORKERS=4` with a Unix compiler. A value of one executes on the
coordinator without creating CPU workers; zero selects the default.

Each worker exclusively owns a run and its saved callers. Workers return
completed words, a new graph or a request to enter foreign code. The coordinator
adopts graphs, writes complete result spans and activates each join once. Child
completion order may differ while each child's sequential evaluation order is
preserved. CPU workers are separate from the existing blocking file workers.

Only compiler-generated registrations opt into worker execution. The dispatcher
checks that property again at every call boundary. Plain foreign closure,
resume and segment callbacks run on the coordinator after outstanding CPU jobs
finish. Effects, packing callbacks and pure-result printing also remain on the
coordinator. Foreign reentry retains its independent root, and registrations
require an idle CPU pool.

Before a value becomes shared, its owned descendants are sealed so retaining a
child never rewrites a published payload. Reference counts, allocation metadata
and global budgets use short mutex-protected operations; generated bodies run
outside those locks. Unique extraction and array updates preserve exact raw-word
masks. Additional count cells use the existing memory budget. Workers have local
failure guards and native-depth counters. Failure cancels pending work and joins
all CPU workers before freeing the VM, including partially created pools.

This implements concurrent execution, with a shared allocator and serialized
foreign boundaries. It does not establish a speedup or match upstream's optimized
worker-local allocation and scheduling. See the
[CPU parallel design](cpu-parallel-design.md) for ownership and acceptance details.

## Specialization

Fully supplied calls to recognized Base primitives and sealed numeric contracts
emit their existing operation directly in the current generated body. Live
arguments are evaluated once, from left to right; erased arguments have no
runtime evaluation. Each operation still checks the shared step budget and
cancellation. Partial applications and dynamic calls retain their closure ABI.
Direct primitive tail results follow their value layout rather than becoming
task controls. This removes primitive thunk/closure dispatch; general flat-call
fusion and upstream borrowing analysis remain unfinished.

Erased type arguments remain private compiler metadata. Direct calls specialize
the full leading lambda telescope, including erased parameters of returned
closures and those following live parameters. Local erased or type-valued aliases
resolve in their lexical scope; simultaneous right-hand sides see only the outer
scope. Live captures and argument evaluation retain their ordinary ownership.
Erased arguments are absent from runtime foreign slots; live type/proof arguments
have zero-valued slots. Higher-order Array elements can retain erased parameters
when their complete cell layout is stable: recursive lists and native inner
arrays remain boxed, and finite datatypes recursively check every live field.
Phantom parameters and erased fields do not affect storage. Constructor-local
existential types remain abstract at all call sites; their fields stay boxed.
This preserves field conversions, variant offsets and active ownership masks,
not just total cell width. As upstream does, Array operations require an outer
datatype; functions can occur inside its fields. An unresolved live field such
as `Maybe<A>` can change between raw and owned storage when instantiated, so it
still fails before C emission. Broader callable specialization remains part of
task 5.15.

Constructor discovery follows Array element types, including types in unselected
datatype variants that a generated printer still handles. It also inspects the
checked source of erased type arguments: a concrete datatype passed to a generic
function can require constructor rows even when it appears nowhere in the live
expression types. This does not add runtime argument slots or accept new proofs.

The specialization fixtures distinguish evaluator semantics from upstream target
support. All 22 pure variants match the upstream checker and normalizer; fourteen
also match generated JavaScript. Upstream JavaScript and C generation reject the
remaining eight returned-closure variants with open Array element types, and only
two of the 22 generate upstream C. Their execution here extends C target coverage
using evaluator evidence; it does not establish whole upstream C runtime parity.

The runtime uses 16-bit constructor/function IDs and 40-bit heap locations.
Constructor and closure arities are checked against their tables. Generated
names, scoped aliases and reserved table names are checked for collisions.
Host handles preserve the upstream 56-bit payload; a wider handle fails instead
of being truncated.

## Foreign source and initialization

The compiler selects the first C companion for each reachable foreign
declaration, in breadth-first reference order, and embeds each canonical source
file once. Namespace-local constructor macros are scoped around that file.
Their definitions use the validated numeric IDs so one alias cannot redirect
another alias. Conflicting request/table macros are refused.

MSVC cannot execute GCC's constructor attribute. The importer recognizes
unconditional, zero-argument void initializer definitions with the attribute
before the name or after the parameter list. It removes the supported attribute
and records explicit calls in source/import order. Those calls run inside the
runtime failure guard after the heap is initialized. Conditional, macro-defined,
prioritized or combined constructor attributes are refused. Other C attributes
remain the C toolchain's responsibility. Every reachable foreign request must
have registered a handler before main runs.

Foreign request fields contain the live arguments followed by their continuation.
`Effect` returns a Term or `IO_PARK`; `IoWork` retains the upstream field layout.
Worker call functions run on host threads and must not access VM memory. Packing
and continuation resumption run on the VM thread. C companions remain trusted
native code: the checked signature describes the assumed contract.

Imported malloc/calloc/realloc/free calls use scoped, tracked host allocators.
Allocation metadata counts toward the budget, including zero-byte allocations.
An active worker retains its invocation's allocation owner after Halt, so
cleanup does not free a buffer still used by that worker. Completed results from
a stopped invocation do not resume its VM.

## Effects and limits

The implemented bundled effects cover console output, tasks, timers, channels,
environment lookup, files and TCP/UDP. CUDA handles eligible computational task
graphs; foreign effect handlers remain on the host. Window/audio effects,
remaining GPU parity and full parallel C optimization remain unfinished.

The cooperative driver drains runnable work before waiting. Timer/readiness
registrations retain order. After worker completions are collected, C callbacks run
while the original wait queue is traversed; a retry can precede a later original
wait. This matches upstream C and differs from the JavaScript wake-task policy.
The runtime supports `io_work`, initial read/time hooks and explicit readiness
re-parking. Worker completions are collected at the outer activation boundaries
used by upstream C; one activation drains its synchronous requests before yielding.
A lazily installed connected loopback UDP pair wakes the descriptor poller on
worker completion. Installation and completion publication share a mutex;
collection drains the notifications together with its completion queue. During
a wait, only a wake reported by that poll snapshot collects workers before
the original ready callbacks. A completion just after the snapshot waits for
the next collection point. Notification sends retry interruption; unexpected
send errors are recorded under the mutex and fail the invocation on the VM,
even if no wake arrives. A full notification queue already contains a wake.
Descriptor waits have a 1,000 ms maximum poll interval. Native C callbacks do
not use Node.

The CPU runtime reclaims dead VM values during execution. The Rust generator
tracks owned uses: earlier uses duplicate, the final use transfers ownership,
and unused bindings are released. Constructor extraction consumes its input;
printers borrow and release any temporary boxed views. Arrays preserve nested
ownership through clone, get, replacement, swap, split and join. Exact per-cell
metadata distinguishes references from raw words, including mixed finite sums.

Reference-count cells preserve the native packed representation. Descendants
are sealed before a constructor is published as shared, so shared extraction
retains child references without rewriting the published payload. This runtime
sealing supplies the sharing guarantees that upstream obtains from its ownership
analysis. Captured closures are duplicated structurally;
the foreign term_keep contract still refuses count cells for captured closures
and tasks. Destruction and nested closure copying use iterative traversals.

Dead allocations return to their exact size-class free list. Live spans, free
classes and reference-count overflow are checked. Free lists do not coalesce
across classes, so fragmentation can still exhaust the bounded arena. Successful
IO shutdown releases parked continuations and channel payloads on the VM thread.
A guarded failure bulk-releases the arena without retraversing values whose
ownership transfer may have been interrupted. Native worker allocations keep
their separate host lifetime. Further CPU/device allocator optimization and
upstream borrowing/sharing optimizations remain unfinished.

The VM reserves stable virtual addresses for its payload and ownership arrays,
then commits pages through the allocated prefix. Logical capacity remains
bounded separately. New allocations commit both arrays while holding the VM
lock, before advancing the bump pointer; free-list reuse retains its existing
pages and links. Shutdown releases both mappings after workers finish. Windows
reservation and commitment follow the [VirtualAlloc contract](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualalloc).
The POSIX mapping/protection counterpart remains unexecuted on this validation
host; its physical backing follows the operating system's memory policy.

Generated temporary scalars and arrays use tracked heap frames, released when
their function completes or transfers a tail call. Synchronous conversion and
printing wrappers allocate their own scratch frames. Internal calls pass Env by
pointer through fixed native bridges; the public foreign ABI still passes Env by
value. This also avoids
MSVC's separate stack copy of Env at every call site in an unoptimized body.
Budgets cover 2,000,000 steps, 512 nested foreign evaluation entries,
512 synchronous helper frames, 65,536 live generated frames
(`BEND_MAX_CONTINUATIONS`, including the active generated call), 65,536 live task
records (`BEND_MAX_TASKS`), a
64 MiB VM payload arena and an equally sized ownership-metadata arena,
64 MiB tracked host allocation, 8 MiB per host allocation,
131,072 live actions and 64 active worker calls. Owned socket rows have a
131,072-entry bound and are reused after close. The worker notifier adds two
internal sockets. Queued work remains bounded by the action limit. Pure printing
additionally bounds bytes, nodes and depth.
These are component budgets, not a total-process memory limit.

## Files and environment

`File.open` accepts exactly `r`, `w` and `a`. Invalid modes and NUL-containing
paths fail synchronously; path NUL takes precedence over an invalid mode.
Valid open, read and write requests use host workers. Reads issue one syscall,
preserve short results and the cursor, and clamp the U32 count to INT32_MAX
before checking the 8 MiB request budget. A larger request fails explicitly;
it is not silently shortened to fit. Returned data and strings remain subject
to the separate VM allocation and evaluation budgets.

Writes preserve raw UTF-8, embedded NUL and newlines. Partial writes continue
from the remaining bytes; the first syscall error ends the request, including
EINTR. An empty write performs no syscall and succeeds even for an invalid or
read-only descriptor. Zero-progress writes fail explicitly instead of repeating
forever. Reads and writes return the descriptor on both success and failure;
close is synchronous and ignores the syscall error, as upstream does.

The native C decoder deliberately preserves upstream's reverse byte scan,
including its handling of malformed UTF-8 and split sequences. It does not
substitute JavaScript's replacement decoder. `File.read_bytes` returns the raw
octets. Windows opens convert validated UTF-8 paths to UTF-16 and select binary
CRT mode, preserving CRLF, NUL and 0x1A. Error codes are CRT errno values. Invalid
foreign descriptors use a thread-local CRT error handler; the process-wide
handler is untouched. Out-of-int-range descriptors produce EBADF instead of
being truncated into a different descriptor.

Handles retain the upstream raw descriptor ABI. The runtime owns descriptors
created by bundled `File.open`; explicit close, normal completion and Halt
release them. In-flight workers retain their allocation owner and a descriptor
lease until their syscall finishes. Shutdown cancels queued requests, closes
idle owned files and discards late completions without resuming released VM
state. It does not interrupt an OS syscall already in progress. Closing an
owned handle with a concurrent trusted foreign alias defers the close until its
worker leases end. Row storage is reused and bounded by the action limit.

Trusted C effects can supply other raw descriptors, which read/write/close
accept without automatic ownership adoption. Foreign code that bypasses these
helpers is responsible for its own descriptors and aliases. This runtime does
not claim to clean up arbitrary native resources created by an import.

Environment lookup distinguishes missing and existing empty variables; NUL or
`=` in a name returns ENOENT. Windows uses the Unicode process environment,
independent of the active ANSI code page, with a bounded retry for concurrent
value resizing. Invalid UTF-16 values return EILSEQ. This is an explicit host
adapter to upstream's narrow `getenv`;
Unix retains native `getenv` and the C byte decoder.

## TCP, UDP and readiness

The standalone C runtime implements TCP.listen/accept/connect/send/recv,
UDP.bind/send_to/recv_from/poll and Socket.close/Listener.close. Socket effects
use nonblocking descriptors and the event loop, independently of file workers.
Accept, TCP receive and UDP receive park before their first syscall, including
zero-length receives. Connect and sends try immediately and park on the relevant
pending/would-block result. A readiness race parks the request again in queue
order. UDP.poll performs one immediate receive and returns None when no packet
is available; Some can contain an empty packet.

Addresses use upstream's strict numeric IPv4 parsing, including rejection of
leading-zero octets, embedded NUL and ports above 65535. Listen and UDP bind
accept port zero. Listen uses backlog 16 and attempts address reuse. Handles
remain real descriptors, with full Windows SOCKET width within the native
56-bit representation; Unix additionally requires an int-sized descriptor.
The disposable Windows C-network test programs define
`TEAMY_BEND_TEST_LOOPBACK_NETWORK` and bind to `127.0.0.1`; ordinary generated
programs keep the wildcard bind and retain their normal network behavior.

TCP send retains its offset across partial writes and readiness waits. Empty
sends make no syscall; zero-progress sends fail explicitly. Other syscall
errors, including EINTR, return through Result with the original socket. TCP
receive preserves short reads and EOF. UDP sends one datagram, and receive
preserves the truncated prefix and sender while consuming the whole datagram,
even with a zero-byte buffer. Receive counts clamp to INT32_MAX before the
explicit 8 MiB limit. Network buffers share the tracked 64 MiB host allocation
budget with other effects. Text uses the native C codec described above.

Windows uses Winsock errors and Unicode system messages; address syntax errors
remain EINVAL 22. Pending connects inspect SO_ERROR after readiness. WSARecv and
WSARecvFrom preserve copied bytes on WSAEMSGSIZE. WSAPoll invalid-descriptor
events retry the effect so it returns its ordinary error, including when all
rows are invalid. Poll retries interruption. Unix sends suppress SIGPIPE with
MSG_NOSIGNAL or SO_NOSIGPIPE on supported platforms.

Bundled listen/bind/connect/accept calls register owned sockets immediately;
setup failure, explicit close, normal exit and Halt release them. Parked socket
requests are cancelled at shutdown. Unlike active file syscalls, these socket
operations do not continue on worker threads after Halt. Foreign C can supply
raw descriptors without automatic adoption. It remains responsible for borrowed
aliases and any raw closes that bypass runtime ownership helpers.

## Validation boundary

Production tests compile emitted C with strict warnings and execute it. The
suite covers native values, boxed data, arrays, captures, foreign callbacks,
erased slots, initialization, aliases, CLI packaging and refusal/resource
boundaries. Additional checked programs exercise the worker and scheduler ABI.
The [implementation plan](implementation-plan.md) records gate totals and
retained release evidence after validation.

CUDA hardware tests run separately from the portable gate. Their retained
receipts distinguish generated Bend programs from adapter and shared-helper
probes. The [GPU validation scope](executable-gpu.md) describes what has executed;
correct output and overlapping device tasks do not establish a performance gain.

Whole-program comparisons execute actual upstream-generated JavaScript against
the emitted native C. Actual upstream C is generated and retained, and separate
probes compile verbatim upstream C value/registration helpers with explicit
Windows adaptations. The complete upstream POSIX/Clang C runtime has not been
executed on this host. Those distinct evidence sets do not establish full C
runtime equivalence or completion of the Bend2 rewrite.
