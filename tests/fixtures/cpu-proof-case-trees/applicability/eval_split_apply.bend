# SPDX-License-Identifier: MPL-2.0
import Base
import ../eval_split_apply.bend as H
law main: @n: H.MN -> {H.wob(n) == H.add(n, H.Z{}) : H.MN}
def main(): H.main
