# SPDX-License-Identifier: MPL-2.0
import Base
def id(~T: Type, x: T) -> T: x
def bad() -> U32:
  id(List, 0)
