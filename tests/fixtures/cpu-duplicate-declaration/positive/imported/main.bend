# SPDX-License-Identifier: MPL-2.0
import Base
import left.bend as L
import right.bend as R
import left.bend as Again
def L.value(): 3
def main() -> U32: (L.value() + R.value() : U32)
