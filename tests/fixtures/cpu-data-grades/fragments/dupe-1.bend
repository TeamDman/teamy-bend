# SPDX-License-Identifier: MPL-2.0
law observe: Sigma<&2, &2, {Succ{Zero{}} == Succ{Zero{}} : Nat}, _ => {Succ{Zero{}} == Succ{Zero{}} : Nat}>
def observe(): dupe(Succ{Zero{}}, Succ{Zero{}}, {==})
