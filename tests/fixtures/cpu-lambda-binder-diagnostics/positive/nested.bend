# SPDX-License-Identifier: MPL-2.0
import Base
def app(f: U32 -> U32 -> U32) -> U32: f(4, 9)
def main() -> U32: app(x => y => (x + y : U32))
