# SPDX-License-Identifier: MPL-2.0
import Base
import ../framed_cell_dep.bend as H
law witness: {0n == 1n : Nat}
def witness(): {==}
law main: Nat
def main(): H.dup_eq.go(1n, witness)
