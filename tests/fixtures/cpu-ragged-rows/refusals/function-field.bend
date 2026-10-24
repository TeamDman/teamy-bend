import Base
type Box is Type:
  Box{f: U32 -> U32}
def main() -> Box: Box{x => x}
