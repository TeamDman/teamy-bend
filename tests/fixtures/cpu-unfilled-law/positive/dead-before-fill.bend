# SPDX-License-Identifier: MPL-2.0
import Base
law token: U32
def ignore(-x: U32) -> U32: 7
def early() -> U32: ignore(token)
def token(): 3
def main() -> U32: early()
