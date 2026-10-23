# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
def main(a: N, b: N, e: {a == b : N}) -> {e == e : {a == b : N}}:
  %e@e : {e == e : {a == _ : N}}
  {==}
#|a => b => e => %e@e : {e == e : {a == _ : N}}; {==}
