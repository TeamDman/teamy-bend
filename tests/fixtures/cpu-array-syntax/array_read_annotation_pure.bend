# SPDX-License-Identifier: Apache-2.0
# Derived from the pinned array_read_annotation.bend; original helpers unchanged.
# Replace only the IO main with a pure main.out entry for separate qualification.
# an array read is an ordinary term: it takes an ascription like any
# other (bend3 pinned annotation placement on its slab reads)
import Base

law main.out:
  U32

law main.rd:
  for r: Array<U32> & U32
  U32

def main.rd(r):
  (a2, x) = r
  x

def main.out():
  main.rd({[4 : U32*2n][0] : Array<U32> & U32})

def main() -> U32:
  main.out()

#|4
