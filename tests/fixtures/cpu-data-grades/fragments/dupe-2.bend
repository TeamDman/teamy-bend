# SPDX-License-Identifier: MPL-2.0
law observe: Sigma<&2, &2, {Succ{Succ{Zero{}}} == Succ{Succ{Zero{}}} : Nat}, _ => {Succ{Succ{Zero{}}} == Succ{Succ{Zero{}}} : Nat}>
def observe(): dupe(Succ{Succ{Zero{}}}, Succ{Succ{Zero{}}}, {==})
