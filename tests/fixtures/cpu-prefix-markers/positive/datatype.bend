# SPDX-License-Identifier: MPL-2.0
import Base
def dupl(+xs: ++++List<Nat>) -> ++++List<Nat> & ++++List<Nat>:
  (xs, xs)
def main() -> ++++List<Nat> & ++++List<Nat>: dupl([1n, 2n])
