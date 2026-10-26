# SPDX-License-Identifier: MPL-2.0
import Base
def quote(~n: Nat) -> Char: '\''
def main() -> List<Char>: [quote(~0n), quote(~1n), quote(~0n)]
