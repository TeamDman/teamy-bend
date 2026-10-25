# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{} S{p: N}
def app(~n: N) -> N:
  (x => S{x})(n)
def main() -> N: app(~later)
def later() -> N: Z{}
