# SPDX-License-Identifier: MPL-2.0
type Nothing is Data:
type Flag is Data: On{}
@unsafe def keep(x: Flag) -> Flag: x
def main() -> Flag: On{}
