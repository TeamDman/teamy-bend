# SPDX-License-Identifier: MPL-2.0
def bad(f: Type -> Type) -> Type:
  g = f
  {@-x: g -> Type : Type}
def main() -> Type: Type
