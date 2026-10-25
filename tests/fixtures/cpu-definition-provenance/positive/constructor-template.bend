# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
type Wrap<-A: Data> is Data:
  W{x: A}
def ident(x: N) -> N: x
def apply(~f: N -> N, x: N) -> N: f(x)
def main() -> Wrap<N>: W{apply(~ident, Z{})}
