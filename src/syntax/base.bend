# SPDX-License-Identifier: Apache-2.0
# Derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
# Selected pure definitions from upstream base.bend; Rust implementation changes: TeamDman.
# See NOTICE and licenses/Apache-2.0.txt. Foreign effects and templates remain unsupported.

type Empty is Data:

type Unit is Data:
  Unit{}

type Bool is Data:
  False{}
  True{}

type Cmp is Data:
  LT{}
  EQ{}
  GT{}

type Either<a, b, -A: Kind(a), -B: Kind(b)> is Kind(a <&> b):
  Inl{value: A}
  Inr{value: B}

type Sigma<a, b, -A: Kind(a), -B: @-x: A -> Kind(b)> is Kind(a <&> b):
  Tuple{fst: A, snd: B(fst)}

type Nat is Data:
  Zero{}
  Succ{pred: Nat}

type Maybe<a, -A: Kind(a)> is Kind(a):
  None{}
  Some{value: A}

type Result<a, b, -E: Kind(a), -A: Kind(b)> is Kind(a <&> b):
  Fail{error: E}
  Done{value: A}

type List<a, -A: Kind(a)> is Kind(a):
  Nil{}
  Con{head: A, tail: List<a, A>}

law Word:
  for n: Nat
  Data

type Word.Nil is Data:
  WNil{}

type Word.Con<-p: Nat> is Data:
  WCon{head: Bool, tail: Word(p)}

type U32 is Data:
  U32{data: Word(32n)}

type F32 is Data:
  F32{data: Word(32n)}

type Char is Data:
  Chr{code: U32}

type String is Data:
  SNil{}
  SCon{head: Char, tail: String}

type Array<-T: Type> is Type:
  ALeaf{value: T}
  ANode{xs: Array<T>, ys: Array<T>}

law Pair:
  for -A: Type
  for -B: Type
  Type

def Word(n):
  match n:
    case 0n:
      Word.Nil
    case 1n+p:
      Word.Con<p>

# Pair
# ----

def Pair(A, B):
  Sigma<&1, &1, A, _ => B>

def Exists(-A: Type, -B: @-x: A -> Type) -> Type:
  Sigma<&1, &1, A, B>

def Or(-A: Type, -B: Type) -> Type:
  Either<&1, &1, A, B>

def Pair.fst(-A: Type, -B: Type, p: A & B) -> A:
  (a, b) = p
  a

def Pair.snd(-A: Type, -B: Type, p: A & B) -> B:
  (a, b) = p
  b

# Effects
# -------

law Equal.cong:
  for -A: Type
  for -B: Type
  for -f: A -> B
  for -a: A
  for -b: A
  for  e: {a == b : A}
  {f(a) == f(b) : B}

def Equal.cong(A, B, f, a, b, e):
  %e : {f(a) == f(_) : B}
  {==}

law Equal.sym:
  for -A: Type
  for -a: A
  for -b: A
  for  e: {a == b : A}
  {b == a : A}

def Equal.sym(A, a, b, e):
  %e : {_ == a : A}
  {==}

law Equal.trans:
  for -A : Type
  for -a : A
  for -b : A
  for -c : A
  for ab : {a == b : A}
  for bc : {b == c : A}
  {a == c : A}

def Equal.trans(A, a, b, c, ab, bc):
  %bc : {a == _ : A}
  ab

# Empty
# -----

def Empty.absurd(-A: Type, e: Empty) -> A:
  match e:

# Bool
# ----

def Bool.not(b: Bool) -> Bool:
  match b:
    case False{}:
      True{}
    case True{}:
      False{}

def Bool.and(a: Bool, b: Bool) -> Bool:
  match a:
    case False{}:
      False{}
    case True{}:
      b

def Bool.or(a: Bool, b: Bool) -> Bool:
  match a:
    case False{}:
      b
    case True{}:
      True{}

def Bool.xor(a: Bool, b: Bool) -> Bool:
  match a:
    case False{}:
      b
    case True{}:
      Bool.not(b)

def Bool.cmp(a: Bool, b: Bool) -> Cmp:
  match a b:
    case False{} False{}:
      EQ{}
    case False{} True{}:
      LT{}
    case True{} False{}:
      GT{}
    case True{} True{}:
      EQ{}

def Bool.full_add(a: Bool, b: Bool, c: Bool) -> Bool & Bool:
  match a b c:
    case False{} False{} False{}:
      (False{}, False{})
    case False{} False{} True{}:
      (True{}, False{})
    case False{} True{} False{}:
      (True{}, False{})
    case False{} True{} True{}:
      (False{}, True{})
    case True{} False{} False{}:
      (True{}, False{})
    case True{} False{} True{}:
      (False{}, True{})
    case True{} True{} False{}:
      (False{}, True{})
    case True{} True{} True{}:
      (True{}, True{})

def Bool.pick(-A: Type, c: Bool, a: A, b: A) -> A:
  match c:
    case False{}:
      b
    case True{}:
      a

def Bool.to_u32(b: Bool) -> U32:
  match b:
    case False{}:
      0
    case True{}:
      1

# Cmp
# ---

def Cmp.is_lt(c: Cmp) -> Bool:
  match c:
    case LT{}:
      True{}
    case EQ{}:
      False{}
    case GT{}:
      False{}

def Cmp.is_eq(c: Cmp) -> Bool:
  match c:
    case LT{}:
      False{}
    case EQ{}:
      True{}
    case GT{}:
      False{}

def Cmp.is_gt(c: Cmp) -> Bool:
  match c:
    case LT{}:
      False{}
    case EQ{}:
      False{}
    case GT{}:
      True{}

def Cmp.is_le(c: Cmp) -> Bool:
  match c:
    case LT{}:
      True{}
    case EQ{}:
      True{}
    case GT{}:
      False{}

def Cmp.is_ge(c: Cmp) -> Bool:
  match c:
    case LT{}:
      False{}
    case EQ{}:
      True{}
    case GT{}:
      True{}

# Nat
# ---

def Nat.double(n: Nat) -> Nat:
  match n:
    case 0n:
      0n
    case 1n+p:
      2n+Nat.double(p)

def Nat.add(a: Nat, b: Nat) -> Nat:
  match a:
    case 0n:
      b
    case 1n+p:
      1n+Nat.add(p, b)

def Nat.sub(a: Nat, b: Nat) -> Nat:
  match a b:
    case 0n 0n:
      0n
    case 0n 1n+bp:
      0n
    case 1n+ap 0n:
      1n+ap
    case 1n+ap 1n+bp:
      Nat.sub(ap, bp)

def Nat.mul(a: Nat, +b: Nat) -> Nat:
  match a:
    case 0n:
      0n
    case 1n+p:
      Nat.add(b, Nat.mul(p, b))

def Nat.divmod.go(n: Nat, m: Nat, d: Nat, r: Nat) -> Nat & Nat:
  match n:
    case 0n:
      (d, r)
    case 1n+np:
      match m:
        case 0n:
          Nat.divmod.go(np, r, 1n+d, 0n)
        case 1n+mp:
          Nat.divmod.go(np, mp, d, 1n+r)

def Nat.divmod(a: Nat, b: Nat) -> Nat & Nat:
  match b:
    case 0n:
      (0n, a)
    case 1n+bp:
      Nat.divmod.go(a, bp, 0n, 0n)

def Nat.cmp(a: Nat, b: Nat) -> Cmp:
  match a b:
    case 0n 0n:
      EQ{}
    case 0n 1n+bp:
      LT{}
    case 1n+ap 0n:
      GT{}
    case 1n+ap 1n+bp:
      Nat.cmp(ap, bp)

def Nat.is_lt(a: Nat, b: Nat) -> Bool:
  Cmp.is_lt(Nat.cmp(a, b))

def Nat.is_eq(a: Nat, b: Nat) -> Bool:
  Cmp.is_eq(Nat.cmp(a, b))

def Nat.is_ne(a: Nat, b: Nat) -> Bool:
  Bool.not(Cmp.is_eq(Nat.cmp(a, b)))

def Nat.is_le(a: Nat, b: Nat) -> Bool:
  Cmp.is_le(Nat.cmp(a, b))

def Nat.is_gt(a: Nat, b: Nat) -> Bool:
  Cmp.is_gt(Nat.cmp(a, b))

def Nat.is_ge(a: Nat, b: Nat) -> Bool:
  Cmp.is_ge(Nat.cmp(a, b))

def Nat.min(+a: Nat, +b: Nat) -> Nat:
  Bool.pick(Nat, Nat.is_lt(a, b), a, b)

def Nat.max(+a: Nat, +b: Nat) -> Nat:
  Bool.pick(Nat, Nat.is_lt(a, b), b, a)

def Nat.div.fin(qr: Nat & Nat) -> Nat:
  (q, r) = qr
  q

def Nat.div(a: Nat, b: Nat) -> Nat:
  Nat.div.fin(Nat.divmod(a, b))

def Nat.mod.fin(qr: Nat & Nat) -> Nat:
  (q, r) = qr
  r

def Nat.mod(a: Nat, b: Nat) -> Nat:
  Nat.mod.fin(Nat.divmod(a, b))

def Nat.pow(+a: Nat, b: Nat) -> Nat:
  match b:
    case 0n:
      1n
    case 1n+p:
      Nat.mul(a, Nat.pow(a, p))

# Maybe
# -----

def Maybe.pure(a, -A: Kind(a), x: A) -> Maybe<a, A>:
  Some{x}

def Maybe.bind(
  a, -A: Kind(a), -B: Kind(a), m: Maybe<a, A>, f: A -> Maybe<a, B>
) -> Maybe<a, B>:
  match m:
    case None{}:
      None{}
    case Some{x}:
      f(x)

def Maybe.default(a, -A: Kind(a), m: Maybe<a, A>, d: A) -> A:
  match m:
    case None{}:
      d
    case Some{x}:
      x

def Maybe.is_some(a, -A: Kind(a), m: Maybe<a, A>) -> Bool:
  match m:
    case None{}:
      False{}
    case Some{x}:
      True{}

def Maybe.is_none(a, -A: Kind(a), m: Maybe<a, A>) -> Bool:
  Bool.not(Maybe.is_some(a, A, m))

def Maybe.map(
  a, -A: Kind(a), -B: Kind(a), f: A -> B, m: Maybe<a, A>
) -> Maybe<a, B>:
  match m:
    case None{}:
      None{}
    case Some{x}:
      Some{f(x)}

def Maybe.or(a, -A: Kind(a), m: Maybe<a, A>, n: Maybe<a, A>) -> Maybe<a, A>:
  match m:
    case None{}:
      n
    case Some{x}:
      Some{x}

# Result
# ------

def Result.pure(a, b, -E: Kind(a), -A: Kind(b), x: A) -> Result<a, b, E, A>:
  Done{x}

def Result.bind(
  a, b, -E: Kind(a), -A: Kind(b), -B: Kind(b), r: Result<a, b, E, A>,
  f: A -> Result<a, b, E, B>
) -> Result<a, b, E, B>:
  match r:
    case Fail{e}:
      Fail{e}
    case Done{x}:
      f(x)

def Result.default(
  a, b, -E: Kind(a), -A: Kind(b), r: Result<a, b, E, A>, d: A
) -> A:
  match r:
    case Fail{e}:
      d
    case Done{x}:
      x

def Result.is_done(
  a, b, -E: Kind(a), -A: Kind(b), r: Result<a, b, E, A>
) -> Bool:
  match r:
    case Fail{e}:
      False{}
    case Done{x}:
      True{}

def Result.is_fail(
  a, b, -E: Kind(a), -A: Kind(b), r: Result<a, b, E, A>
) -> Bool:
  Bool.not(Result.is_done(a, b, E, A, r))

def Result.map(
  a, b, -E: Kind(a), -A: Kind(b), -B: Kind(b), f: A -> B,
  r: Result<a, b, E, A>
) -> Result<a, b, E, B>:
  match r:
    case Fail{e}:
      Fail{e}
    case Done{x}:
      Done{f(x)}

# List
# ----

def List.length(a, -A: Kind(a), xs: List<a, A>) -> Nat:
  match xs:
    case Nil{}:
      0n
    case h <> t:
      1n+List.length(a, A, t)

def List.append(a, -A: Kind(a), xs: List<a, A>, ys: List<a, A>) -> List<a, A>:
  match xs:
    case Nil{}:
      ys
    case h <> t:
      h <> List.append(a, A, t, ys)

def List.concat(a, -A: Kind(a), xss: List<a, List<a, A>>) -> List<a, A>:
  match xss:
    case Nil{}:
      Nil{}
    case h <> t:
      List.append(a, A, h, List.concat(a, A, t))

def List.reverse.go(a, -A: Kind(a), xs: List<a, A>, acc: List<a, A>) ->
  List<a, A>:
  match xs:
    case Nil{}:
      acc
    case h <> t:
      List.reverse.go(a, A, t, h <> acc)

def List.reverse(a, -A: Kind(a), xs: List<a, A>) -> List<a, A>:
  List.reverse.go(a, A, xs, Nil{})

def List.is_empty(a, -A: Kind(a), xs: List<a, A>) -> Bool:
  match xs:
    case Nil{}:
      True{}
    case h <> t:
      False{}

def List.head(a, -A: Kind(a), xs: List<a, A>) -> Maybe<a, A>:
  match xs:
    case Nil{}:
      None{}
    case h <> t:
      Some{h}

def List.tail(a, -A: Kind(a), xs: List<a, A>) -> List<a, A>:
  match xs:
    case Nil{}:
      Nil{}
    case h <> t:
      t

def List.last.go(a, -A: Kind(a), xs: List<a, A>, last: A) -> A:
  match xs:
    case Nil{}:
      last
    case h <> t:
      List.last.go(a, A, t, h)

def List.last(a, -A: Kind(a), xs: List<a, A>) -> Maybe<a, A>:
  match xs:
    case Nil{}:
      None{}
    case h <> t:
      Some{List.last.go(a, A, t, h)}

def List.get(a, -A: Kind(a), xs: List<a, A>, n: Nat) -> Maybe<a, A>:
  match xs n:
    case Nil{} _:
      None{}
    case h <> t 0n:
      Some{h}
    case h <> t 1n+p:
      List.get(a, A, t, p)

def List.set(a, -A: Kind(a), xs: List<a, A>, n: Nat, x: A) -> List<a, A>:
  match xs n:
    case Nil{} _:
      Nil{}
    case h <> t 0n:
      x <> t
    case h <> t 1n+p:
      h <> List.set(a, A, t, p, x)

def List.take(a, -A: Kind(a), xs: List<a, A>, n: Nat) -> List<a, A>:
  match xs n:
    case Nil{} _:
      Nil{}
    case h <> t 0n:
      Nil{}
    case h <> t 1n+p:
      h <> List.take(a, A, t, p)

def List.drop(a, -A: Kind(a), xs: List<a, A>, n: Nat) -> List<a, A>:
  match xs n:
    case Nil{} _:
      Nil{}
    case h <> t 0n:
      h <> t
    case h <> t 1n+p:
      List.drop(a, A, t, p)

def List.zip(
  a, -A: Kind(a), b, -B: Kind(b), xs: List<a, A>, ys: List<b, B>
) -> List<&1, A & B>:
  match xs ys:
    case Nil{} _:
      Nil{}
    case h <> t Nil{}:
      Nil{}
    case x <> xt y <> yt:
      (x, y) <> List.zip(a, A, b, B, xt, yt)

def List.range.go(+n: Nat, acc: List<&2, Nat>) -> List<&2, Nat>:
  match n:
    case 0n:
      acc
    case 1n+p:
      List.range.go(p, p <> acc)

def List.range(n: Nat) -> List<&2, Nat>:
  List.range.go(n, Nil{})

def List.replicate(-A: Data, n: Nat, +x: A) -> List<&2, A>:
  match n:
    case 0n:
      Nil{}
    case 1n+p:
      x <> List.replicate(A, p, x)

def List.filter.put(-A: Data, h: A, r: List<&2, A>, keep: Bool) -> List<&2, A>:
  match keep:
    case False{}:
      r
    case True{}:
      h <> r

def Char.to_u32(c: Char) -> U32:
  match c:
    case Chr{x}:
      x

def Char.from_u32(x: U32) -> Char:
  Chr{x}

def String.append(a: String, b: String) -> String:
  match a:
    case SNil{}:
      b
    case SCon{h, t}:
      SCon{h, String.append(t, b)}

def String.length(s: String) -> Nat:
  match s:
    case SNil{}:
      0n
    case SCon{h, t}:
      1n+String.length(t)

def String.is_empty(s: String) -> Bool:
  match s:
    case SNil{}:
      True{}
    case SCon{h, t}:
      False{}

def String.reverse.go(s: String, acc: String) -> String:
  match s:
    case SNil{}:
      acc
    case SCon{h, t}:
      String.reverse.go(t, SCon{h, acc})

def String.reverse(s: String) -> String:
  String.reverse.go(s, SNil{})

def String.take(s: String, n: Nat) -> String:
  match s n:
    case SNil{} _:
      SNil{}
    case SCon{h, t} 0n:
      SNil{}
    case SCon{h, t} 1n+p:
      SCon{h, String.take(t, p)}

def String.drop(s: String, n: Nat) -> String:
  match s n:
    case SNil{} _:
      SNil{}
    case SCon{h, t} 0n:
      SCon{h, t}
    case SCon{h, t} 1n+p:
      String.drop(t, p)

def String.get(s: String, n: Nat) -> Maybe<&2, Char>:
  match s n:
    case SNil{} _:
      None{}
    case SCon{h, t} 0n:
      Some{h}
    case SCon{h, t} 1n+p:
      String.get(t, p)

def String.to_list(s: String) -> List<&2, Char>:
  match s:
    case SNil{}:
      Nil{}
    case SCon{h, t}:
      h <> String.to_list(t)

def String.from_list(cs: List<&2, Char>) -> String:
  match cs:
    case Nil{}:
      SNil{}
    case h <> t:
      SCon{h, String.from_list(t)}

def String.concat(xs: List<&2, String>) -> String:
  match xs:
    case Nil{}:
      SNil{}
    case h <> t:
      String.append(h, String.concat(t))

def String.join.go(xs: List<&2, String>, h: String, +sep: String) -> String:
  match xs:
    case Nil{}:
      h
    case h2 <> t:
      String.append(h, String.append(sep, String.join.go(t, h2, sep)))

def String.join(xs: List<&2, String>, +sep: String) -> String:
  match xs:
    case Nil{}:
      SNil{}
    case h <> t:
      String.join.go(t, h, sep)

def String.repeat(+s: String, n: Nat) -> String:
  match n:
    case 0n:
      SNil{}
    case 1n+p:
      String.append(s, String.repeat(s, p))

def Char.show(c: Char) -> String:
  SCon{c, SNil{}}
