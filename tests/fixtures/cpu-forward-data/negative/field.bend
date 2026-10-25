# SPDX-License-Identifier: MPL-2.0
import Base
type Wrapper is Type:
  Wrapped{value: Later<U32>}
type Later<-A: Type> is Type:
  LaterValue{value: A}
