# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
def Alias() -> Type:
  N
type Bad is Alias: C{}
