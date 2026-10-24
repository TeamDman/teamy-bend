# SPDX-License-Identifier: MPL-2.0
law observe: Sigma<&2, &2, {Succ{Succ{Succ{Succ{Zero{}}}}} == Succ{Succ{Succ{Succ{Zero{}}}}} : Nat}, _ => {Succ{Succ{Succ{Succ{Zero{}}}}} == Succ{Succ{Succ{Succ{Zero{}}}}} : Nat}>
def observe(): dupe(Succ{Succ{Succ{Succ{Zero{}}}}}, Succ{Succ{Succ{Succ{Zero{}}}}}, {==})
