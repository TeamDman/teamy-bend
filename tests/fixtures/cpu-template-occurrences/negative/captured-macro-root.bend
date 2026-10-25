# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
def inner(~f: N -> N) -> N: f(Z{})
def outer(~n: N) -> N:
  inner(~(x => n))
def main() -> N: outer(~later)
def later() -> N: Z{}
