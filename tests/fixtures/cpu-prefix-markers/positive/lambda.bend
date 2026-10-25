# SPDX-License-Identifier: MPL-2.0
import Base
def twice() -> Nat -> Nat: ++++n => Nat.add(n, n)
def main() -> Nat: twice()(3n)
