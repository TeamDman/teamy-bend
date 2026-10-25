# SPDX-License-Identifier: MPL-2.0
import Base
import ../imports/defs.bend as D
def wrapper(~x: U32) -> U32: D.missing(~x)
def main() -> U32: wrapper(~9)
