# SPDX-License-Identifier: MPL-2.0
import Base
import ../dependent_telescope.bend as H
def count(n: H.Nq) -> U32:
  match n:
    case H.Zq{}: 0
    case H.Sq{p}: (1 + count(p) : U32)
def main() -> List<U32>: [count(H.total(H.Mq{H.Sq{H.Zq{}},H.Zq{},H.Zq{},H.Zq{},H.Zq{},H.Zq{},H.Zq{},{==}})), count(H.total(H.Mq{H.Sq{H.Zq{}},H.Sq{H.Zq{}},H.Sq{H.Zq{}},H.Sq{H.Zq{}},H.Sq{H.Zq{}},H.Sq{H.Zq{}},H.Sq{H.Zq{}},{==}})), count(H.total(H.Mq{H.Sq{H.Zq{}},H.Zq{},H.Sq{H.Zq{}},H.Sq{H.Sq{H.Zq{}}},H.Sq{H.Sq{H.Sq{H.Zq{}}}},H.Sq{H.Sq{H.Sq{H.Sq{H.Zq{}}}}},H.Zq{},{==}})), count(H.total(H.Mq{H.Sq{H.Zq{}},H.Sq{H.Sq{H.Sq{H.Sq{H.Zq{}}}}},H.Sq{H.Sq{H.Sq{H.Zq{}}}},H.Sq{H.Sq{H.Zq{}}},H.Sq{H.Zq{}},H.Zq{},H.Sq{H.Sq{H.Sq{H.Sq{H.Zq{}}}}},{==}}))]
