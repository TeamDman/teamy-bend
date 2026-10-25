# SPDX-License-Identifier: MPL-2.0
import Base
def keep(~A: Type, x: ++++List<+List<A>>) -> ++++List<+List<A>>: x
def main() -> ++++List<+List<Nat>>: keep(~Nat, [[1n], [2n]])
