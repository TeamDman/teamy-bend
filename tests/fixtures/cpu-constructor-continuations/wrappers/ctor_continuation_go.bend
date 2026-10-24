# SPDX-License-Identifier: MPL-2.0
import Base
import ../ctor_continuation_go.bend as H
def from_nat(n: Nat) -> H.W:
  match n:
    case 0n: H.Z{}
    case 1n+p: H.S{from_nat(p)}
def count(n: H.W) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: (1 + count(p) : U32)
def finish(text: H.W, effect: H.Io) -> List<U32>:
  match effect:
    case H.Done{value}: [2, count(text), 0, count(value)]
    case H.Tick{cont}: [2, count(text), 1]
    case H.Print{next_text, cont}: [2, count(text), 2, count(next_text)]
def tick(text: H.W, effect: H.Io, m: H.W) -> List<U32>:
  match effect:
    case H.Done{value}: [1, count(text), 0, count(value)]
    case H.Tick{cont}: finish(text, cont(m))
    case H.Print{next_text, cont}: [1, count(text), 2, count(next_text)]
def observe(effect: H.Io, n: H.W, m: H.W) -> List<U32>:
  match effect:
    case H.Done{value}: [0, count(value)]
    case H.Tick{cont}: finish(H.Z{}, cont(n))
    case H.Print{text, cont}: tick(text, cont(n), m)
def direct(x: H.W) -> H.Io: H.Done{x}
def increment(x: H.W) -> H.Io: H.Done{H.S{x}}
def residual(x: H.W) -> H.Io: H.Print{x, n => H.Tick{m => H.Done{H.S{n}}}}
def apply(n: Nat, m: Nat) -> List<U32>: observe(H.main(residual), from_nat(n), from_nat(m))
def main() -> List<List<U32>>:
  [observe(H.main(direct), H.Z{}, H.Z{}), observe(H.main(increment), H.Z{}, H.Z{}), apply(0n, 0n), apply(0n, 2n), apply(1n, 0n), apply(1n, 2n), apply(4n, 0n), apply(4n, 2n)]
