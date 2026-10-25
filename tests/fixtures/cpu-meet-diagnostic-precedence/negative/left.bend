# SPDX-License-Identifier: MPL-2.0
import Base
law q: Quant
law r: Quant
law s: Quant
law wrong: {(q <&> r) <&> &1 == &1 : Quant}
def wrong(): {==}
def q(): &0
def r(): &1
def s(): &2
