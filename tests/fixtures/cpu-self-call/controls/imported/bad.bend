# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
  S{p: N}
def bad(n: N) -> N:
  bad(n)
