# SPDX-License-Identifier: MPL-2.0
import Base
law q: Quant
law wrong: {&1 == q <&> &1 : Quant}
def wrong(): {==}
def q(): &0
