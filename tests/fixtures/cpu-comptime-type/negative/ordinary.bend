# SPDX-License-Identifier: MPL-2.0
import Base
def bad(x: Nat) -> Bool: Bool.not(x)
def main() -> Bool: bad(3n)
