# SPDX-License-Identifier: MPL-2.0
import Base
import ../printer_fixpoint.bend as H
def finish(text: Nat, effect: H.Eff) -> List<U32>:
  match effect:
    case H.EDone{value}: [1, U32.from_nat(text), 0, U32.from_nat(value)]
    case H.EPrint{next_text, cont}: [1, U32.from_nat(text), 1, U32.from_nat(next_text)]
def observe(effect: H.Eff, n: Nat) -> List<U32>:
  match effect:
    case H.EDone{value}: [0, U32.from_nat(value)]
    case H.EPrint{text, cont}: finish(text, cont(n))
def apply(n: Nat) -> List<U32>:
  observe(H.main(), n)
def main() -> List<List<U32>>:
  [apply(0n), apply(1n), apply(2n), apply(7n)]
