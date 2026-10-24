# SPDX-License-Identifier: MPL-2.0
import Base
import ../eval_case_tree_002.bend as H
law main: @p: H.P3 -> {H.getx(H.rot(p)) == H.gety(p) : H.MN}
def main(): H.main
