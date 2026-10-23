# Competing failure controls

These MPL-2.0 authored controls extend `cpu-demand-review/nat_first.bend`.
They are not upstream inventory fixtures and do not grant an oracle exception.

- `nat_first_indirect.bend` puts the first argument behind a closure application,
  probing the C compiler's distinction between inline and continuation calls.
- `nat_first_bound.bend` binds the first value through `IO.pure` before the next
  continuation, probing explicit sequencing without running a private request.

`scripts/compare-failure-order.py` retains exact streams, status, source/generated
hashes, bounded repetitions and compiler profiles. The original `nat_first`,
`request_first` and `word_live_tail` controls remain in `cpu-demand-review`.
Its completion status means the experiment finished, not that parity passed.
