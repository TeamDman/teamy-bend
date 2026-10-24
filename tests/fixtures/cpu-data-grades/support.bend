# SPDX-License-Identifier: MPL-2.0
# Each affine closure is consumed once; the Nat input is explicitly reusable.
def apply_fns(xs: List<&1, Nat -> Nat>, +n: Nat) -> List<&2, Nat>:
  match xs:
    case Nil{}: Nil{}
    case Con{f, rest}: Con{f(n), apply_fns(rest, n)}
def apply_mixed(p: Sigma<&2, &1, Nat, _ => (Nat -> Nat)>, n: Nat) -> Sigma<&2, &2, Nat, _ => Nat>:
  match p:
    case Tuple{value, f}: Tuple{value, f(n)}
