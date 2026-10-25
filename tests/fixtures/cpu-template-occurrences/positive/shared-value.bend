# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
type Result is Data: R{a: N, b: N, c: N}
def zero() -> N: Z{}
def pack(~n: N) -> Result:
  R{n,
    n,
    n}
def main() -> Result: pack(~zero)
