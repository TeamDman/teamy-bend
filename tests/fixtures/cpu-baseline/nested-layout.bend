import Base
type Leaf is Data:
  Number{value: U32}
  Caption{text: String, amount: U32}
type Branch is Data: Branch{leaf: Leaf, count: U32}
def finish(pair: Array<Branch> & Branch) -> Branch:
  (array, branch) = pair
  branch
def roundtrip() -> Branch:
  array = Array.new(Branch, 1n, Branch{Caption{"x", 7}, 9})
  finish(Array.get(Branch, array, 0))
def score_leaf(leaf: Leaf) -> U32:
  match leaf:
    case Number{value}: value
    case Caption{text, amount}: U32.add(amount, U32.from_nat(String.length(text)))
def score(branch: Branch) -> U32:
  match branch:
    case Branch{leaf, count}: U32.add(score_leaf(leaf), count)
def sample_result() -> U32: score!(roundtrip!())

def main() -> IO(Unit): IO.print(U32.show(sample_result()))
