# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
def app(~n: N) -> N:
  (x => n)(Z{})
def main() -> N: app(~later)
def later() -> N: Z{}
