# SPDX-License-Identifier: MPL-2.0
type N is Data: Z{}
law bad: N -> N
def bad(x): missing(x)
