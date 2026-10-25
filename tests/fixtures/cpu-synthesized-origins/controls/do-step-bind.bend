# SPDX-License-Identifier: MPL-2.0
import Base
type Box is Data: Mk{v: U32}
law main: Box
def main():
  do Box<>:
    Mk{3}
    return 3
