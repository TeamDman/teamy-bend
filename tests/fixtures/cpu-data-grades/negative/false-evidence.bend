# SPDX-License-Identifier: MPL-2.0
law observe: Sigma<&2, &2, {Zero{} == Succ{Zero{}} : Nat}, _ => {Zero{} == Succ{Zero{}} : Nat}>
def observe(): dupe(Zero{}, Succ{Zero{}}, {==})
