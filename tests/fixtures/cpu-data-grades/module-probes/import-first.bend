# SPDX-License-Identifier: MPL-2.0
import ../accept_data.bend as H
import Base
law main: H.Sigma<&2, &2, H.Nat, _ => H.Nat>
def main(): H.main
