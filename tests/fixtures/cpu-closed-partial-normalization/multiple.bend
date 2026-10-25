# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
  S{p: N}
def pick(a: N, b: N, c: N) -> N: c
def main() -> N: {pick(Z{}) : N -> N -> N}(Z{}, S{Z{}})
