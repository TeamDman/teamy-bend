import Base
import left.bend as L
import right.bend as R
type Combined is Data:
  Both{left: L.Item, right: R.Item}
def main() -> Combined: Both{L.make(), R.make()}
