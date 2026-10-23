# Competing runtime failures at the frozen upstream pin

Research candidate: `6e5bd61b5f2cf7e1e4de7c398f17cfe04a0150cb`.
Target: [Bend e6676b0](https://github.com/bendlang/bend/tree/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599).
This investigation changes no demand rule, failure ordering, resource limit or
accepted exception. The upstream-C diagnostic discrepancy remains **open**.

## What the pinned sources establish

The [Lean core strategy](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.lean#L9347)
explicitly specifies deterministic, left-to-right weak reduction on erasures:
reduce the head first, deepen arguments in order, and invoke a saturated definition
after its arguments are deep. That is stronger evidence than assuming by-value
alone determines order. However, the model
[excludes foreign/bodyless definitions, number sections and the error window](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/bend.lean#L99).
It does not establish which of a runtime Nat overflow and private-request fault
must be printed when both arguments fail. The Lean file was inspected, not built
or newly proved in this environment.

The [usage account](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/docs/BendTT/main.typ#L481)
supports strict executable arguments. The [IO guide](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/guide/GUIDE.md#L340)
separates constructing an action from handling its effects and sequences `do`
continuations. Neither inspected account supplies a cross-backend rule for
competing fail-stop diagnostic selection. Thus left-to-right is specified in the
core model; the broader runtime diagnostic guarantee is **not established** by
these sources. This is not a declaration that arbitrary output is permitted.

## Exact bounded experiment

The existing [Nat-first](../tests/fixtures/cpu-demand-review/nat_first.bend) and
[request-first](../tests/fixtures/cpu-demand-review/request_first.bend) controls
pass two failing arguments to an otherwise constant function. `overflow()`
computes `(2^24-1)*(2^24+1)+1`, one above the maximum runtime Nat.
`invalid()` attempts to inspect a private IO request with explicit public
constructor arms. It does not execute the request. These are MPL-authored
controls, not additional upstream inventory entries.

Two new controls change only the way that first value reaches the call:

```bend
# Direct: two competing failures in one application.
pair(overflow(), invalid())

# Indirect: a closure application makes the first call non-flat in upstream C.
# def apply(f: Unit -> Nat) -> Nat: f(Unit{})
pair(apply(u => overflow()), invalid())

# Bound: first complete IO.pure before entering the next continuation.
n : Nat <- IO.pure(Nat, overflow())
Unit <- IO.print(pair(n, invalid()))
```

Complete checked programs are in
[nat_first_indirect.bend](../tests/fixtures/cpu-failure-order/nat_first_indirect.bend)
and [nat_first_bound.bend](../tests/fixtures/cpu-failure-order/nat_first_bound.bend).
All four order controls exit **1**, stdout is exactly `before` without a newline,
and no `SECRET`, `kept` or `after` output appears. The exact diagnostic bytes are:

- **Nat:** `bend: a Nat past the largest immediate 2^48-1\n`
- **Request:** `bend: runtime fail-stop\n`

| Control | Native Rust | Port C | Port JS | Upstream JS | Upstream C, each of O0/O1/O2/O3 |
| --- | --- | --- | --- | --- | --- |
| `nat_first` | Nat | Nat | Nat | Nat | Request — unresolved difference |
| `request_first` | Request | Request | Request | Request | Request |
| `nat_first_indirect` | Nat | Nat | Nat | Nat | Nat |
| `nat_first_bound` | Nat | Nat | Nat | Nat | Nat |

Every available lane ran twice with identical streams/status; C used
`--threads 1 --gpu off`. The compiler was the existing Clang 19.1.7. No other
compiler/version, multiworker profile or GPU lane was tested here.

## Cause: C lowering, not an observed optimization-level change

The C compiler's [ANF pass](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1281)
hoists non-flat calls into continuation bindings but leaves
[flat calls](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L1462)
inline. Its [C body emitter](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L2401)
uses that pass. The direct control's generated `FID_MAIN_C12` jumps to
`FID_INVALID`; its Nat check is in a later `FID_MAIN_C14`. The latter is never
reached after the earlier failure. The indirect control instead jumps to
`FID_APPLY` before `FID_INVALID`. Exact generated sources and hashes are retained
in the receipt. Upstream's [JS emitter](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L2899)
does not use this C ANF path. CPU
[err_post/err_fail](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L3600)
flushes stdout, prints the encountered error and immediately exits.

The observed order therefore depends on Bend's backend/lowering and source call
shape. It does **not** vary across the four tested Clang optimization levels.
There is no evidence here to attribute this case to C's operand evaluation rules
or to a Clang optimization bug. Nor does one compiler establish universal
compiler independence. Preserve the port's current first-error behavior and keep
this as an unresolved cross-oracle requirement. A future decision needs an
explicit runtime-error contract or an upstream correction; neither a C-specific
reordering nor a new compatibility exception is implemented by this checkpoint.

## Separate Word oracle build limitation

The [Word-tail control](../tests/fixtures/cpu-demand-review/word_live_tail.bend)
has one invalid live field. Native, port C/JS and upstream JS all exit 1, emit no
stdout, and emit exactly `bend: runtime fail-stop\n` twice. Upstream C does the
same at **O1, O2 and O3**. At **O0**, generation succeeds but Clang exits 1 with
twelve `ran out of registers during register allocation` diagnostics; no program
execution is available in that profile.

Crucially, pinned [cli_build](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/main.ts#L253)
uses `-std=c11 -O3 <source> -lpthread -lm -o <binary>` for this Linux CPU program.
The earlier unoptimized comparison profile was a harness choice, **not the
upstream CLI default**. The prior O2 success remains historical evidence; this
checkpoint independently qualifies the CLI-equivalent O3 profile. O0 remains a
recorded oracle build limitation, not a language-runtime refusal or an execution
pass. No compiler flags in existing runners were silently changed. The source's
[preserve calling convention/musttail host segments](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L3244)
provide context for register pressure; identifying a precise Clang defect would
require a separate compiler investigation.

## Receipt and reproduction

[compare-failure-order.py](../scripts/compare-failure-order.py) uses only the
installed Python standard library and existing tools. It checks the clean pin,
snapshots the port binary, records exact commands, sources/generated hashes,
raw streams/status and compiler identity, and kills only its own child process
group if a 60-second deadline expires. It does not convert research completion
into parity acceptance.

```sh
python3 scripts/compare-failure-order.py --reference <reference> --output <fresh-ordering-dir>
```

The receipt contains **123 commands**: twenty successful generation commands,
24 successful C compilations, one O0 compilation failure, and **78 executions**.
Of those executions, eight reproduce the unresolved Nat-first C diagnostic
difference; seventy match their corresponding port control contract. Two Word O0
executions are unavailable. All 39 available repeated pairs are byte-stable;
there were no timeouts (longest child under 0.60 seconds). Runner exit 0 means
research completed; it does not waive its recorded failures.

Receipt SHA-256:
`ea5e7a41867a1deac7aa077eb1a69a92de69a004c435f629bcc8bb578ffaeedf`.
The [ordering ledger](cpu-failure-order-evidence.csv) records each profile and
keeps its authored origin explicit. The approved private-wildcard exception is
unchanged and does not cover these competing-error cases.
