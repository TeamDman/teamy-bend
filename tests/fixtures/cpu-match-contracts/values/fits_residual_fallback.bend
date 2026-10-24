# SPDX-License-Identifier: MPL-2.0
import Base
import ../fits_residual_fallback.bend as H
def main() -> List<H.Unit>: [H.fallback(H.A{}), H.fallback(H.B{}), H.matcher(H.A{}), H.matcher(H.B{})]
