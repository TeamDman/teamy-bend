# SPDX-License-Identifier: MPL-2.0
import Base
def twice(+n: U32, -ignored: U32) -> U32: U32.add(n, n)
def main() -> U32: twice(3, 8)
