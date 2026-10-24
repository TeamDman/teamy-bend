# SPDX-License-Identifier: MPL-2.0
import Base
import ../fits_residual_fallback.bend as H
def count(n: H.Unit) -> U32:
  match n:
    case H.Unit{}: 0
def main() -> List<U32>: [count(H.fallback(H.A{})), count(H.fallback(H.B{})), count(H.matcher(H.A{})), count(H.matcher(H.B{}))]
