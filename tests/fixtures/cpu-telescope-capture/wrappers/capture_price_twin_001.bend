# SPDX-License-Identifier: MPL-2.0
import Base
import ../capture_price_twin_001.bend as H
def count(n: H.Ne) -> U32:
  match n:
    case H.Ze{}: 0
    case H.Se{p}: (1 + count(p) : U32)
def main() -> List<U32>: [count(H.use(H.Ze{}, {==}, x => x)), count(H.use(H.Ze{}, {==}, x => H.Se{x})), count(H.fe(H.Ze{})), count(H.use.go(x => x, H.Ze{})), count(H.use.go(x => H.Se{x}, H.Ze{})), count(H.fe(H.Se{H.Ze{}})), count(H.use.go(x => x, H.Se{H.Ze{}})), count(H.use.go(x => H.Se{x}, H.Se{H.Ze{}})), count(H.fe(H.Se{H.Se{H.Ze{}}})), count(H.use.go(x => x, H.Se{H.Se{H.Ze{}}})), count(H.use.go(x => H.Se{x}, H.Se{H.Se{H.Ze{}}})), count(H.fe(H.Se{H.Se{H.Se{H.Se{H.Ze{}}}}})), count(H.use.go(x => x, H.Se{H.Se{H.Se{H.Se{H.Ze{}}}}})), count(H.use.go(x => H.Se{x}, H.Se{H.Se{H.Se{H.Se{H.Ze{}}}}}))]
