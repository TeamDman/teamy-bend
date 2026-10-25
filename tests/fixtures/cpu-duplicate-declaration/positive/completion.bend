# SPDX-License-Identifier: MPL-2.0
import Base
law value: U32
def value(): 3
def app(~f: U32 -> U32, x: U32) -> U32: f(x)
def inc(x: U32) -> U32: (x + 1 : U32)
def main() -> U32: (value() + app(~inc, 3) : U32)
