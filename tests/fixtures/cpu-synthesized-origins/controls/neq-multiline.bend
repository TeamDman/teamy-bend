# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
law bad: Type
def bad():
  {Z{}
   != Z{} : N}
