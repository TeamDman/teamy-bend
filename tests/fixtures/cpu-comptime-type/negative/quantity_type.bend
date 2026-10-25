# SPDX-License-Identifier: MPL-2.0
import Base
def app(~f: Type -> U32) -> U32: f(Quant)
def main() -> U32: app(~(x => U32.inc(x)))
