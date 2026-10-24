# SPDX-License-Identifier: MPL-2.0
import Base
import ../capture_price_twin_000.bend as H
def count(n: H.Nd) -> U32:
  match n:
    case H.Zd{}: 0
    case H.Sd{p}: (1 + count(p) : U32)
def main() -> List<U32>: [count(H.use(H.Zd{}, {==}, x => x)), count(H.use(H.Zd{}, {==}, x => H.Sd{x}))]
