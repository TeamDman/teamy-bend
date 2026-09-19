# SPDX-License-Identifier: MPL-2.0
type Nat is Data:
  Zero{}
  Succ{pred: Nat}

def identity(n: Nat) -> Nat:
  n

law identity_law:
  for n: Nat
  {identity(n) == n : Nat}

def identity_law(n):
  {==}

def main() -> Nat:
  identity(3n)
