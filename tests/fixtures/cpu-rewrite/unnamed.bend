# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
def main(e: {Z{} == Z{} : N}) -> N:
  %e : N
  Z{}
#|e => %e : N; Z{}
