# SPDX-License-Identifier: MPL-2.0
type N is Data:
  Z{}
type Pair<-A: Type> is Type:
  Pair{left: A, right: A}
def duplicate(f: N -> N) -> Pair<(N -> N)>:
  Pair{f, f}
