# SPDX-License-Identifier: MPL-2.0
import Base
import ../open/defs.bend as D
law D.value: U32
def D.value(): 3
def main() -> U32: D.value()
