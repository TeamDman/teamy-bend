import Base
type Item is Data:
  Box{value: U32}
def make() -> Item: Box{7}
def main() -> Item: make()
