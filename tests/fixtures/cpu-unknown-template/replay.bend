# SPDX-License-Identifier: MPL-2.0
import Base
import imports/defs.bend as D
def main() -> U32: U32.add(U32.add(D.pick(~9, 3), D.pick(~9, 4)), D.pick(~1, 2))
