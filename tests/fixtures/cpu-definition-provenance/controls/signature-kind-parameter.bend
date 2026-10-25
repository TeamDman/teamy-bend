# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
type Bad<-A: Type> is N:
  C{x: A}
