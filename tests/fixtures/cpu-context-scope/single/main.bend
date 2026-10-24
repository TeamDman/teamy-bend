# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
  S{p: N}
law bad:
  for x: N
  N -> {x == S{x} : N}
def bad(x):
  x => {==}
