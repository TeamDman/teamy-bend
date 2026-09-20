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

Erased type arguments remain private compiler metadata for specialization of
direct references. They are absent from runtime foreign argument slots. Live
type/proof arguments have zero-valued slots. Higher-order polymorphic arrays
whose element layout still contains free type variables are refused; treating
an unresolved nested type as a fixed Array layout would produce wrong values.
Broader higher-order specialization remains part of task 5.15.

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

The implemented bundled effects cover console output, tasks, timers, channels
and environment lookup. File and TCP/UDP builtins are explicitly refused until
their C host adapters are implemented. Window/audio, GPU and the optimized
parallel C engine remain unfinished.

The cooperative driver drains runnable work before waiting. Timer/readiness
registrations retain order. After worker completions are collected, C callbacks run
while the original wait queue is traversed; a retry can precede a later original
wait. This matches upstream C and differs from the JavaScript wake-task policy.
The runtime supports `io_work`, initial read/time hooks and explicit readiness
re-parking. Worker completions are collected at the outer activation boundaries
used by upstream C; one activation drains its synchronous requests before yielding.
Waits currently poll worker completion at most every 10 ms; a wake
descriptor remains follow-up work. Native C callbacks do not use Node.

This foundation retains VM values in a bounded arena until invocation cleanup.
It does not yet implement upstream reference-count reclamation or the optimized
task/GPU allocator. Generated temporary scalars and arrays use tracked heap
frames, released on function return. Small wrappers allocate before entering
generated bodies. Internal calls pass Env by pointer through fixed native
bridges; the public foreign ABI still passes Env by value. This also avoids
MSVC's separate stack copy of Env at every call site in an unoptimized body.
Budgets cover 2,000,000 steps, 512 nested applications, 512 generated
frames (including conversion and printing calls), a
64 MiB VM arena, 64 MiB tracked host allocation, 8 MiB per host allocation,
131,072 live actions and 64 active worker calls. Queued work remains bounded by
the action limit. Pure printing additionally bounds bytes, nodes and depth.
These are component budgets, not a total-process memory limit.

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
