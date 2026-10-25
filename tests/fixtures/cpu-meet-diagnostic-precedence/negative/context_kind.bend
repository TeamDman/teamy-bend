# SPDX-License-Identifier: MPL-2.0
import Base
law wrong:
  for -q: Quant
  for -T: Kind(q <&> &1)
  {&0 == &1 : Quant}
def wrong(q, T): {==}
