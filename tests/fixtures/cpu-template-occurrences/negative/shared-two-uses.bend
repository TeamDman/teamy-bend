# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
type Other is Data: O{}
type Pair is Data: P{a: N, b: Other}
def zero() -> N: Z{}
def app(~n: N) -> Pair:
  P{n,
    n}
def main() -> Pair: app(~zero)
