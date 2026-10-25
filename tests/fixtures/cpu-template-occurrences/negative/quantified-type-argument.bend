# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
type Box<g, -A: Kind(g)> is Kind(g): B{x: A}
def app(~T: Type) -> +Box<&1, T>:
  B{Z{}}
def main() -> +Box<&1, N>: app(~Later)
type Later is Data: ZL{}
