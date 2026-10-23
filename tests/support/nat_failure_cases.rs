// SPDX-License-Identifier: MPL-2.0
// Each expression crosses the same 48-bit language boundary. The boolean
// explicitly records a native checked-body expansion that still exhausts work;
// a passing safety regression does not qualify that lane as upstream parity.
pub fn cases() -> [(&'static str, bool); 6] {
    [
        ("Nat.mul(U32.to_nat(16777216), U32.to_nat(16777216))", false),
        ("Nat.mul(limit(), limit())", false),
        ("Succ{limit()}", false),
        ("Nat.add(1n, limit())", false),
        ("Nat.add(limit(), 1n)", true),
        (
            "Nat.double(Nat.mul(U32.to_nat(8388608), U32.to_nat(16777216)))",
            true,
        ),
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
