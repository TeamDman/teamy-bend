# SPDX-License-Identifier: MPL-2.0
type Flag is Data: On{} Off{}
def choose(~key: Flag, x: Flag) -> Flag:
  {\{key: Off{}; (rest => rest)} : Flag -> Flag}(x)
def main() -> Flag:
  choose(~On, On{})
