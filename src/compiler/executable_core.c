/* SPDX-License-Identifier: Apache-2.0
 * Native ABI derived from Bend 2.0.5 comp.ts, Copyright 2026 HigherOrderCO.
 * Bounded CPU implementation: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
 * This first executable-C arena retains values until invocation cleanup. It
 * deliberately does not claim upstream reclamation or its GPU/task optimizer. */
#ifdef _MSC_VER
#define _CRT_SECURE_NO_WARNINGS
#endif
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <limits.h>
#include <errno.h>
#include <setjmp.h>
#include <math.h>
#include <time.h>
#ifdef _WIN32
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <winsock2.h>
#include <windows.h>
#include <process.h>
#include <io.h>
#include <fcntl.h>
#ifdef _MSC_VER
#pragma comment(lib, "ws2_32.lib")
#endif
#define TB_THREAD_LOCAL __declspec(thread)
#define INLINE static __inline
#else
#include <pthread.h>
#include <unistd.h>
#include <poll.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <fcntl.h>
#define TB_THREAD_LOCAL _Thread_local
#define INLINE static inline
#endif
#define OUTLINE static
#ifdef _MSC_VER
#define TB_NORETURN __declspec(noreturn)
#define TB_NOINLINE __declspec(noinline)
#else
#define TB_NORETURN _Noreturn
#define TB_NOINLINE __attribute__((noinline))
#endif
#define THR
#define DEV
#define DEVL
#define CONSTV static const
#ifndef BEND_MAX_STEPS
#define BEND_MAX_STEPS UINT64_C(2000000)
#endif
#ifndef BEND_MAX_DEPTH
#define BEND_MAX_DEPTH 512u
#endif
#ifndef BEND_MAX_FRAMES
#define BEND_MAX_FRAMES 512u
#endif
#ifndef BEND_MAX_ALLOC
#define BEND_MAX_ALLOC UINT64_C(67108864)
#endif
#ifndef BEND_MAX_HOST_ALLOC
#define BEND_MAX_HOST_ALLOC UINT64_C(67108864)
#endif
#ifndef BEND_MAX_HOST_BUFFER
#define BEND_MAX_HOST_BUFFER UINT64_C(8388608)
#endif
#ifndef BEND_MAX_ACTIONS
#define BEND_MAX_ACTIONS 131072u
#endif
#ifndef BEND_MAX_WORKERS
#define BEND_MAX_WORKERS 64u
#endif
#ifndef IO_HOTS
#define IO_HOTS 0
#endif
#ifndef CID_UNIT
#define CID_UNIT BEND_CID_COUNT
#endif
#ifndef CID_EMIT
#define CID_EMIT BEND_CID_COUNT
#endif
#ifndef CID_HALT
#define CID_HALT BEND_CID_COUNT
#endif
#ifndef CID_SCON
#define CID_SCON BEND_CID_COUNT
#endif
#ifndef CID_SNIL
#define CID_SNIL BEND_CID_COUNT
#endif
#ifndef CID_TUPLE
#define CID_TUPLE BEND_CID_COUNT
#endif
#ifndef CID_DONE
#define CID_DONE BEND_CID_COUNT
#endif
#ifndef CID_FAIL
#define CID_FAIL BEND_CID_COUNT
#endif
#ifndef CID_NONE
#define CID_NONE BEND_CID_COUNT
#endif
#ifndef CID_SOME
#define CID_SOME BEND_CID_COUNT
#endif
#ifndef CID_TRUE
#define CID_TRUE BEND_CID_COUNT
#endif
#ifndef CID_FALSE
#define CID_FALSE BEND_CID_COUNT
#endif
#ifndef CID_WCON
#define CID_WCON BEND_CID_COUNT
#endif
#ifndef CID_WNIL
#define CID_WNIL BEND_CID_COUNT
#endif
#ifndef CID_CHR
#define CID_CHR BEND_CID_COUNT
#endif
typedef uint64_t u64;
typedef uint32_t u32;
typedef uint8_t u8;
typedef float f32;
typedef u64 Term;
typedef u64 Loc;
typedef u32 Cls;
typedef u32 Fid;
typedef Term Nat;
typedef Term U32;
typedef Term Reply;
typedef Term *Corpus;
typedef struct { Corpus mem; u64 *alc; } Env;
#define LOC_MASK ((UINT64_C(1) << 40) - 1)
#define TAG_PAK 1u
#define TAG_CTR 2u
#define TAG_CLO 3u
#define TAG_BUF 4u
#define TAG_TSK 5u
#define TAG_ARR 6u
#define TERM_HOLE UINT64_MAX
#define RFC_BIT (UINT64_C(1) << 63)
#define NAT_IMM ((UINT64_C(1) << 48) - 1)
#define HEAP_OFF 1u
#define NCLS_ALL 32u
#ifndef FID_CLO_APPLY
#define FID_CLO_APPLY 0u
#endif
#ifndef FID_IO_EMIT
#define FID_IO_EMIT 1u
#endif

#ifdef _WIN32
typedef SRWLOCK TBMutex;
#define TB_MUTEX_INIT SRWLOCK_INIT
INLINE void tb_lock(TBMutex *mutex) { AcquireSRWLockExclusive(mutex); }
INLINE void tb_unlock(TBMutex *mutex) { ReleaseSRWLockExclusive(mutex); }
#else
typedef pthread_mutex_t TBMutex;
#define TB_MUTEX_INIT PTHREAD_MUTEX_INITIALIZER
INLINE void tb_lock(TBMutex *mutex) { (void)pthread_mutex_lock(mutex); }
INLINE void tb_unlock(TBMutex *mutex) { (void)pthread_mutex_unlock(mutex); }
#endif

typedef struct TBAllocation TBAllocation;
typedef struct TBHost TBHost;
#ifdef _MSC_VER
struct __declspec(align(16)) TBAllocation { TBAllocation *next; size_t size; };
#else
struct TBAllocation { TBAllocation *next; size_t size; max_align_t alignment; };
#endif
struct TBHost {
  TBMutex mutex;
  TBAllocation *allocations;
  u64 bytes;
  u32 references;
  bool stopped;
  bool worker_failed;
#ifdef _WIN32
  bool winsock;
#endif
};
static TB_THREAD_LOCAL TBHost *tb_host_current;
static TB_THREAD_LOCAL jmp_buf *tb_failure_guard;
static Corpus tb_memory;
static Loc tb_bump;
static Loc tb_capacity;
static u64 tb_steps;
static u32 tb_depth;
static u32 tb_frames;

OUTLINE TB_NORETURN void err_fail(const char *message) {
  if (tb_host_current != NULL) {
    tb_lock(&tb_host_current->mutex);
    tb_host_current->worker_failed = true;
    tb_unlock(&tb_host_current->mutex);
  }
  (void)fprintf(stderr, "teamy-bend executable C: %s\n", message);
  if (tb_failure_guard != NULL) longjmp(*tb_failure_guard, 1);
  exit(1);
}
INLINE void tb_tick(void) {
  if (tb_steps++ >= BEND_MAX_STEPS) err_fail("evaluation budget exhausted");
}
OUTLINE void *tb_host_malloc(size_t size) {
  TBHost *host = tb_host_current;
  TBAllocation *allocation;
  u64 charge;
  if (host == NULL || size > BEND_MAX_HOST_BUFFER || size > SIZE_MAX - sizeof(*allocation)) {
    errno = ENOMEM; return NULL;
  }
  charge = (u64)size + sizeof(*allocation);
  tb_lock(&host->mutex);
  if (host->bytes > BEND_MAX_HOST_ALLOC || charge > BEND_MAX_HOST_ALLOC - host->bytes) {
    tb_unlock(&host->mutex); errno = ENOMEM; return NULL;
  }
  allocation = (TBAllocation *)malloc(sizeof(*allocation) + size);
  if (allocation != NULL) {
    allocation->size = size;
    allocation->next = host->allocations;
    host->allocations = allocation;
    host->bytes += charge;
  }
  tb_unlock(&host->mutex);
  return allocation == NULL ? NULL : allocation + 1;
}
OUTLINE void *tb_host_calloc(size_t count, size_t size) {
  void *memory;
  if (size != 0 && count > SIZE_MAX / size) { errno = ENOMEM; return NULL; }
  memory = tb_host_malloc(count * size);
  if (memory != NULL) memset(memory, 0, count * size);
  return memory;
}
OUTLINE void tb_host_free(void *memory) {
  TBHost *host = tb_host_current;
  TBAllocation **at;
  TBAllocation *allocation;
  if (memory == NULL) return;
  if (host == NULL) err_fail("foreign free outside an invocation");
  tb_lock(&host->mutex);
  for (at = &host->allocations; *at != NULL && (void *)(*at + 1) != memory; at = &(*at)->next) {}
  allocation = *at;
  if (allocation != NULL) { *at = allocation->next; host->bytes -= allocation->size + sizeof(*allocation); }
  tb_unlock(&host->mutex);
  if (allocation == NULL) err_fail("foreign free of an untracked allocation");
  free(allocation);
}
OUTLINE void *tb_host_realloc(void *memory, size_t size) {
  TBHost *host = tb_host_current;
  TBAllocation *allocation;
  size_t old_size = 0;
  void *replacement;
  if (memory == NULL) return tb_host_malloc(size);
  if (size == 0) { tb_host_free(memory); return NULL; }
  if (host == NULL) err_fail("foreign realloc outside an invocation");
  tb_lock(&host->mutex);
  for (allocation = host->allocations; allocation != NULL; allocation = allocation->next) {
    if ((void *)(allocation + 1) == memory) { old_size = allocation->size; break; }
  }
  tb_unlock(&host->mutex);
  if (allocation == NULL) err_fail("foreign realloc of an untracked allocation");
  replacement = tb_host_malloc(size);
  if (replacement == NULL) return NULL;
  memcpy(replacement, memory, old_size < size ? old_size : size);
  tb_host_free(memory);
  return replacement;
}
OUTLINE void tb_host_release(TBHost *host) {
  TBAllocation *allocation;
  bool final;
  tb_lock(&host->mutex);
  final = --host->references == 0;
  tb_unlock(&host->mutex);
  if (!final) return;
  while ((allocation = host->allocations) != NULL) {
    host->allocations = allocation->next;
    free(allocation);
  }
#ifdef _WIN32
  if (host->winsock) (void)WSACleanup();
#endif
#ifndef _WIN32
  (void)pthread_mutex_destroy(&host->mutex);
#endif
  free(host);
}
INLINE void *io_mem(void *memory) {
  if (memory == NULL) err_fail("host allocation budget exhausted");
  return memory;
}
/* Generated functions keep every temporary Term in these bounded heap frames.
 * Small non-inlined wrappers allocate before entering a body and release after
 * every normal return. The host owner releases frames abandoned by longjmp.
 * Count conversion/printer/definition calls as well as closure applications. */
OUTLINE Term *tb_frame_push(size_t slots) {
  if (tb_frames >= BEND_MAX_FRAMES) err_fail("generated frame depth budget exhausted");
  ++tb_frames;
  if (slots == 0) return NULL;
  if (slots > SIZE_MAX / sizeof(Term)) err_fail("generated frame allocation overflow");
  return (Term *)io_mem(tb_host_calloc(slots, sizeof(Term)));
}
OUTLINE void tb_frame_pop(Term *values) {
  if (tb_frames == 0) err_fail("unbalanced generated frame");
  tb_host_free(values);
  --tb_frames;
}
INLINE Cls cls_fit(u32 count) {
  Cls cls = 0;
  while ((UINT64_C(1) << cls) < count) ++cls;
  return cls;
}
INLINE Loc heap_alloc(Env e, Cls cls) {
  Loc at = tb_bump;
  u64 size;
  if (e.mem != tb_memory || cls >= 32) err_fail("invalid heap allocation");
  size = UINT64_C(1) << cls;
  if (size > tb_capacity || at > tb_capacity - size) err_fail("VM allocation budget exhausted");
  tb_bump += size;
  memset(e.mem + at, 0, (size_t)size * sizeof(Term));
  return at;
}
INLINE void tb_span(Env e, Loc at, u64 count) {
  if (e.mem != tb_memory || at < HEAP_OFF || at > tb_bump || count > tb_bump - at)
    err_fail("invalid native heap span");
}
INLINE void heap_free(Env e, Cls cls, Loc at) { (void)e; (void)cls; (void)at; }
INLINE void spare_free(Env e, Cls cls, Loc at) { (void)e; (void)cls; (void)at; }
INLINE Term term_make(u64 tag, u64 aux, u64 loc) {
  if (tag > 127 || aux > 65535 || loc > LOC_MASK) err_fail("native term fields overflow");
  return (tag << 56) | (aux << 40) | loc;
}
#define term_pak(cid,loc) term_make(TAG_PAK,(cid),(loc))
#define term_ctr(cid,loc) term_make(TAG_CTR,(cid),(loc))
#define term_clo(fid,loc) term_make(TAG_CLO,(fid),(loc))
#define term_buf(cls,loc) term_make(TAG_BUF,(cls),(loc))
#define term_tsk(fid,loc) term_make(TAG_TSK,(fid),(loc))
INLINE u64 term_tag(Term term) { return (term >> 56) & 127; }
INLINE u64 term_aux(Term term) { return (term >> 40) & 65535; }
INLINE Loc term_loc(Term term) { return term & LOC_MASK; }
INLINE bool term_rfc(Term term) { return (term & RFC_BIT) != 0; }
INLINE bool term_triv(Term term) { return term_tag(term) <= TAG_PAK || term == TERM_HOLE; }
INLINE Loc term_peek(Env e, Term term) {
  if (term_rfc(term)) err_fail("external reference-count cells are unsupported");
  tb_span(e, term_loc(term), 1);
  return term_loc(term);
}
INLINE Term rfc_seal(Env e, Term term) { (void)e; return term; }
INLINE Term term_keep(Env e, Term term) { (void)e; return term; }
INLINE void term_drop(Env e, Term term) { (void)e; (void)term; }
INLINE void term_sink(Env e, Term term) { term_drop(e, term); }
INLINE u32 cid_arity(u32 cid) {
  if (cid >= BEND_CID_COUNT) err_fail("unknown constructor id");
  return CID_ARITY_T[cid];
}
INLINE Loc ctr_take(Env e, Term term, u32 count, Term *fields) {
  Loc at;
  if (term_tag(term) == TAG_PAK) {
    if (count > 1) err_fail("packed constructor has too many fields");
    if (count == 1) fields[0] = term_loc(term);
    return 0;
  }
  if (term_tag(term) != TAG_CTR) err_fail("constructor expected");
  at = term_peek(e, term);
  tb_span(e, at, count);
  memcpy(fields, e.mem + at, count * sizeof(Term));
  return at;
}
INLINE Term tb_construct(Env e, u32 cid, u32 count, const Term *fields, bool packed) {
  Loc at;
  if (cid >= BEND_CID_COUNT || count != cid_arity(cid)) err_fail("constructor arity mismatch");
  if (packed) {
    if (count > 1 || (count != 0 && fields[0] > LOC_MASK)) err_fail("invalid packed constructor");
    return term_pak(cid, count == 0 ? 0 : fields[0]);
  }
  at = heap_alloc(e, cls_fit(count));
  if (count != 0) memcpy(e.mem + at, fields, count * sizeof(Term));
  return term_ctr(cid, at);
}
INLINE void tb_fields(Env e, Term value, u32 cid, u32 count, bool packed, Term *out) {
  if (term_aux(value) != cid || count != cid_arity(cid) || term_tag(value) != (packed ? TAG_PAK : TAG_CTR))
    err_fail("constructor representation mismatch");
  (void)ctr_take(e, value, count, out);
}
INLINE Term io_hand(u64 handle) {
  if (handle >> 56) err_fail("host handle exceeds native 56-bit representation");
  return term_pak(handle >> 40, handle & LOC_MASK);
}
INLINE u64 io_hand_v(Term handle) {
  if (term_tag(handle) != TAG_PAK) err_fail("native handle expected");
  return (term_aux(handle) << 40) | term_loc(handle);
}
INLINE f32 f32_unbox(u64 bits) { u32 raw = (u32)bits; f32 value; memcpy(&value, &raw, 4); return value; }
INLINE u64 f32_rewrap(f32 value) { u32 bits; memcpy(&bits, &value, 4); return bits; }
INLINE Nat nat_chk(Env e, Nat value) { (void)e; if (value > NAT_IMM) err_fail("Nat exceeds largest immediate"); return value; }
INLINE Nat nat_mul(Env e, Nat left, Nat right) {
  if (right != 0 && left > NAT_IMM / right) err_fail("Nat multiplication overflow");
  return nat_chk(e, left * right);
}
#define U32_BIN(a,op,b) ((u64)((u32)(a) op (u32)(b)))
INLINE Term term_word(Env e, Term word) {
  u32 value = 0;
  u32 index = 0;
  while (term_aux(word) == CID_WCON && index < 32) {
    Loc at = term_peek(e, word);
    tb_span(e, at, 2);
    value |= ((u32)e.mem[at] & 1) << index++;
    word = e.mem[at + 1];
  }
  return value;
}
INLINE Term tb_word(Env e, u32 value) {
#if CID_WNIL >= BEND_CID_COUNT || CID_WCON >= BEND_CID_COUNT
  err_fail("Word representation unavailable");
#endif
  Term word = term_pak(CID_WNIL, 0);
  for (u32 index = 32; index != 0; --index) {
    Loc at = heap_alloc(e, 1);
    e.mem[at] = (value >> (index - 1)) & 1; e.mem[at + 1] = word;
    word = term_ctr(CID_WCON, at);
  }
  return word;
}

typedef Term (*BendClosureFn)(Env, const Term *, Term);
static BendClosureFn tb_closure_functions[65536];
static u32 tb_closure_captures[65536];
OUTLINE void tb_register_closure(u32 fid, BendClosureFn function, u32 count) {
  if (fid < 2 || fid >= 65536 || function == NULL || count > 255) err_fail("invalid closure registration");
  if (tb_closure_functions[fid] != NULL && tb_closure_functions[fid] != function) err_fail("duplicate closure registration");
  tb_closure_functions[fid] = function;
  tb_closure_captures[fid] = count;
}
INLINE Term tb_closure(Env e, u32 fid, u32 count, const Term *captures) {
  Loc at = 0;
  if (fid < 2 || fid >= 65536 || tb_closure_functions[fid] == NULL || count != tb_closure_captures[fid])
    err_fail("unregistered closure");
  if (count != 0) { at = heap_alloc(e, cls_fit(count)); memcpy(e.mem + at, captures, count * sizeof(Term)); }
  return term_clo(fid, at);
}
OUTLINE Term tb_apply(Env e, Term closure, Term argument) {
  u32 fid;
  Term result;
  const Term *captures = NULL;
  tb_tick();
  if (term_tag(closure) != TAG_CLO) err_fail("application of a non-function");
  fid = (u32)term_aux(closure);
  if (fid == FID_IO_EMIT) return term_pak(CID_EMIT, 0);
  if (tb_closure_functions[fid] == NULL) err_fail("unregistered closure application");
  if (++tb_depth > BEND_MAX_DEPTH) err_fail("call depth budget exhausted");
  if (tb_closure_captures[fid] != 0) { Loc at = term_peek(e, closure); tb_span(e, at, tb_closure_captures[fid]); captures = e.mem + at; }
  result = tb_closure_functions[fid](e, captures, argument);
  --tb_depth;
  return result;
}
INLINE Loc task_node(Env e, Fid fid, Term continuation, u32 index, u32 remaining) {
  Loc at;
  (void)continuation; (void)index; (void)remaining;
  if (fid != FID_CLO_APPLY) err_fail("foreign task id is unsupported");
  at = heap_alloc(e, 1);
  return at;
}
INLINE Term corpus_eval(Corpus memory, Term task) {
  Env e = {memory, NULL}; Loc at;
  if (term_tag(task) != TAG_TSK || term_aux(task) != FID_CLO_APPLY) err_fail("foreign task is unsupported");
  at = term_peek(e, task); tb_span(e, at, 2);
  return tb_apply(e, memory[at], memory[at + 1]);
}

INLINE Term term_blk(bool array, Cls cls, Loc loc) { return term_make(array ? TAG_ARR : TAG_BUF, cls, loc); }
INLINE Cls blk_cls(Term block) { return (Cls)term_aux(block); }
INLINE Cls buf_wcls(Cls cls) { return cls == 0 ? 0 : cls - 1; }
INLINE Cls blk_span(Term block) { return term_tag(block) == TAG_ARR ? blk_cls(block) : buf_wcls(blk_cls(block)); }
INLINE void blk_free(Env e, Term block) { (void)e; (void)block; }
INLINE Term blk_read(Corpus memory, bool array, Loc at, u32 index) {
  Env e = {memory, NULL};
  tb_span(e, at, (array ? (u64)index : (u64)index / 2) + 1);
  return array ? memory[at + index] : (u32)(memory[at + index / 2] >> ((index & 1) * 32));
}
INLINE void blk_write(Corpus memory, bool array, Loc at, u32 index, Term value) {
  Env e = {memory, NULL}; u32 shift = (index & 1) * 32;
  tb_span(e, at, (array ? (u64)index : (u64)index / 2) + 1);
  if (array) memory[at + index] = value;
  else memory[at + index / 2] = (memory[at + index / 2] & ~(UINT64_C(0xffffffff) << shift)) | ((u64)(u32)value << shift);
}
INLINE u32 blk_at(Term block, U32 index, u32 lgs) {
  Cls cls = blk_cls(block);
  if (cls < lgs || cls > 17) err_fail("invalid array layout");
  return ((u32)index & ((1u << (cls - lgs)) - 1)) << lgs;
}
INLINE Term blk_new(Env e, bool array, Nat depth, u32 lgs, u32 count, Term *values) {
  Cls cls; Loc at; u32 stride;
  if (depth > 17 || lgs > 17 || depth + lgs > 17 || count > (1u << lgs)) err_fail("array budget exhausted");
  cls = (Cls)depth + lgs; stride = 1u << lgs;
  at = heap_alloc(e, array ? cls : buf_wcls(cls));
  for (u32 index = 0; index < (1u << cls); ++index) blk_write(e.mem, array, at, index, index % stride < count ? values[index % stride] : 0);
  return term_blk(array, cls, at);
}
INLINE Term blk_copy(Env e, Term block) {
  Cls cls = blk_span(block); Loc at; Loc from = term_peek(e, block);
  if (blk_cls(block) > 17) err_fail("array budget exhausted");
  tb_span(e, from, UINT64_C(1) << cls);
  at = heap_alloc(e, cls); memcpy(e.mem + at, e.mem + from, ((size_t)1 << cls) * sizeof(Term));
  return term_blk(term_tag(block) == TAG_ARR, blk_cls(block), at);
}
INLINE Term blk_half(Env e, Term block, u32 high) {
  bool array = term_tag(block) == TAG_ARR; Cls cls = blk_cls(block); Loc at; Loc from = term_peek(e, block);
  if (cls == 0 || cls > 17 || high > 1) err_fail("invalid array half");
  --cls; at = heap_alloc(e, array ? cls : buf_wcls(cls));
  for (u32 index = 0; index < (1u << cls); ++index) blk_write(e.mem, array, at, index, blk_read(e.mem, array, from, (high << cls) + index));
  return term_blk(array, cls, at);
}
INLINE Term blk_node(Env e, Term left, Term right) {
  bool array = term_tag(left) == TAG_ARR; Cls cls = blk_cls(left); Loc at;
  if (cls != blk_cls(right) || term_tag(left) != term_tag(right) || cls >= 17) err_fail("invalid array concatenation");
  at = heap_alloc(e, array ? cls + 1 : buf_wcls(cls + 1));
  for (u32 index = 0; index < (1u << cls); ++index) {
    blk_write(e.mem, array, at, index, blk_read(e.mem, array, term_loc(left), index));
    blk_write(e.mem, array, at, (1u << cls) + index, blk_read(e.mem, array, term_loc(right), index));
  }
  return term_blk(array, cls + 1, at);
}

/* Implemented by executable_io.c after this core. */
static void tb_io_initialize(void);
static void tb_io_shutdown(void);
static int io_loop(Env e, Term main);
OUTLINE int tb_run(Term (*entry)(Env), int is_io, void (*show)(Env, Term), void (*initialize)(Env)) {
  TBHost *host = (TBHost *)calloc(1, sizeof(TBHost));
  jmp_buf guard;
  volatile int result = 1;
  Env e;
  if (host == NULL) { (void)fprintf(stderr, "host initialization failed\n"); return 1; }
#ifdef _WIN32
  InitializeSRWLock(&host->mutex);
#else
  if (pthread_mutex_init(&host->mutex, NULL) != 0) { free(host); return 1; }
#endif
  host->references = 1;
  tb_host_current = host; tb_failure_guard = &guard;
  tb_capacity = BEND_MAX_ALLOC / sizeof(Term);
  tb_memory = (Corpus)calloc((size_t)tb_capacity, sizeof(Term));
  e.mem = tb_memory; e.alc = NULL;
  tb_bump = HEAP_OFF; tb_steps = 0; tb_depth = 0; tb_frames = 0;
  memset(tb_closure_functions, 0, sizeof(tb_closure_functions));
  if (setjmp(guard) == 0) {
    Term main;
    if (tb_memory == NULL || tb_capacity <= HEAP_OFF) err_fail("VM initialization failed");
    tb_io_initialize();
    if (initialize != NULL) initialize(e);
    if (entry == NULL) { (void)fputs("All terms check.\n", stdout); result = 0; }
    else {
      main = entry(e);
      if (is_io) result = io_loop(e, main);
      else { if (show != NULL) show(e, main); result = 0; }
    }
  }
  tb_io_shutdown();
  free(tb_memory); tb_memory = NULL;
  tb_failure_guard = NULL; tb_host_current = NULL;
  tb_host_release(host);
  return result;
}
