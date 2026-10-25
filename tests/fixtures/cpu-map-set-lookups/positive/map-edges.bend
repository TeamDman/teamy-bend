# SPDX-License-Identifier: MPL-2.0
import Base
import ../original/map_set_ops.bend as M

def nul() -> String: SCon{Chr{0}, SNil{}}
def edge() -> Map<&2, U32>: Map.from_list(&2, U32, [("", 1), (nul(), 2)])
def value(r: Map<&2, U32> & U32) -> U32:
  (m, x) = r
  x
def twice.finish(first: Maybe<&2, U32>, r: Map<&2, U32> & Maybe<&2, U32>) -> U32:
  (m, second) = r
  U32.mul(M.got(first, 1), M.is_none(second))
def twice.start(r: Map<&2, U32> & Maybe<&2, U32>) -> U32:
  (m, first) = r
  twice.finish(first, Map.pop(&2, U32, m, ""))
def twice() -> U32: twice.start(Map.pop(&2, U32, edge(), ""))
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.print(U32.show(value(Map.get(U32, 99, Map.new(&2, U32), ""))))
    Unit <- IO.print(U32.show(value(Map.get(U32, 99, edge(), ""))))
    Unit <- IO.print(U32.show(value(Map.get(U32, 99, edge(), nul()))))
    Unit <- IO.print(U32.show(value(Map.get(U32, 99, Map.set(&2, U32, edge(), "", 9), ""))))
    Unit <- IO.print(U32.show(value(Map.get(U32, 99, Map.set(&2, U32, edge(), "", 9), nul()))))
    Unit <- IO.print(U32.show(value(Map.get(U32, 99, Map.del(&2, U32, edge(), ""), nul()))))
    Unit <- IO.print(U32.show(value(Map.get(U32, 99, Map.del(&2, U32, edge(), nul()), ""))))
    Unit <- IO.print(U32.show(twice()))
    Unit <- IO.print(M.Str.join(Map.keys(&2, U32, Map.from_list(&2, U32, [("🙂", 3), ("é", 2), ("", 1)]))))
    Unit <- IO.print(U32.show(U32.from_nat(Set.size(Set.from_list(["", "é", "é"])))))
    IO.print(M.Str.join(Set.to_list(Set.del(Set.from_list(["", "é", "é"]), ""))))
