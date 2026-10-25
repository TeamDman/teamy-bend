# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
type Four is Data: Four{a: N, b: N, c: N, d: N}
def pack(~n: N) -> Four:
  Four{Z{}, Z{}, Z{}, n}
def main() -> Four: pack(~later)
def later() -> N: Z{}
