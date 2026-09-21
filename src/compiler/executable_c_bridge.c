/* SPDX-License-Identifier: Apache-2.0
 * Native ABI derived from Bend 2.0.5 comp.ts, Copyright 2026 HigherOrderCO.
 * Pointer-Env bridges: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
 * Keep generated bodies on pointer arguments. In unoptimized MSVC, passing
 * the 16-byte Env by value creates a distinct stack temporary at every call
 * site. These fixed-size non-inlined bridges preserve the foreign ABI without
 * allowing a generated body's native frame to grow with its source length. */
OUTLINE TB_NOINLINE Term tb_c_construct(const Env *e, u32 cid, u32 count, const Term *fields, bool packed, const Term *box_mask) {
  Term value = tb_construct(*e, cid, count, fields, packed);
  if (!packed && box_mask != NULL) {
    Loc at = term_peek(*e, value);
    for (u32 index = 0; index < count; ++index)
      if (box_mask[index] == 0) tb_mark_raw(*e, at + index, 1);
  }
  return value;
}
OUTLINE TB_NOINLINE void tb_c_fields(const Env *e, Term value, u32 cid, u32 count, bool packed, Term *out) {
  tb_fields(*e, value, cid, count, packed, out);
}
OUTLINE TB_NOINLINE void tb_c_borrow_fields(const Env *e, Term value, u32 cid, u32 count, bool packed, Term *out) {
  tb_borrow_fields(*e, value, cid, count, packed, out);
}
OUTLINE TB_NOINLINE Term tb_c_duplicate(const Env *e, Term *value) { return tb_duplicate(*e, value); }
OUTLINE TB_NOINLINE void tb_c_drop(const Env *e, Term value) { term_drop(*e, value); }
OUTLINE TB_NOINLINE Loc tb_c_peek(const Env *e, Term value) { return term_peek(*e, value); }
OUTLINE TB_NOINLINE Term tb_c_apply(const Env *e, Term closure, Term argument) {
  return tb_apply(*e, closure, argument);
}
OUTLINE TB_NOINLINE Term tb_c_tail_apply(const Env *e, Term closure, Term argument) {
  return tb_tail_apply(*e, closure, argument);
}
OUTLINE TB_NOINLINE Term tb_c_join(const Env *e, u32 fid, u32 held_count, const Term *held,
    u32 children, const Term *applications) {
  Loc at;
  Term join;
  if (children < 2 || held_count > 254 || children > 254 - held_count
      || fid_arity(fid) != held_count + children + 1)
    err_fail("invalid generated fork layout");
  at = task_node(*e, fid, TERM_HOLE, 0, children);
  join = term_tsk(fid, at);
  if (held_count != 0) memcpy(e->mem + at, held, held_count * sizeof(Term));
  e->mem[at + held_count + children] = 0;
  for (u32 index = 0; index < children; ++index) {
    u32 destination = held_count + index;
    Loc child = task_node(*e, FID_CLO_APPLY, join, destination, 0);
    e->mem[child] = applications[2 * index];
    e->mem[child + 1] = applications[2 * index + 1];
    e->mem[at + destination] = term_tsk(FID_CLO_APPLY, child);
  }
  return join;
}
OUTLINE TB_NOINLINE Term tb_c_closure(const Env *e, u32 fid, u32 count, const Term *captures) {
  return tb_closure(*e, fid, count, captures);
}
OUTLINE TB_NOINLINE Term tb_c_word(const Env *e, u32 value) { return tb_word(*e, value); }
OUTLINE TB_NOINLINE Term tb_c_term_word(const Env *e, Term value) { return term_word(*e, value); }
OUTLINE TB_NOINLINE Term tb_c_nat_chk(const Env *e, Nat value) { return nat_chk(*e, value); }
OUTLINE TB_NOINLINE Term tb_c_nat_mul(const Env *e, Nat left, Nat right) { return nat_mul(*e, left, right); }
OUTLINE TB_NOINLINE Term tb_c_blk_node(const Env *e, Term left, Term right) { return blk_node(*e, left, right); }
OUTLINE TB_NOINLINE Term tb_c_blk_new(const Env *e, bool array, Nat depth, u32 lgs, u32 count, Term *values, const Term *box_mask) {
  return tb_blk_new_masked(*e, array, depth, lgs, count, values, box_mask);
}
OUTLINE TB_NOINLINE Term tb_c_blk_half(const Env *e, Term value, u32 high) { return blk_half(*e, value, high); }
OUTLINE TB_NOINLINE Term tb_c_blk_copy(const Env *e, Term value) { return blk_copy(*e, value); }
OUTLINE TB_NOINLINE Term tb_c_blk_unique(const Env *e, Term value) { return blk_unique(*e, value); }
OUTLINE TB_NOINLINE Term tb_c_blk_keep(const Env *e, Loc at) { return blk_keep(*e, at); }
OUTLINE TB_NOINLINE void tb_c_blk_free(const Env *e, Term value) { blk_free(*e, value); }
OUTLINE TB_NOINLINE void tb_c_blk_write(const Env *e, bool array, Loc at, u32 index, Term value, bool owned, bool discard) {
  if (array && discard && tb_cell_owned(*e, at + index)) term_drop(*e, e->mem[at + index]);
  blk_write(e->mem, array, at, index, value);
  if (array) {
    if (owned) tb_mark_owned(*e, at + index, 1);
    else tb_mark_raw(*e, at + index, 1);
  }
}
OUTLINE TB_NOINLINE Term tb_c_f32_show(const Env *e, Term value) { return tb_f32_show(*e, value); }
OUTLINE TB_NOINLINE Term tb_c_f32_read(const Env *e, Term value) { return tb_f32_read(*e, value); }
OUTLINE TB_NOINLINE void tb_c_show_native(const Env *e, Term value, unsigned kind) { tb_show_native(*e, value, kind); }
