# SPDX-License-Identifier: MPL-2.0
import Base
import ../list_ctor_001.bend as H
def scalar(n: H.Nat) -> U32:
  match n:
    case H.Z{}: 0
    case H.S{p}: U32.add(1, scalar(p))

def values(xs: H.List<&2, H.Nat>) -> List<U32>:
  match xs:
    case H.Nil{}: []
    case H.Cons{h, t}: Con{scalar(h), values(t)}

def observe(value: H.R) -> List<List<U32>>:
  match value:
    case H.A{a, b}: [[0], [scalar(a)], values(b)]
    case H.B{a}: [[1], values(a)]
    case H.C{}: [[2]]

def main() -> List<List<List<U32>>>:
  [observe(H.main(H.Nil{})),
   observe(H.main(H.Cons{H.Z{}, H.Nil{}})),
   observe(H.main(H.Cons{H.S{H.S{H.Z{}}}, H.Nil{}})),
   observe(H.main(H.Cons{H.Z{}, H.Cons{H.Z{}, H.Nil{}}})),
   observe(H.main(H.Cons{H.Z{}, H.Cons{H.S{H.S{H.Z{}}}, H.Nil{}}})),
   observe(H.main(H.Cons{H.S{H.S{H.Z{}}}, H.Cons{H.Z{}, H.Nil{}}})),
   observe(H.main(H.Cons{H.S{H.S{H.Z{}}}, H.Cons{H.S{H.S{H.Z{}}}, H.Nil{}}}))]
