# SPDX-License-Identifier: MPL-2.0
def observe(n: Nat) -> (Nat -> Nat) -> Sigma<&1, &1, (Nat -> Nat), _ => (Nat -> Nat)>:
  f => Tuple{f, f}
