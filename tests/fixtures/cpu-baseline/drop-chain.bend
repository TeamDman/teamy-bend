import Base
type Chain is Data:
  End{}
  Link{tail: Chain, label: String}
def build(n: Nat, chain: Chain) -> Chain:
  match n:
    case 0n: chain
    case 1n+p: build(p, Link{chain, "x"})
def discard(chain: Chain) -> U32: 16
def sample_result() -> U32: discard!(build!(16n, End{}))

def main() -> IO(Unit): IO.print(U32.show(sample_result()))
