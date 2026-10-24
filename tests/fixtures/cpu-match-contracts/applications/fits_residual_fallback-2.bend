# SPDX-License-Identifier: MPL-2.0
import Base
import ../fits_residual_fallback.bend as H
law main: H.Unit
def main(): H.matcher(H.A{})
