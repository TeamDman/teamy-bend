# SPDX-License-Identifier: MPL-2.0
import Base
import ../list_ctor_000.bend as H
def scalar(n: H.Bit) -> U32:
  match n:
    case H.F{}: 0
    case H.T{}: 1

def values(xs: H.List<&2, H.Bit>) -> List<U32>:
  match xs:
    case H.Nil{}: []
    case H.Cons{h, t}: Con{scalar(h), values(t)}

def observe(value: H.R) -> List<List<U32>>:
  match value:
    case H.A{a}: [[0], values(a)]
    case H.B{a}: [[1], values(a)]
    case H.C{}: [[2]]

def main() -> List<List<List<U32>>>:
  [observe(H.main(H.Nil{})),
   observe(H.main(H.Cons{H.F{}, H.Nil{}})),
   observe(H.main(H.Cons{H.T{}, H.Nil{}})),
   observe(H.main(H.Cons{H.F{}, H.Cons{H.F{}, H.Nil{}}})),
   observe(H.main(H.Cons{H.F{}, H.Cons{H.T{}, H.Nil{}}})),
   observe(H.main(H.Cons{H.T{}, H.Cons{H.F{}, H.Nil{}}})),
   observe(H.main(H.Cons{H.T{}, H.Cons{H.T{}, H.Nil{}}}))]
