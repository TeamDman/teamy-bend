# SPDX-License-Identifier: MPL-2.0
import Base
import ../eval_case_tree_002.bend as H
law main: {H.getx(H.rot(H.Mk{H.Z{}, H.S{H.Z{}}, H.Z{}})) == H.Z{} : H.MN}
def main(): {==}
