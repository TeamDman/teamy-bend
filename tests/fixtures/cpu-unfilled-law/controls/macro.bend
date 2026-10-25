# SPDX-License-Identifier: MPL-2.0
import Base
def choose(~x: U32) -> U32: x
law ghost: U32
def bad() -> U32: choose(~ghost)
def ghost(): 3
def main() -> U32: bad()
