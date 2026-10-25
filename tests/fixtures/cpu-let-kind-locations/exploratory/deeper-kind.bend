# SPDX-License-Identifier: MPL-2.0
type Box<-A: Type> is Type: B{v: A}
def bad(f: Type -> Type) -> Type:
  +g = {x => x : @-x: f -> Type}
  Type
def main() -> Type: Type
