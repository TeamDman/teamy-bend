# SPDX-License-Identifier: MPL-2.0
type Observed is Type:
  Observed{field0: List<&2, Nat>, field1: List<&2, Nat>, field2: List<&2, Nat>, field3: List<&2, Nat>, field4: Sigma<&2, &2, Nat, _ => Nat>, field5: Sigma<&2, &2, Nat, _ => Nat>, field6: Sigma<&2, &2, Nat, _ => Nat>, field7: Sigma<&2, &2, Nat, _ => Nat>}
law observe: Observed
def observe(): Observed{apply_fns(fns, Zero{}), apply_fns(fns, Succ{Zero{}}), apply_fns(fns, Succ{Succ{Zero{}}}), apply_fns(fns, Succ{Succ{Succ{Succ{Zero{}}}}}), apply_mixed(mixed, Zero{}), apply_mixed(mixed, Succ{Zero{}}), apply_mixed(mixed, Succ{Succ{Zero{}}}), apply_mixed(mixed, Succ{Succ{Succ{Succ{Zero{}}}}})}
