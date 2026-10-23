# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
def main(e: {Z{} == Z{} : N}, f: {Z{} == Z{} : N}) -> N:
  %h@e : N
  %h@f : N
  Z{}
#|e => f => %h@e : N; %h@f : N; Z{}
