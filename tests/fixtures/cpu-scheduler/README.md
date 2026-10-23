# CPU scheduler differential control

`ready-before-timers.bend` is the authored MPL-2.0 scenario already exercised by
the native and generated-JS scheduler regressions. The differential runner also
compares it with pinned upstream C/JS and generated port C. It requires stdout
`main\nready\na\nb\n`, empty stderr and status 0; it asserts no elapsed duration.

The runner's `scheduler` suite additionally copies the unchanged upstream
`tests/io/sleep_order.bend` and `tests/io/sleep_timer.bend` into scratch output
directories from pin `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`. Those upstream
files remain Apache-2.0, Copyright 2026 HigherOrderCO; see NOTICE and
`licenses/Apache-2.0.txt`. They are not modified or shortened for comparison.

Missing Bun host support is recorded per observation, not counted as a pass.
The mixed-duration `spawn_sleep` golden is excluded from exact-order gating:
host descheduling can make multiple timers overdue, so their promotion can
follow registration order. Existing controlled-clock regressions cover both
earliest-deadline waits and this overdue case without wall-clock races.
