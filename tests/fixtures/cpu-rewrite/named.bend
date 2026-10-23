# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
def main(a: N, b: N, e: {a == b : N}) -> {b == a : N}:
  %h@e : {_ == a : N}
  {==}
#|a => b => e => %h@e : {_ == a : N}; {==}
