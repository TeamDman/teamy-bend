# SPDX-License-Identifier: MPL-2.0
import Base
import ../eval_case_tree_001.bend as H
law main: @d: H.Dir -> {H.turn(H.turn(d)) == H.back(d) : H.Dir}
def main(): H.main
