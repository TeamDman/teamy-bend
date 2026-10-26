# SPDX-License-Identifier: MPL-2.0
import Base
type Entry<-A: Data> is Data:
  Entry{value: A, words: List<&2, U32>}
def duplicate(-A: Data, r: Array<Entry<A>> & Entry<A>) -> Entry<A> & Entry<A>:
  (a, read) = r
  (read, read)
def main() -> Entry<U32> & Entry<U32>:
  duplicate(U32, Array.get(Entry<U32>, ALeaf{Entry{7, [10, 20]}}, 0))
