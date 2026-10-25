# SPDX-License-Identifier: MPL-2.0
type Container<-A: Type> is Type:
  Contained{first: A, second: Missing<A>}
