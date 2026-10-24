# SPDX-License-Identifier: MPL-2.0
import Base
import ../shared_scrutinee.bend as H
law main: {H.add(H.S{H.Z{}}, H.Z{}) == H.Z{} : H.N}
def main(): {==}
