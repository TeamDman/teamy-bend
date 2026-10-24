# SPDX-License-Identifier: MPL-2.0
import Base
import ../eval_split_apply.bend as H
law main: {H.wob(H.S{H.S{H.S{H.S{H.Z{}}}}}) == H.add(H.S{H.S{H.S{H.S{H.Z{}}}}}, H.Z{}) : H.MN}
def main(): H.main(H.S{H.S{H.S{H.S{H.Z{}}}}})
