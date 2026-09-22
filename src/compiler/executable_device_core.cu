/* SPDX-License-Identifier: MPL-2.0
 * CUDA storage adapter for the shared Bend value runtime.
 * Device corpus words and their ownership metadata have the CPU representation.
 * The host transfers both arrays only at an exclusive ownership boundary.
 * Generated task/frame persistence is layered above this storage adapter. */
#if defined(__CUDA_ARCH__) && __CUDA_ARCH__ < 700
#error Bend CUDA requires compute capability 7.0 or newer
#endif
typedef unsigned long long u64;
typedef unsigned int u32;
typedef unsigned char u8;
typedef unsigned char uint8_t;
typedef unsigned long long size_t;
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

#define INLINE static __device__ __forceinline__
#define OUTLINE static __device__ __noinline__
#define TB_NOINLINE __noinline__
#define UINT64_C(value) value##ull
#define UINT32_C(value) value##u
#define UINT64_MAX UINT64_C(18446744073709551615)
#define UINT32_MAX UINT32_C(4294967295)
#define SIZE_MAX UINT64_MAX
#define NULL 0
#define LOC_MASK ((UINT64_C(1) << 40) - 1)
#define RFC_BIT (UINT64_C(1) << 63)
#define TERM_HOLE UINT64_MAX
#define NAT_IMM ((UINT64_C(1) << 48) - 1)
#define TAG_PAK 1u
#define TAG_CTR 2u
#define TAG_CLO 3u
#define TAG_BUF 4u
#define TAG_TSK 5u
#define TAG_ARR 6u
#define HEAP_OFF 1u
#define NCLS_ALL 32u
#define FID_CLO_APPLY 0u
#define FID_IO_EMIT 1u
#define TB_SCRATCH_ACTIVE (UINT64_C(1) << 63)

/* NVRTC receives no host-libc headers. These byte operations run on device
 * pointers; overlapping moves preserve bytes in either direction. */
INLINE void *memcpy(void *destination, const void *source, size_t count) {
  u8 *to = (u8 *)destination;
  const u8 *from = (const u8 *)source;
  for (size_t i = 0; i < count; ++i) to[i] = from[i];
  return destination;
}
INLINE void *memmove(void *destination, const void *source, size_t count) {
  u8 *to = (u8 *)destination;
  const u8 *from = (const u8 *)source;
  if ((u64)to <= (u64)from) {
    for (size_t i = 0; i < count; ++i) to[i] = from[i];
  } else {
    for (size_t i = count; i != 0; --i) to[i - 1] = from[i - 1];
  }
  return destination;
}
INLINE void *memset(void *destination, int value, size_t count) {
  u8 *to = (u8 *)destination;
  for (size_t i = 0; i < count; ++i) to[i] = (u8)value;
  return destination;
}

/* TB_DEVICE_STATE */
static __device__ TBDeviceState *tb_device_state;
static __device__ Corpus tb_memory;
static __device__ u64 *tb_heap_meta;
static __device__ u64 *tb_device_scratch;
static __device__ u32 tb_closure_captures[65536];
#define tb_capacity (tb_device_state->capacity)
#define tb_bump (tb_device_state->bump)
#define tb_live_words (tb_device_state->live_words)
#define tb_live_blocks (tb_device_state->live_blocks)
#define tb_steps (tb_device_state->steps)
#define tb_free_lists (tb_device_state->free_lists)

/* Stop this lane without a CUDA exception. Expected Bend failures must not
 * poison future CUDA invocations in the same process. Failed storage is
 * discarded as a whole, so suspended owners and held locks need no unwinding;
 * every lock waiter observes cancellation before trying again. */
OUTLINE __attribute__((noreturn)) void tb_device_exit(void) {
  asm volatile("exit;");
  for (;;) {}
}
INLINE void tb_device_check_cancelled(void) {
  if (atomicAdd(&tb_device_state->error, 0u) != 0) tb_device_exit();
}
OUTLINE __attribute__((noreturn)) void err_fail(const char *message) {
  if (atomicCAS(&tb_device_state->error, 0u, 1u) == 0) {
    u32 at = 0;
    for (; at + 1 < sizeof(tb_device_state->error_text) && message[at] != 0; ++at)
      tb_device_state->error_text[at] = message[at];
    tb_device_state->error_text[at] = 0;
    __threadfence_system();
  }
  tb_device_exit();
}

/* The CUDA target requires independent thread scheduling (sm_70 or newer).
 * A lane sleeping in the lock loop must not prevent its winning peer running. */
INLINE void tb_vm_acquire(void) {
  for (;;) {
    tb_device_check_cancelled();
    if (atomicCAS(&tb_device_state->lock, 0u, 1u) == 0) break;
    __nanosleep(64);
  }
  __threadfence();
}
INLINE void tb_vm_release(void) {
  __threadfence();
  atomicExch(&tb_device_state->lock, 0u);
}
INLINE void tb_tick(void) {
  tb_vm_acquire();
  if (tb_steps >= tb_device_state->step_limit) err_fail("evaluation budget exhausted");
  ++tb_steps;
  tb_vm_release();
}

/* Shared value helpers need bounded temporary traversal storage. Power-of-two
 * scratch blocks preserve allocations across segment launches and reuse freed
 * blocks. Their links and headers contain offsets, never host addresses. */
OUTLINE void *tb_host_malloc(size_t bytes) {
  if (bytes > SIZE_MAX - 7) return NULL;
  u64 words = ((u64)bytes + 7) / 8;
  /* A zero-size allocation still needs a payload address strictly inside its
   * block so that the same validated free path accepts it. */
  if (words == 0) words = 1;
  if (words > (UINT64_C(1) << 31) - 2) return NULL;
  u32 cls = 1;
  while ((UINT64_C(1) << cls) < words + 2) ++cls;
  u64 span = UINT64_C(1) << cls;
  tb_vm_acquire();
  u64 at = tb_device_state->scratch_free[cls];
  if (at != 0) {
    if (at >= tb_device_state->scratch_bump || tb_device_scratch[at] != cls)
      err_fail("invalid device scratch free list");
    tb_device_state->scratch_free[cls] = tb_device_scratch[at + 1];
  } else {
    at = tb_device_state->scratch_bump;
    if (at < 2 || span > tb_device_state->scratch_capacity
        || at > tb_device_state->scratch_capacity - span) {
      tb_vm_release();
      return NULL;
    }
    tb_device_state->scratch_bump += span;
  }
  tb_device_scratch[at] = TB_SCRATCH_ACTIVE | cls;
  tb_device_scratch[at + 1] = words;
  tb_device_state->scratch_live += span;
  if (tb_device_state->scratch_live > tb_device_state->scratch_peak)
    tb_device_state->scratch_peak = tb_device_state->scratch_live;
  tb_vm_release();
  return tb_device_scratch + at + 2;
}
OUTLINE void *tb_host_calloc(size_t count, size_t bytes) {
  if (bytes != 0 && count > SIZE_MAX / bytes) return NULL;
  void *result = tb_host_malloc(count * bytes);
  if (result != NULL) memset(result, 0, count * bytes);
  return result;
}
OUTLINE void tb_host_free(void *memory) {
  if (memory == NULL) return;
  u64 address = (u64)memory, base = (u64)tb_device_scratch;
  if (address < base || (address - base) % sizeof(u64) != 0)
    err_fail("invalid device scratch pointer");
  u64 offset = (address - base) / sizeof(u64);
  tb_vm_acquire();
  if (offset < 4 || offset >= tb_device_state->scratch_bump)
    err_fail("invalid device scratch pointer");
  u64 at = offset - 2, header = tb_device_scratch[at];
  u32 cls = (u32)(header & ~TB_SCRATCH_ACTIVE);
  if (cls < 1 || cls >= NCLS_ALL || header != (TB_SCRATCH_ACTIVE | cls))
    err_fail("invalid device scratch allocation");
  u64 span = UINT64_C(1) << cls;
  if (span > tb_device_state->scratch_bump - at || tb_device_state->scratch_live < span)
    err_fail("invalid device scratch span");
  tb_device_state->scratch_live -= span;
  tb_device_scratch[at] = cls;
  tb_device_scratch[at + 1] = tb_device_state->scratch_free[cls];
  tb_device_state->scratch_free[cls] = at;
  tb_vm_release();
}
INLINE void *io_mem(void *memory) {
  if (memory == NULL) err_fail("device scratch allocation budget exhausted");
  return memory;
}

/* Generated tables supply this before the shared term-destruction code. */
INLINE u32 fid_arity(Fid fid);

extern "C" __global__ void tb_device_initialize(TBDeviceState *state,
    Term *memory, u64 *metadata, u64 *scratch) {
  if (blockIdx.x != 0 || threadIdx.x != 0) return;
  tb_device_state = state;
  tb_memory = memory;
  tb_heap_meta = metadata;
  tb_device_scratch = scratch;
  state->lock = 0;
  state->error = 0;
  state->error_text[0] = 0;
  if (state->capacity == 0 || state->capacity > LOC_MASK || state->bump < HEAP_OFF
      || state->bump > state->capacity || state->scratch_capacity < 4
      || state->scratch_bump < 2 || state->scratch_bump > state->scratch_capacity)
    err_fail("invalid device corpus boundary");
}
