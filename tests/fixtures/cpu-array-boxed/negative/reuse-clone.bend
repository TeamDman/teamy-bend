# SPDX-License-Identifier: MPL-2.0
import Base

def bad(r: Array<String> & Array<String>) -> Array<String> & Array<String>:
  (left, right) = r
  (left, left)

def main() -> Array<String> & Array<String>:
  bad(Array.clone(String, ALeaf{"x"}))
