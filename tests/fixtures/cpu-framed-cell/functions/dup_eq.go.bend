# SPDX-License-Identifier: MPL-2.0
import Base
import ../framed_cell_dep.bend as H
law main:
  for n: Nat
  for -e: {n == n : Nat}
  Nat
def main(): H.dup_eq.go
