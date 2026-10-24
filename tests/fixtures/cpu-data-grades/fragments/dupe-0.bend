# SPDX-License-Identifier: MPL-2.0
law observe: Sigma<&2, &2, {Zero{} == Zero{} : Nat}, _ => {Zero{} == Zero{} : Nat}>
def observe(): dupe(Zero{}, Zero{}, {==})
