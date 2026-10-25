<!-- SPDX-License-Identifier: MPL-2.0 -->
# Unresolved reusable Nat-pattern probe

These project-authored pure programs isolate the next semantic candidate from IO.
At the recorded source, pinned upstream accepts tight `1n++k` and prints `4n`;
the port rejects it during loading as `String.append(1n, k)`. The ordinary `1n+k`
control rejects duplicated affine use during checking in both implementations.
The spaced `1n + +k` control load-rejects in both, with differing `.add`/`Nat.add`
representation. All repeated observations are deterministic. No correction is
implemented here, and these files are not added to the passing negative manifest.
