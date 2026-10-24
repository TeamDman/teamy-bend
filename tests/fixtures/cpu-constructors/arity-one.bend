type Nat is Data:
  Zero{}
  Succ{pred: Nat}
def main() -> Nat: Succ{}
