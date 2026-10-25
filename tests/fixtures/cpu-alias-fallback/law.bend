# SPDX-License-Identifier: MPL-2.0
import Base
import original/defs.bend as D
law D.local: U32
def D.local(): 9
def main() -> U32: D.local()
