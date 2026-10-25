# SPDX-License-Identifier: MPL-2.0
import Base
import ../imports/typed.bend as D
def wrap(~n: U32) -> U32: D.pick(~n, 3)
def main() -> U32: U32.add(wrap(~9), wrap(~9))
