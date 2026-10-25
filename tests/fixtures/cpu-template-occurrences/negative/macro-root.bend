# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
def app(~macro: N -> N, ~n: N) -> N:
  macro(n)
def main() -> N: app(~(x => x), ~later)
def later() -> N: Z{}
