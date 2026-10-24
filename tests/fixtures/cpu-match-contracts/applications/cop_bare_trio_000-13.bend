# SPDX-License-Identifier: MPL-2.0
import Base
import ../cop_bare_trio_000.bend as H
law main: H.Nat
def main(): H.Nat.add(H.Succ{H.Succ{H.Zero{}}}, H.Succ{H.Zero{}})
