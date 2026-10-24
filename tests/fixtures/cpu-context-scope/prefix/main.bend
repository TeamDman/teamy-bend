# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
  S{p: N}
law bad:
  for x: N
  for h: {x == S{x} : N}
  N -> N -> {x == S{x} : N}
def bad(x, h):
  x => x => {==}
