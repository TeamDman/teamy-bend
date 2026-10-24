# SPDX-License-Identifier: MPL-2.0
import Base
import ../eval_case_tree_001.bend as H
law main: {H.turn(H.turn(H.Up{})) == H.Up{} : H.Dir}
def main(): {==}
