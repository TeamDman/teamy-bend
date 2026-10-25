# SPDX-License-Identifier: MPL-2.0
import Base
import ../imports/defs.bend as D
def main() -> Nat: Nat.add(D.app(~Nat.add(1n), 3n), D.app(~Nat.add(1n), 3n))
