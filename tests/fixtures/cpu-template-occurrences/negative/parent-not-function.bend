# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
def zero() -> N: Z{}
def app(~n: N) -> N:
  n(Z{})
def main() -> N: app(~zero)
