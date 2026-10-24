# SPDX-License-Identifier: MPL-2.0
def observe(f: Nat -> Nat) -> Sigma<&1, &1, (Nat -> Nat), _ => (Nat -> Nat)>: Tuple{f, f}
