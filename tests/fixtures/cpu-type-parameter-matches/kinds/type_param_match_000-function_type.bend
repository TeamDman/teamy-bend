# SPDX-License-Identifier: MPL-2.0
import Base
import ../type_param_match_000.bend as H
def main() -> H.List<H.Nat>: H.main((H.Nat -> H.Nat), H.Nil{}, H.Nil{})
