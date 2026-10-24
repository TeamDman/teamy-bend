# SPDX-License-Identifier: MPL-2.0
import Base
import ../multi_column_match.bend as H
law main: {H.and2(H.T{}, H.F{}) == H.and2(H.F{}, H.T{}) : H.Bit}
def main(): H.and_comm(H.T{}, H.F{})
