# Native channels

Native `run` implements the executable Base contracts Chan.new, Chan.send,
Chan.recv and Chan.close, and the ordinary checked IO.fork/join helpers.
These contracts remain execution assumptions. Chan is absent from strict proof
Base; runtime handles cannot become proof evidence.

## Behavior

- A capacity greater than zero buffers values in FIFO order. A full buffer parks
  the sender; receiving frees a slot and transfers the oldest parked send into it.
- Capacity zero performs a direct handoff between a sender and a receiver.
  The first arrival parks. The second enqueues the first task's continuation,
  then continues its own task without yielding.
- Close rejects waiting sends with False and wakes waiting receivers with None,
  preserving registration order. Buffered values remain available until drained.
  Later sends return False and empty receives return None. Repeated close is safe.
- Private generation-checked handles remain closed after their table slot is
  reused. Handles can be copied and sent through channels; affine payloads retain
  the checker's usage restrictions. Payloads and continuations remain lazy.
- Fork/join uses the same ordinary Base definitions as upstream. Join receives
  and closes the channel; joining its copied handle again halts with the upstream
  closed-channel message.
- Children remain live after main finishes. If all tasks are parked without a
  runnable task or timer, the driver reports deadlock. Halt, cancellation and
  errors discard pending tasks, timers, buffered values and channel waits.

The native representation follows upstream C's distinct receiver sentinel.
Erased proof/type payloads are valid values, including sender-first rendezvous.
Upstream JavaScript uses null for both erased values and the receiver marker,
so that case deadlocks there. The generated JavaScript backend preserves that
reference behavior; native execution deliberately follows the native C target.

Handles are private runtime values, not source constructors or public row
objects. Native foreign code cannot manufacture them. Constructor matching and
pure result materialization do not expose their representation.

## Collection and bounds

The Machine owns the channel table. Collection traces buffered payloads,
parked send payloads and every waiting continuation directly. Detached transition
results remain rooted while their continuation applications are enqueued.
Allocations do not collect; subsequent evaluation safe points trace the new
scheduler roots. An open channel is retained until close/drain or driver exit,
matching upstream native table ownership even if all source handles are dropped.

The channel table, retained payload count and waiter count each have an aggregate
limit of 131,072. Buffered and parked-send values share the payload limit. Logical
capacity accepts the full U32 range without preallocating that much memory;
storage grows only as values arrive. Drains release oversized queue allocations.
Table slots at generation exhaustion retire instead of wrapping and reviving an
old handle. Existing task, thunk, environment, caller-root and evaluation budgets
also apply. Exhaustion returns an error; no partial proof result is accepted.

## Validation

Nine pure state tests cover FIFO transfer, both rendezvous directions,
close/drain, stale handles and generation exhaustion, limits and root visitation.
Twelve checked-source tests cover the public native execution path, including
fork/join, affine/function/handle payloads, erased payloads, deadlock, cancellation,
Halt and private effect-request opacity. Five additional tests force collection
at every safe point and check terminal cleanup.

An ignored standalone adapter imports the production Rust channel module and
compares it with the verbatim upstream C channel/queue/effect functions, wrapped
in a portable harness. All 509 scenarios and 45,307 operations match, including
immediate results and ordered wake lists after each operation. This validates the
transition state machine; it does not claim the entire generated C runtime ran.

The frozen native candidate also passes 22 whole-program cases against actual
upstream JavaScript: 19 exact output/status matches, one expected deadlock with
different diagnostic wording, and two intentional native erased-payload differences.
Nine cases are unchanged upstream fixtures. All 256 integer comparisons, 21 Image
workloads and the full 1,302-fixture strict audit retain their prior results.
Detailed release evidence is recorded in the implementation plan.

Arbitrary native foreign calls, descriptor readiness, other
platform effects, executable C and GPU execution remain separate unfinished work.
