# Checking-only fixtures

`list_reverse_sort.bend` is an unchanged copy of
[`tests/eval/list_reverse_sort.bend`](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/eval/list_reverse_sort.bend).
Copyright 2026 HigherOrderCO; Apache-2.0. See NOTICE and `licenses/Apache-2.0.txt`.

It has no `main`. Qualification checks its concrete reverse, reverse-involution
and duplicate-preserving insertion-sort proofs. Named-definition normalization
is separate evidence, not a generated or native program execution. Authored
false-proof controls change only an expected list and must fail strict checking.
