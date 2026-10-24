# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
  S{p: N}
type K is Type:
  Box{n: N}
def g(n: N) -> N:
  match n:
    case Z{}: Z{}
    case S{p}: p
def f(k: K) -> N:
  match k:
    case Box{n}: g(n)
def main() -> N -> N:
  x => f(Box{x})
