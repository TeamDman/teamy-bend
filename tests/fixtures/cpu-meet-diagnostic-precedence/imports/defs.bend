# SPDX-License-Identifier: MPL-2.0
import Base
law q: Quant
law wrong: {q <&> &1 == &1 : Quant}
def wrong(): {==}
def q(): &0
