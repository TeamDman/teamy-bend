# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
def app(~n: N) -> N -> N: n => n
def main() -> N -> N: app(~later)
def later() -> N: Z{}
