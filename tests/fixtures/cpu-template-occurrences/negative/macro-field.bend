# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{} S{p: N}
def app(~macro: N -> N, ~n: N) -> N:
  macro(n)
def main() -> N: app(~(x => S{x}), ~later)
def later() -> N: Z{}
