# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
type Empty is Type:
def Exists(A: Type, -B: @-x:A -> Type) -> Type: A
law witness:
  exs x: N
  N
def witness(): Z{}
law inline:
  &x:N -> N
def inline(): Z{}
law constrained:
  for x: N where N
  N
def constrained(x): x
law neq_type: Type
def neq_type(): {Z{} != Z{} : N}
law main: N
def main(): constrained(witness())
