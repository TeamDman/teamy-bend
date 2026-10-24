# SPDX-License-Identifier: MPL-2.0
law observe: for f: Nat -> Nat
  Sigma<&1, &1, (Nat -> Nat), _ => (Nat -> Nat)>
def observe(
  f
): Tuple{f, f}
