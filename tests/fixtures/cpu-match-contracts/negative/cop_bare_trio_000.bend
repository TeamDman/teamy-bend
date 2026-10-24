# SPDX-License-Identifier: MPL-2.0
import Base
import ../cop_bare_trio_000.bend as H
law main: {H.addself(H.Succ{H.Succ{H.Zero{}}}) == H.Succ{H.Succ{H.Zero{}}} : H.Nat}
def main(): {==}
