# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{} S{p: N}
type Result is Data: R{a: N, b: N, c: N}
def ident(x: N) -> N: x
def inner(~f: N -> N, x: N) -> N: f(x)
def outer(~f: N -> N, x: N) -> N: inner(~f, x)
def macro(~f: N -> N, x: N) -> N: f(x)
def main() -> Result:
  R{outer(~ident, Z{}), macro(~(x => S{x}), Z{}), outer(~ident, S{Z{}})}
