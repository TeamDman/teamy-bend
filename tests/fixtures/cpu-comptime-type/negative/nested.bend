# SPDX-License-Identifier: MPL-2.0
import Base
def app(~f: Nat -> Nat, x: Nat) -> Nat: f(x)
def wrap(~f: Nat -> Nat, x: Nat) -> Nat: app(~f, x)
def main() -> Nat: wrap(~Bool.not, 3n)
