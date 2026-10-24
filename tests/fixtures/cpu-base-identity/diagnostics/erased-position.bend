# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
law ignore:
  for -A: Type
  N
def ignore(A): Z{}
law bad: N
def bad(): ignore(Missing)
