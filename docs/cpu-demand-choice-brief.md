# Three executable-contract choices

**For review only. Nothing below changes runtime behavior.** The five open cases
are measured against Bend pin `e6676b080f25b1bc1bf5b5b7d7a17e22f8022599`, with
port candidate `6bfc10ac69cf523dd0fb1c302e92c9ee63e26362`. Each original case ran
twice across native Rust, port C/JS and upstream C/JS; all 50 results are unchanged.

## 1. An argument the function never uses

This complete [reproducer](../tests/fixtures/cpu-demand-review/unused_argument.bend)
passes an overflowing addition to a function that ignores its argument:

```bend
import Base
def limit() -> Nat: Nat.mul(U32.to_nat(16777215), U32.to_nat(16777217))
def discard(n: Nat) -> String: "kept"
def main() -> IO(Unit): IO.print(discard(Nat.add(limit(), 1n)))
```

Native prints `kept\n` and exits 0: it never computes the unused argument.
Both upstream compiled lanes and port C/JS compute it before the call, emit the
Nat-overflow diagnostic and exit 1. No compiled lane prints `kept`.

**Recommendation:** evaluate live arguments for executable parity, including
arguments the called function ignores. The pinned
[BendTT account](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/docs/BendTT/main.typ#L481)
describes weak by-value execution and a strict compiler. **Tradeoff:** native
programs relying on unused invalid expressions would start failing; additional
work can affect budgets/performance. Erased arguments and unselected branches
are separate and must not be evaluated merely to enforce this rule.

## 2. Inspecting just the outer constructor of an invalid Nat

This [reproducer](../tests/fixtures/cpu-demand-review/partial_double_overflow.bend)
asks only whether twice `2^47` is zero or positive:

```bend
import Base
def half() -> Nat: Nat.mul(U32.to_nat(8388608), U32.to_nat(16777216))
def head(n: Nat) -> String:
  match n:
    case 0n: "zero"
    case 1n+p: "positive"
def main() -> IO(Unit): IO.print(head(Nat.double(half())))
```

Native prints `positive\n`, exit 0, without finishing the Nat. All four compiled
lanes reject it. Inspecting `Nat.add(limit(), 1n)` has the same results. These
are cases two and three; unlike case one, the caller does inspect the result,
but only its first constructor.

**Recommendation:** a runtime Nat must satisfy the 48-bit bound before it is
observed as a value, even by this shallow match. The
[guide](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/guide/GUIDE.md#L111)
states the runtime bound; [add/double lowering](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/bend2/comp.ts#L254)
enforces it in both compiled lanes. **Tradeoff:** native inspection may demand
more of a tail and report errors sooner. This is a separate boundary from unused
function arguments; fixing only one leaves the other case unresolved. A bounded
design must preserve order, sharing and cancellation rather than globally force
every native field. Proof normalization needs its own contract.

## 3. A wildcard that swallows a private effect request

The original last two cases combine a lazy Nat tail with a wildcard match over
IO.OP. This additional [isolation control](../tests/fixtures/cpu-demand-review/forced_request_wildcard.bend)
forces that match, removing arithmetic laziness from the experiment:

```bend
import Base
def reject(op: IO.OP<Unit>) -> Nat:
  match op:
    case Emit{x}: 0n
    case _: 0n
def main() -> IO(Unit):
  IO.print(Nat.show(reject(IO.print("SECRET")(Unit, x => Emit{x}))))
```

Across ten repeated executions, upstream C alone prints `0\n` and exits 0.
Native Rust, port C/JS and upstream JS all emit `bend: runtime fail-stop\n` and
exit 1. **No lane prints SECRET.** This distinguishes swallowing a constructed
request from actually running its effect.

**Recommendation:** keep private requests unmatchable outside the IO driver,
including a wildcard. The pinned
[request regression](https://github.com/bendlang/bend/blob/e6676b080f25b1bc1bf5b5b7d7a17e22f8022599/tests/io/request_out_of_band.bend#L1)
explicitly requires refusal in every lane. JS guards the request before matching;
the emitted C wildcard instead sinks it and returns zero. **Tradeoff:** following
that stated rule preserves the port's refusal but cannot also reproduce this
pinned C anomaly byte-for-byte. A narrowly approved conflicting-oracle exception,
or an unresolved exact-C compatibility blocker, must remain explicit. No exception
has been approved or applied.

## All five original outcomes

Here `m = limit()`, `h = half()` and `invalid()` performs the wildcard match above.
Every successful cell has empty stderr and status 0; each printed word ends in
LF. Every failure cell has empty stdout and status 1.

| Case | Native | Upstream C | Upstream JS | Port C/JS |
| --- | --- | --- | --- | --- |
| `discard(Nat.add(m, 1n))` | `kept` | Nat failure | Nat failure | Nat failure |
| `head(Nat.add(m, 1n))` | `positive` | Nat failure | Nat failure | Nat failure |
| `head(Nat.double(h))` | `positive` | Nat failure | Nat failure | Nat failure |
| `head(Nat.add(U32.to_nat(4294967295), invalid()))` | `positive` | `positive` | Request failure | Request failure |
| `head(Nat.add(Succ{invalid()}, invalid()))` | `positive` | `positive` | Request failure | Request failure |

Nat failure is exactly `bend: a Nat past the largest immediate 2^48-1\n`.
Request failure is exactly `bend: runtime fail-stop\n`.
In the last two rows native avoids the invalid tail, whereas upstream C evaluates
the wildcard and accepts it. Identical output therefore has different causes;
the forced control above makes that distinction observable.

Accepting recommendations 1–2 would align these native executable cases with the
compiled target and change existing lazy behavior. Keeping native laziness is
possible as a separately named evaluator, but requires an explicit scope decision
and cannot silently count as executable parity. Recommendation 3 already matches
the port's forced-request behavior; its remaining decision concerns the conflicting
upstream oracle. No blanket runtime rewrite, test weakening or dependency change
is proposed. See the [full review](cpu-demand-contract.md) for sources, limitations
and the separate implementation review required before any semantic edit.
