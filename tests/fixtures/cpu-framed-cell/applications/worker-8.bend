# SPDX-License-Identifier: MPL-2.0
import Base
import ../framed_cell_dep.bend as H
law main: Nat
def main(): H.dup_eq.go(8n, {==})
