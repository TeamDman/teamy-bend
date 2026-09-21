# CPU parallel execution design

This document describes executable C's worker and ownership contracts. The
[implementation plan](implementation-plan.md) records implementation progress.

The change lets generated sibling tasks run simultaneously within one process.
It builds on existing simultaneous-let lowering, flat result vectors and saved
continuation frames. It does not change which Bend expressions form a fork.
GPU execution remains a separate part of the engine port.

## Worker and coordinator responsibilities

A bounded persistent CPU pool runs compiler-generated function bodies. One
coordinator owns graph validation, ready queues, dependency completion and root
results. Each worker exclusively owns its active run, saved callers and result
packet until it returns them to the coordinator.

Workers return a completed result, a graph to adopt, a request for coordinator
execution, or a failure. They do not wait for dependent tasks. This allows the
coordinator to drain active work before entering an unknown foreign callback.

The coordinator validates each returned graph before publishing its children.
It writes a child's entire result span, including ownership masks, before
decrementing the parent's dependency count. Only the final child makes the
parent ready. Scheduling must preserve each child's private sequential order;
independent children may finish in either order.

## Mark the entry being executed

Every registration starts with parallel execution disabled. This includes
`tb_register_closure`, `tb_register_generated` and `tb_register_segment`.
Trusted C companions already use all three interfaces, so their names do not
establish that a callback is safe for a worker.

Only compiler-emitted registrations opt in through `tb_register_parallel(fid)`.
The runtime stores this property separately from callback kind and result width.
The marker describes one entry's immediate body, not all functions it may call.

The dispatcher checks the actual next function ID before every invocation.
This includes resumed frames, direct registered tasks, dynamic closures,
ordinary tail calls and ready word-segment tail calls. A worker reaching an
unmarked entry returns its run to the coordinator before invoking the callback.
A marked entry cannot pass its permission to a returned closure or task.

Generated application bodies already yield a task at call boundaries. They do
not synchronously invoke an unknown callback. Generated foreign definitions
construct an IO request and capture its continuation; the IO loop executes the
effect later. These properties allow generated entries to opt in without
assuming that their higher-order arguments are also generated code.

## Keep foreign execution ordered

The coordinator drains active CPU jobs before invoking an unmarked callback.
It stops publishing further jobs until that callback returns or explicitly
enters a nested evaluation. Registry changes therefore occur while CPU workers
are idle. The runtime must also reject registry changes from a CPU worker.

The same rule applies to foreign boxed callbacks, foreign resume callbacks and
foreign word segments. Effect handlers and IO packing callbacks remain on the
coordinator. Initialization finishes before CPU dispatch starts. Pure-result
printing runs on the coordinator after the root result completes.

`IoWork.call` keeps its existing explicit host-worker contract. That callback
has no VM environment and performs host work; its packing callback returns to
the coordinator. These IO workers are separate from the new CPU pool.

A synchronous foreign callback may call `tb_apply`, `corpus_eval` or
`corpus_eval_words`. Nested evaluation retains a separate root and continuation
context. It may execute generated work without consuming a CPU worker merely
to wait for that work. Returning from nested evaluation restores the outer
context before the foreign callback resumes.

Unknown C remains trusted code. This routing does not sandbox imported C or
make detached host threads safe to mutate VM state or registration tables.

## Shared values and runtime state

Published shared values must have immutable payloads. Retaining or extracting
a shared child must not rewrite a field that another worker can read. Values
must be sealed before publication, with exact ownership metadata preserved.
Raw words must never be treated as references because their bits resemble a
tagged value. Unique owners may still reuse storage after the required acquire.

Reference counts need atomic retain and release operations, including overflow
checks and an acquire step for final ownership. Allocation metadata, free lists
and global resource accounting need a complete synchronization policy. A locked
allocator is sufficient initially; worker-local caches are an optimization.

Failure guards, active native depth and evaluation contexts belong to the
executing thread. Shared limits still bound aggregate live allocations, tasks
and saved continuations. Transferring a run between threads must not transfer a
pointer to another thread's failure guard or native stack.

Array operations and layout conversions only manipulate values and allocation
state. They do not call foreign handlers. Numeric intrinsics use the same
runtime ownership rules. F32 text conversion uses local buffers plus standard
library conversion routines; its string helpers do not execute IO effects.
Foreign callbacks must not change process-wide conversion state while CPU
jobs are active.

Pure-result printers are different: they write standard output and update
output budgets. They remain outside worker dispatch. Effects also keep their
existing coordinator-owned activation, channel and readiness state.

## Failure and shutdown

Each worker catches failures within its own stack. It reports the first failure
to the coordinator and stops accepting work after cancellation. No worker may
longjmp into another thread, print competing fatal diagnostics, or keep using
the VM arena after shutdown.

The coordinator cancels pending work, joins all CPU workers and accounts for
returned owners before releasing the arena or registration state. Partial pool
creation follows the same shutdown path. IO-worker resource lifetime retains
its existing separate contract.

## Acceptance criteria

Correct output alone does not establish parallel execution. The release needs:

- observed overlap between 2 compiler-generated child bodies on distinct CPU
  worker threads in one process, using a bounded test hook at actual dispatch
- exact results with 1, 2 and 4 selected workers for nested joins, reordered
  multiword results, saved callers and scalar tail conversions
- shared constructor, String, Array and captured-closure cases that retain exact
  masks and release all owners, including concurrent final-owner transitions
- coordinator-only execution of plain foreign closure, resume and segment
  callbacks reached through both initial dispatch and a generated tail chain
- nested foreign reentry and callback-time registration without deadlock,
  duplicate invocation or registration races
- effect and IO packing callbacks that keep their coordinator ordering while
  generated computation uses the pool
- bounded cancellation after allocation, step and continuation failures, plus
  cleanup after partial worker creation

Compare useful CPU workloads separately from these correctness checks. Record
worker counts and allocation limits with timing results. A passing overlap test
does not establish speedup, upstream scheduler equivalence or GPU completion.

## Reference boundary for unsafe ownership

The integration suite also duplicates an affine record containing nested arrays
and a captured closure through an explicit `@unsafe` reusable parameter. This
checks that this runtime's existing ownership behavior is consistent with one,
two and four workers. It is not an upstream JavaScript equivalence test:
upstream JavaScript aliases the arrays and observes earlier destructive updates.

Upstream C generates that program, but generation alone does not verify its
execution. Separate literal upstream helper probes show that explicit array
cloning preserves independent storage, a wrapped count-cell location cannot be
used directly as an affine raw block location, and captured closures reject
generic reference-count retention. Those helper results do not establish
whole-program C equivalence for the unsafe case. Ordinary shared Data fixtures
have separate actual upstream output comparisons.
