// SPDX-License-Identifier: MPL-2.0
// Each expression crosses the same 48-bit language boundary.
pub fn cases() -> [&'static str; 6] {
    [
        "Nat.mul(U32.to_nat(16777216), U32.to_nat(16777216))",
        "Nat.mul(limit(), limit())",
        "Succ{limit()}",
        "Nat.add(1n, limit())",
        "Nat.add(limit(), 1n)",
        "Nat.double(Nat.mul(U32.to_nat(8388608), U32.to_nat(16777216)))",
    ]
}

pub fn source(expression: &str) -> String {
    format!(
        r#"import Base
def limit() -> Nat: Nat.mul(U32.to_nat(16777215), U32.to_nat(16777217))
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.write("before")
    Unit <- IO.print(Nat.show({expression}))
    IO.print("after")
"#
    )
}
