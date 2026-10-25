# SPDX-License-Identifier: MPL-2.0
import Base
import defs.bend as D
import ./defs.bend as Again
def D.value(): 3
def main() -> U32: (D.value() + Again.value() : U32)
