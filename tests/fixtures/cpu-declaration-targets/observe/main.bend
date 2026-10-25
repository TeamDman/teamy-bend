# SPDX-License-Identifier: MPL-2.0
import Base
import ../completed/main.bend as A
def main() -> U32: (A.main() + A.D.value() : U32)
