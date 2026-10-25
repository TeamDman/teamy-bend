# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
def app(~f: N -> N, x: N) -> N:
  (
    f
  )(x)
def main() -> N: app(~later, Z{})
def later(x: N) -> N: x
