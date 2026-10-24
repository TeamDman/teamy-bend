# SPDX-License-Identifier: MPL-2.0
import Base
import ../partial_app_snf.bend as H
def apply(f: H.Trm, x: H.Trm) -> H.Trm:
  match f:
    case H.TLam{bod}: bod(x)
    case f: f

def number(t: H.Trm) -> U32:
  match t:
    case H.TNum{v}: v
    case t: 999

def observe(t: H.Trm) -> List<U32>:
  match t:
    case H.TApp{fun, arg}: [1, number(fun), number(arg)]
    case H.TNum{v}: [0, v]
    case H.TLam{bod}: [2]

def main() -> List<U32>:
  observe(apply(apply(H.main(), H.TNum{7}), H.TNum{9}))
