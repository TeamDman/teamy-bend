# SPDX-License-Identifier: MPL-2.0
type Other is Data: Other{}
law bad: Nx
def bad(): depth(Other{})
