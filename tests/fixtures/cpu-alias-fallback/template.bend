# SPDX-License-Identifier: MPL-2.0
import Base
import original/defs.bend as D
def D.local(~x: U32) -> U32: x
def main() -> U32: D.local(~9)
