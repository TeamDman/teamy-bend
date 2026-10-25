# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
def app(~n: N, x: N) -> N:
  match x:
    case n: Z{}
def main() -> N: app(~later, Z{})
def later() -> N: Z{}
