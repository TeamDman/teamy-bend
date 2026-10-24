# λ source
type Nat is Data:
  Zero{}
  Succ{pred: Nat}
def bad() -> {0n == 1n : Nat}: {==}
