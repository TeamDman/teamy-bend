type Nat is Data:
  Zero{}
  Succ{pred: Nat}
def bad(n: Nat) -> Nat:
  match n:
    case Succ{
      Missing{}
    }: Zero{}
    case Zero{}: Zero{}
def main() -> Nat: bad(0n)
