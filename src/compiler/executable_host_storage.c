/* SPDX-License-Identifier: MPL-2.0
 * Stable host corpus addresses with demand-backed prefixes. Logical capacity
 * is independent of committed pages. The coordinator reserves/releases; CPU
 * allocation extends both prefixes under the VM lock, and GPU readback extends
 * them only after the worker pool is idle and device completion is validated. */
#ifndef _WIN32
#include <sys/mman.h>
#endif

typedef struct {
  void *address;
  size_t capacity, reserved, committed, page_size;
} TBHostStorage;
static TBHostStorage tb_corpus_storage, tb_metadata_storage;

static bool tb_host_storage_reserve(TBHostStorage *storage, size_t bytes) {
  size_t page_size, rounded;
  void *address;
  if (storage->address != NULL || bytes == 0) return false;
#ifdef _WIN32
  SYSTEM_INFO system;
  GetSystemInfo(&system);
  page_size = system.dwPageSize;
#else
  long page = sysconf(_SC_PAGESIZE);
  if (page <= 0) return false;
  page_size = (size_t)page;
#endif
  if (page_size == 0 || bytes > SIZE_MAX - (page_size - 1)) return false;
  rounded = (bytes + page_size - 1) / page_size * page_size;
#ifdef _WIN32
  address = VirtualAlloc(NULL, rounded, MEM_RESERVE, PAGE_NOACCESS);
  if (address == NULL) return false;
#else
  /* /dev/zero keeps anonymous private mappings available with the generated
   * program's strict POSIX feature macros on both Linux and macOS. The mapping
   * owns its reference after mmap, so the descriptor closes immediately. */
  int descriptor = open("/dev/zero", O_RDWR | O_CLOEXEC);
  if (descriptor < 0) return false;
  address = mmap(NULL, rounded, PROT_NONE, MAP_PRIVATE, descriptor, 0);
  (void)close(descriptor);
  if (address == MAP_FAILED) return false;
#endif
  storage->address = address;
  storage->capacity = bytes; storage->reserved = rounded;
  storage->committed = 0; storage->page_size = page_size;
  return true;
}

static bool tb_host_storage_commit(TBHostStorage *storage, size_t bytes) {
  size_t rounded, count;
  unsigned char *start;
  if (storage->address == NULL || bytes > storage->capacity) return false;
  if (bytes <= storage->committed) return true;
  /* Reserve already established that every request through capacity rounds
   * without overflow and lies within the mapping. */
  rounded = (bytes + storage->page_size - 1) / storage->page_size * storage->page_size;
  count = rounded - storage->committed;
  start = (unsigned char *)storage->address + storage->committed;
#ifdef _WIN32
  if (VirtualAlloc(start, count, MEM_COMMIT, PAGE_READWRITE) != start) return false;
#else
  if (mprotect(start, count, PROT_READ | PROT_WRITE) != 0) return false;
#endif
  storage->committed = rounded;
  return true;
}

static void tb_host_storage_release(TBHostStorage *storage) {
  if (storage->address != NULL) {
#ifdef _WIN32
    (void)VirtualFree(storage->address, 0, MEM_RELEASE);
#else
    (void)munmap(storage->address, storage->reserved);
#endif
  }
  memset(storage, 0, sizeof(*storage));
}

INLINE void tb_heap_commit(Loc end) {
  if (end > tb_capacity || end > SIZE_MAX / sizeof(Term))
    err_fail("invalid host corpus commitment");
  size_t bytes = (size_t)end * sizeof(Term);
  /* A partial success remains owned for shutdown. Do not publish tb_bump or
   * imported allocator state until both arrays can hold the entire prefix. */
  if (!tb_host_storage_commit(&tb_corpus_storage, bytes)
      || !tb_host_storage_commit(&tb_metadata_storage, bytes))
    err_fail("host corpus commitment failed");
}
