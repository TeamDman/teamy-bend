import Base
type Custom is Data:
  Con{head: U32, tail: Custom}
  Nil{}
type Product is Data:
  Tuple{first: Custom, second: U32}
def make() -> Product: Tuple{Con{5, Nil{}}, 6}
