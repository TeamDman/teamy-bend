# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
  S{p: N}
type K is Type:
  Box{n: N}
def main() -> N -> N:
  x => {\{Box: \{Z: Z{}; S: p => p}} : K -> N}(Box{x})
