# SPDX-License-Identifier: MPL-2.0
type Family<-A: Type> is Type:
  Item{value: A}
def bad() -> Type:
  Family
