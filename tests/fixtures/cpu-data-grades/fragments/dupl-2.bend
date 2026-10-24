# SPDX-License-Identifier: MPL-2.0
law observe: Sigma<&2, &2, List<&2, Nat>, _ => List<&2, Nat>>
def observe(): dupl(Con{Succ{Zero{}}, Con{Succ{Succ{Zero{}}}, Nil{}}})
