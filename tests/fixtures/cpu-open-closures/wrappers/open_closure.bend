# SPDX-License-Identifier: MPL-2.0
import Base
import ../open_closure.bend as H
def identity(x: U32) -> U32: x
def increment(x: U32) -> U32: (x + 1 : U32)
def double(x: U32) -> U32: (x * 2 : U32)
def apply(f: U32 -> U32, g: U32 -> U32, x: U32) -> U32:
  H.pipe(f, g)(x)
def main() -> List<U32>:
  [apply(identity, identity, 0), apply(identity, identity, 5), apply(identity, identity, 4294967295), apply(identity, increment, 0), apply(identity, increment, 5), apply(identity, increment, 4294967295), apply(identity, double, 0), apply(identity, double, 5), apply(identity, double, 4294967295), apply(increment, identity, 0), apply(increment, identity, 5), apply(increment, identity, 4294967295), apply(increment, increment, 0), apply(increment, increment, 5), apply(increment, increment, 4294967295), apply(increment, double, 0), apply(increment, double, 5), apply(increment, double, 4294967295), apply(double, identity, 0), apply(double, identity, 5), apply(double, identity, 4294967295), apply(double, increment, 0), apply(double, increment, 5), apply(double, increment, 4294967295), apply(double, double, 0), apply(double, double, 5), apply(double, double, 4294967295)]
