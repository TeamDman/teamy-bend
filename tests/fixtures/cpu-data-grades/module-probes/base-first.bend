# SPDX-License-Identifier: MPL-2.0
import Base
import ../accept_data.bend as H
law main: H.Sigma<&2, &2, H.Nat, _ => H.Nat>
def main(): H.main
