# SPDX-License-Identifier: MPL-2.0
import Base
import ../hoas_binder_refresh.bend as H
def main() -> List<U32>:
  [H.main()(0)(0), H.main()(0)(7), H.main()(0)(4294967295), H.main()(7)(0), H.main()(7)(7), H.main()(7)(4294967295), H.main()(4294967295)(0), H.main()(4294967295)(7), H.main()(4294967295)(4294967295)]
