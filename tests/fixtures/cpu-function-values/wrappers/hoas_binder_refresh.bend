# SPDX-License-Identifier: MPL-2.0
import Base
import ../hoas_binder_refresh.bend as H
def apply(a: U32, b: U32) -> U32:
  H.main()(a)(b)
def main() -> List<U32>:
  [apply(0, 0), apply(0, 7), apply(0, 4294967295), apply(7, 0), apply(7, 7), apply(7, 4294967295), apply(4294967295, 0), apply(4294967295, 7), apply(4294967295, 4294967295)]
