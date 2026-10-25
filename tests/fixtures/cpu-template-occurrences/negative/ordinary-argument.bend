# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
def ident(x: N) -> N: x
def app(~n: N) -> N:
  ident(n)
def main() -> N: app(~later)
def later() -> N: Z{}
