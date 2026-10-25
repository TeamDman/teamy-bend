# SPDX-License-Identifier: MPL-2.0
import Base
def field(~T: Type, x: T) -> T: x
def bad() -> U32:
  field(Missing<U32>, 0)
