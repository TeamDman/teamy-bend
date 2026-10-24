type Nat is Data:
  Zero{}
  Succ{pred: Nat}
def bad(n: Nat) -> Nat: Wrong{n}
def main() -> Nat: bad(0n)
