/* SPDX-License-Identifier: Apache-2.0
 * Native values derived from Bend 2.0.5 comp.ts, Copyright 2026 HigherOrderCO.
 * Shared CPU/device implementation: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
 * Backend supplies allocation state, synchronization and bounded scratch storage. */
INLINE Term term_blk(bool array, Cls cls, Loc loc) { return term_make(array ? TAG_ARR : TAG_BUF, cls, loc); }
INLINE Cls blk_cls(Term block) { return (Cls)term_aux(block); }
INLINE Cls buf_wcls(Cls cls) { return cls == 0 ? 0 : cls - 1; }
INLINE Cls blk_span(Term block) { return term_tag(block) == TAG_ARR ? blk_cls(block) : buf_wcls(blk_cls(block)); }
INLINE void blk_free(Env e, Term block) {
  if (term_rfc(block) || (term_tag(block) != TAG_ARR && term_tag(block) != TAG_BUF) || blk_cls(block) > 17)
    err_fail("unique array storage expected");
  heap_free(e, blk_span(block), term_loc(block));
}
INLINE Term blk_read(Corpus memory, bool array, Loc at, u32 index) {
  Env e = {memory, NULL};
  tb_span(e, at, (array ? (u64)index : (u64)index / 2) + 1);
  return array ? memory[at + index] : (u32)(memory[at + index / 2] >> ((index & 1) * 32));
}
INLINE void blk_write(Corpus memory, bool array, Loc at, u32 index, Term value) {
  Env e = {memory, NULL}; u32 shift = (index & 1) * 32;
  tb_span(e, at, (array ? (u64)index : (u64)index / 2) + 1);
  if (tb_cell_sealed(e, at)) err_fail("mutation of sealed native payload");
  if (array) memory[at + index] = value;
  else memory[at + index / 2] = (memory[at + index / 2] & ~(UINT64_C(0xffffffff) << shift)) | ((u64)(u32)value << shift);
}
INLINE u32 blk_at(Term block, U32 index, u32 lgs) {
  Cls cls = blk_cls(block);
  if (cls < lgs || cls > 17) err_fail("invalid array layout");
  return ((u32)index & ((1u << (cls - lgs)) - 1)) << lgs;
}
INLINE Term blk_keep(Env e, Loc at) {
  return tb_cell_owned(e, at) ? tb_duplicate(e, e.mem + at) : e.mem[at];
}
INLINE Term tb_blk_new_masked(Env e, bool array, Nat depth, u32 lgs, u32 count, Term *values, const Term *box_mask) {
  Cls cls; Loc at; u32 stride;
  if (depth > 17 || lgs > 17 || depth + lgs > 17 || count > (1u << lgs)) err_fail("array budget exhausted");
  cls = (Cls)depth + lgs; stride = 1u << lgs;
  at = heap_alloc(e, array ? cls : buf_wcls(cls));
  for (u32 index = 0; index < (1u << cls); ++index) {
    u32 column = index % stride;
    bool owned = array && column < count && (box_mask == NULL || box_mask[column] != 0);
    Term value = 0;
    if (column < count) {
      value = owned && (index >> lgs) + 1 < (1u << (u32)depth)
          ? tb_duplicate(e, values + column) : values[column];
    }
    blk_write(e.mem, array, at, index, value);
    if (array && !owned) tb_mark_raw(e, at + index, 1);
  }
  return term_blk(array, cls, at);
}
INLINE Term blk_new(Env e, bool array, Nat depth, u32 lgs, u32 count, Term *values) {
  return tb_blk_new_masked(e, array, depth, lgs, count, values, NULL);
}
INLINE Term blk_copy(Env e, Term block) {
  Cls cls = blk_span(block); Loc at; Loc from = term_peek(e, block);
  bool array = term_tag(block) == TAG_ARR;
  if ((!array && term_tag(block) != TAG_BUF) || blk_cls(block) > 17) err_fail("array budget exhausted");
  tb_allocation(e, from, cls);
  at = heap_alloc(e, cls);
  for (u64 index = 0; index < (UINT64_C(1) << cls); ++index) {
    bool owned = array && tb_cell_owned(e, from + index);
    e.mem[at + index] = owned ? blk_keep(e, from + index) : e.mem[from + index];
    if (!owned) tb_mark_raw(e, at + index, 1);
  }
  return term_blk(term_tag(block) == TAG_ARR, blk_cls(block), at);
}
INLINE Term blk_unique(Env e, Term block) {
  Loc at = term_peek(e, block);
  if ((term_tag(block) != TAG_ARR && term_tag(block) != TAG_BUF) || blk_cls(block) > 17)
    err_fail("array expected");
  tb_allocation(e, at, blk_span(block));
  if (term_rfc(block)) {
    Loc cell = term_loc(block);
    if (rfc_claim_unique(e, cell)) {
      tb_set_sealed(e, at, false);
      return (block & ~(RFC_BIT | LOC_MASK)) | at;
    }
    Term copy = blk_copy(e, block); term_drop(e, block); return copy;
  }
  return block;
}
INLINE Term blk_half(Env e, Term block, u32 high) {
  bool array = term_tag(block) == TAG_ARR; Cls cls = blk_cls(block); Loc at; Loc from = term_peek(e, block);
  if (term_rfc(block) || (!array && term_tag(block) != TAG_BUF) || cls == 0 || cls > 17 || high > 1)
    err_fail("invalid array half");
  tb_allocation(e, from, blk_span(block));
  --cls; at = heap_alloc(e, array ? cls : buf_wcls(cls));
  for (u32 index = 0; index < (1u << cls); ++index) {
    u32 source = (high << cls) + index;
    blk_write(e.mem, array, at, index, blk_read(e.mem, array, from, source));
    if (array && !tb_cell_owned(e, from + source)) tb_mark_raw(e, at + index, 1);
  }
  if (high != 0) blk_free(e, block);
  return term_blk(array, cls, at);
}
INLINE Term blk_node(Env e, Term left, Term right) {
  bool array = term_tag(left) == TAG_ARR; Cls cls = blk_cls(left); Loc at, l, r;
  if (term_rfc(left) || term_rfc(right) || (!array && term_tag(left) != TAG_BUF)
      || cls != blk_cls(right) || term_tag(left) != term_tag(right) || cls >= 17) err_fail("invalid array concatenation");
  l = term_peek(e, left); r = term_peek(e, right);
  tb_allocation(e, l, blk_span(left)); tb_allocation(e, r, blk_span(right));
  if (l == r) err_fail("array concatenation aliases unique storage");
  at = heap_alloc(e, array ? cls + 1 : buf_wcls(cls + 1));
  for (u32 index = 0; index < (1u << cls); ++index) {
    blk_write(e.mem, array, at, index, blk_read(e.mem, array, l, index));
    blk_write(e.mem, array, at, (1u << cls) + index, blk_read(e.mem, array, r, index));
    if (array && !tb_cell_owned(e, l + index)) tb_mark_raw(e, at + index, 1);
    if (array && !tb_cell_owned(e, r + index)) tb_mark_raw(e, at + (UINT64_C(1) << cls) + index, 1);
  }
  blk_free(e, left); blk_free(e, right);
  return term_blk(array, cls + 1, at);
}

