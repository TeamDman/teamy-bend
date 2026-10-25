<!-- SPDX-License-Identifier: MPL-2.0 -->
# List template execution boundaries

`original/list_fold.bend` is byte-identical to the
[pinned original](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/base/list_fold.bend),
under the upstream Apache-2.0 attribution in NOTICE. Other Bend fixtures are
project-authored MPL-2.0 probes. Pure graph JSON captures the complete pinned
interpreter value. Runtime stdout/stderr/status are captured from the existing
port and independently compared to upstream C/JS before qualification.

List.for_each is an execution-Base template; strict proof loading deliberately
omits it. Its load refusal is retained and not counted as executable failure or
waived from the checking census. Pure folds/filter/sort are checked separately.
The halt case emits the first iteration's output, halts on the next, and must not
run the third. Empty and nested controls retain ordering without exact-time tests.

Negative executable diagnostics are recorded separately from strict-proof
refusals and upstream diagnostics. Only fixture paths are made repository-relative.
See docs/cpu-list-for-each.md for current versus historical qualification and
unresolved differences. Raw private captures and binaries remain outside Git.
