# SPDX-License-Identifier: MPL-2.0
type Box is Data: Full{}
def Box.pure(x: Box) -> Box: x
def Box.bind(-A: Type, x: A, f: @v:A -> Box) -> Box: f(x)
law main: Box
def main():
  do Box<>:
    x : Box <- Full{}
    return x
