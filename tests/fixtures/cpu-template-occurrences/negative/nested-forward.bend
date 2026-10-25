# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
def inner(~f: N -> N, x: N) -> N: f(x)
def outer(~f: N -> N, x: N) -> N:
  inner(~f, x)
def main() -> N: outer(~later, Z{})
def later(x: N) -> N: x
