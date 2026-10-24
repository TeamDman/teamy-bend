# SPDX-License-Identifier: MPL-2.0
import Base
import ../type_param_match_001.bend as H
def main() -> H.R: H.main((H.Nat -> H.Nat), H.Z{}, H.Z{}, H.Z{})
