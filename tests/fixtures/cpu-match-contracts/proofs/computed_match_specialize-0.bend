# SPDX-License-Identifier: MPL-2.0
import Base
import ../computed_match_specialize.bend as H
law main: {H.Zf{} == H.Zf{} : H.Nf}
def main(): H.p_var(H.Zf{}, {==})
