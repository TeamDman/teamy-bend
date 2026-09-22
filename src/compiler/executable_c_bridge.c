/* SPDX-License-Identifier: Apache-2.0
 * Native ABI derived from Bend 2.0.5 comp.ts, Copyright 2026 HigherOrderCO.
 * Pointer-Env bridges: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
 * Keep generated bodies on pointer arguments. In unoptimized MSVC, passing
 * the 16-byte Env by value creates a distinct stack temporary at every call
 * site. These fixed-size non-inlined bridges preserve the foreign ABI without
 * allowing a generated body's native frame to grow with its source length. */
/* TB_SHARED_BRIDGES */
OUTLINE TB_NOINLINE Term tb_c_apply(const Env *e, Term closure, Term argument) {
  return tb_apply(*e, closure, argument);
}
OUTLINE TB_NOINLINE Term tb_c_f32_show(const Env *e, Term value) { return tb_f32_show(*e, value); }
OUTLINE TB_NOINLINE Term tb_c_f32_read(const Env *e, Term value) { return tb_f32_read(*e, value); }
OUTLINE TB_NOINLINE void tb_c_show_native(const Env *e, Term value, unsigned kind) { tb_show_native(*e, value, kind); }
