# SPDX-License-Identifier: MPL-2.0
import Base
law bad:
  for n: Nat
  {n == 0n : Nat}
def bad(n):
  match n:
    case 0n: {==}
    case n: {==}
