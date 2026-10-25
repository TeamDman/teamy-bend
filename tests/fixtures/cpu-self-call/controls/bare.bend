# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
  S{p: N}
def loop(n: N) -> N:
  f = loop
  f(n)
