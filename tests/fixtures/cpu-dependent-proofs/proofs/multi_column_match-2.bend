# SPDX-License-Identifier: MPL-2.0
import Base
import ../multi_column_match.bend as H
law main: {H.and2(H.F{}, H.T{}) == H.and2(H.T{}, H.F{}) : H.Bit}
def main(): H.and_comm(H.F{}, H.T{})
