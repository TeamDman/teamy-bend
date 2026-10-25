# SPDX-License-Identifier: MPL-2.0
import Base
def as(x: U32) -> U32: x
def match_value(x: U32) -> U32: (x + 1 : U32)
def main() -> U32: as(match_value(6))
