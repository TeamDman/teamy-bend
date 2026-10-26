# SPDX-License-Identifier: MPL-2.0
import Base

def main() -> Array<Array<String>> & Array<Array<String>>:
  Array.clone(Array<String>, ALeaf{ALeaf{"nested"}})
