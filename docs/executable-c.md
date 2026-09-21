# Executable C

`compile --executable --target c` generates a standalone C program from the
execution-only checked book. The compiler and type-directed lowering run in
Rust. The generated program uses a CPU runtime with Bend's packed native value
and foreign-effect interfaces. Strict proof checking remains a separate path;
foreign results are runtime assumptions, not proofs.

```text
teamy-bend compile example.bend --executable --target c --output example.c
cl /nologo /TC /std:c11 /W4 /WX example.c
```

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

Erased type arguments remain private compiler metadata. Direct calls specialize
the full leading lambda telescope, including erased parameters of returned
closures and those following live parameters. Local erased or type-valued aliases
resolve in their lexical scope; simultaneous right-hand sides see only the outer
scope. Live captures and argument evaluation retain their ordinary ownership.
Erased arguments are absent from runtime foreign slots; live type/proof arguments
have zero-valued slots. Dynamic higher-order polymorphic arrays whose element
layout still contains free type variables are refused: choosing a layout from
the outer datatype alone can disagree with a concrete caller. Broader callable
specialization remains part of task 5.15.

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
environment lookup, files and TCP/UDP. Window/audio, GPU and the optimized
parallel C engine remain unfinished.

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

Reference-count cells preserve the native packed representation. Shared
constructor extraction upgrades child references dynamically and writes their
new wrappers back to the shared node. This is a correctness adaptation to the
current generator: upstream instead expects those shared children to have been
sealed by its ownership analysis. Captured closures are duplicated structurally;
the foreign term_keep contract still refuses count cells for captured closures
and tasks. Destruction and nested closure copying use iterative traversals.

Dead allocations return to their exact size-class free list. Live spans, free
classes and reference-count overflow are checked. Free lists do not coalesce
across classes, so fragmentation can still exhaust the bounded arena. Successful
IO shutdown releases parked continuations and channel payloads on the VM thread.
A guarded failure bulk-releases the arena without retraversing values whose
ownership transfer may have been interrupted. Native worker allocations keep
their separate host lifetime. The optimized parallel CPU/task/GPU allocator and
upstream borrowing/sharing optimizations remain unfinished.

Generated temporary scalars and arrays use tracked heap
frames, released on function return. Small wrappers allocate before entering
generated bodies. Internal calls pass Env by pointer through fixed native
bridges; the public foreign ABI still passes Env by value. This also avoids
MSVC's separate stack copy of Env at every call site in an unoptimized body.
Budgets cover 2,000,000 steps, 512 nested applications, 512 generated
frames (including conversion and printing calls), a
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

Whole-program comparisons execute actual upstream-generated JavaScript against
the emitted native C. Actual upstream C is generated and retained, and separate
probes compile verbatim upstream C value/registration helpers with explicit
Windows adaptations. The complete upstream POSIX/Clang C runtime has not been
executed on this host. Those distinct evidence sets do not establish full C
runtime equivalence or completion of the Bend2 rewrite.
