/* SPDX-License-Identifier: MPL-2.0
 * Dynamically loaded CUDA host transport. The coordinator owns this session;
 * CPU workers must be parked before transferring the Bend corpus. Device
 * pointers stay separate from host pointers: no managed-memory assumptions.
 * Stream/module/buffers persist between calls. Successful buffer growth
 * discards old contents; allocation failure preserves the old allocation.
 * A CUDA execution error requires shutdown rather than retrying consumed work.
 * This adapter never longjmps. Callers may copy its diagnostic, shut it down,
 * and then report an evaluator failure without leaking CUDA resources. */
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <wchar.h>
#ifdef _WIN32
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#define TB_CUDA_CALL __stdcall
typedef HMODULE TBCudaLibrary;
#else
#include <dlfcn.h>
#define TB_CUDA_CALL
typedef void *TBCudaLibrary;
#endif
#ifndef BEND_MAX_GPU_ALLOC
#define BEND_MAX_GPU_ALLOC (BEND_MAX_ALLOC > UINT64_MAX / 3 ? UINT64_MAX : BEND_MAX_ALLOC * UINT64_C(3))
#endif
#define TB_CUDA_BUFFERS 8u
enum { TB_CUDA_ERROR_NONE, TB_CUDA_ERROR_UNAVAILABLE, TB_CUDA_ERROR_COMPILE, TB_CUDA_ERROR_RUNTIME };
typedef struct {
  int device, major, minor, driver_version, nvrtc_major, nvrtc_minor;
  int max_block, max_grid;
  char device_name[256];
  uint64_t compilations, allocations, launches, upload_bytes, download_bytes;
} TBCudaInfo;
typedef struct { uint64_t address; size_t capacity; } TBCudaBuffer;
typedef struct {
  TBCudaLibrary driver, compiler;
  void *context, *module, *stream;
  char *source;
  char error[2048];
  int error_class;
  bool initialized;
  TBCudaBuffer buffers[TB_CUDA_BUFFERS];
  TBCudaInfo info;
  int (TB_CUDA_CALL *cuInit)(unsigned int);
  int (TB_CUDA_CALL *cuDriverGetVersion)(int *);
  int (TB_CUDA_CALL *cuDeviceGet)(int *, int);
  int (TB_CUDA_CALL *cuDeviceGetName)(char *, int, int);
  int (TB_CUDA_CALL *cuDeviceGetAttribute)(int *, int, int);
  int (TB_CUDA_CALL *cuCtxCreate)(void **, unsigned int, int);
  int (TB_CUDA_CALL *cuCtxDestroy)(void *);
  int (TB_CUDA_CALL *cuCtxPushCurrent)(void *);
  int (TB_CUDA_CALL *cuCtxPopCurrent)(void **);
  int (TB_CUDA_CALL *cuCtxSynchronize)(void);
  int (TB_CUDA_CALL *cuModuleLoadData)(void **, const void *);
  int (TB_CUDA_CALL *cuModuleUnload)(void *);
  int (TB_CUDA_CALL *cuModuleGetFunction)(void **, void *, const char *);
  int (TB_CUDA_CALL *cuStreamCreate)(void **, unsigned int);
  int (TB_CUDA_CALL *cuStreamDestroy)(void *);
  int (TB_CUDA_CALL *cuStreamSynchronize)(void *);
  int (TB_CUDA_CALL *cuMemAlloc)(uint64_t *, size_t);
  int (TB_CUDA_CALL *cuMemFree)(uint64_t);
  int (TB_CUDA_CALL *cuMemcpyHtoD)(uint64_t, const void *, size_t);
  int (TB_CUDA_CALL *cuMemcpyDtoH)(void *, uint64_t, size_t);
  int (TB_CUDA_CALL *cuLaunchKernel)(void *, unsigned int, unsigned int, unsigned int,
    unsigned int, unsigned int, unsigned int, unsigned int, void *, void **, void **);
  int (TB_CUDA_CALL *cuGetErrorString)(int, const char **);
  int (*nvrtcVersion)(int *, int *);
  int (*nvrtcCreateProgram)(void **, const char *, const char *, int, const char *const *, const char *const *);
  int (*nvrtcDestroyProgram)(void **);
  int (*nvrtcCompileProgram)(void *, int, const char *const *);
  int (*nvrtcGetProgramLogSize)(void *, size_t *);
  int (*nvrtcGetProgramLog)(void *, char *);
  int (*nvrtcGetCUBINSize)(void *, size_t *);
  int (*nvrtcGetCUBIN)(void *, char *);
  const char *(*nvrtcGetErrorString)(int);
} TBCuda;
static TBCuda tb_cuda;
static void tb_cuda_counter(uint64_t *counter, uint64_t amount) {
  /* Diagnostic counters saturate; they never alter execution semantics. */
  *counter = *counter > UINT64_MAX - amount ? UINT64_MAX : *counter + amount;
}
static bool tb_cuda_message(int category, const char *message) {
  tb_cuda.error_class = category;
  (void)snprintf(tb_cuda.error, sizeof(tb_cuda.error), "%s", message);
  return false;
}
static bool tb_cuda_status(int result, const char *operation, int category) {
  const char *description = NULL;
  if (result == 0) return true;
  if (tb_cuda.cuGetErrorString != NULL) (void)tb_cuda.cuGetErrorString(result, &description);
  tb_cuda.error_class = category;
  (void)snprintf(tb_cuda.error, sizeof(tb_cuda.error), "%s: CUDA error %d (%s)",
    operation, result, description == NULL ? "unavailable diagnostic" : description);
  return false;
}
static bool tb_cuda_compile_status(int result, const char *operation) {
  if (result == 0) return true;
  tb_cuda.error_class = TB_CUDA_ERROR_COMPILE;
  (void)snprintf(tb_cuda.error, sizeof(tb_cuda.error), "%s: %s", operation, tb_cuda.nvrtcGetErrorString(result));
  return false;
}
static void tb_cuda_cleanup_status(int result, const char *operation) {
  if (result != 0 && tb_cuda.error_class == TB_CUDA_ERROR_NONE)
    (void)tb_cuda_status(result, operation, TB_CUDA_ERROR_RUNTIME);
}
static bool tb_cuda_symbol(TBCudaLibrary library, const char *name, void *destination, size_t size) {
#ifdef _WIN32
  FARPROC symbol = GetProcAddress(library, name);
#else
  void *symbol = dlsym(library, name);
#endif
  if (symbol == NULL || size != sizeof(symbol)) {
    tb_cuda.error_class = TB_CUDA_ERROR_UNAVAILABLE;
    (void)snprintf(tb_cuda.error, sizeof(tb_cuda.error), "CUDA library entry is unavailable: %s", name);
    return false;
  }
  /* POSIX and Windows dynamic-loader ABIs represent these pointers equally.
   * memcpy avoids nonportable ISO C object/function pointer casts. */
  memcpy(destination, &symbol, size);
  return true;
}
#ifdef _WIN32
static HMODULE tb_cuda_find_nvrtc(const wchar_t *root, const wchar_t *suffix) {
  wchar_t *pattern = (wchar_t *)malloc(32768u * sizeof(wchar_t));
  wchar_t *chosen = (wchar_t *)calloc(32768u, sizeof(wchar_t));
  WIN32_FIND_DATAW entry;
  HANDLE search = INVALID_HANDLE_VALUE;
  HMODULE library = NULL;
  if (pattern == NULL || chosen == NULL) goto done;
  if (swprintf(pattern, 32768u, L"%ls%ls\\nvrtc64_*.dll", root, suffix) < 0) goto done;
  search = FindFirstFileW(pattern, &entry);
  if (search == INVALID_HANDLE_VALUE) goto done;
  do {
    if ((entry.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0 && wcscmp(entry.cFileName, chosen) > 0)
      (void)wmemcpy(chosen, entry.cFileName, wcslen(entry.cFileName) + 1);
  } while (FindNextFileW(search, &entry));
  if (chosen[0] != 0 && swprintf(pattern, 32768u, L"%ls%ls\\%ls", root, suffix, chosen) >= 0)
    library = LoadLibraryExW(pattern, NULL, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_DEFAULT_DIRS);
done:
  if (search != INVALID_HANDLE_VALUE) (void)FindClose(search);
  free(chosen); free(pattern);
  return library;
}
#endif
static bool tb_cuda_libraries(void) {
#ifdef _WIN32
  wchar_t *root;
  DWORD length;
  tb_cuda.driver = LoadLibraryExW(L"nvcuda.dll", NULL, LOAD_LIBRARY_SEARCH_SYSTEM32);
  if (tb_cuda.driver == NULL) return tb_cuda_message(TB_CUDA_ERROR_UNAVAILABLE, "CUDA driver library is unavailable");
  root = (wchar_t *)malloc(32768u * sizeof(wchar_t));
  if (root == NULL) return tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "CUDA discovery allocation failed");
  length = GetEnvironmentVariableW(L"CUDA_PATH", root, 32768u);
  if (length != 0 && length < 32768u) {
    tb_cuda.compiler = tb_cuda_find_nvrtc(root, L"\\bin");
    if (tb_cuda.compiler == NULL) tb_cuda.compiler = tb_cuda_find_nvrtc(root, L"\\bin\\x64");
  }
  free(root);
  /* Stable NVRTC DLL names from CUDA's major-version compatibility policy.
   * The toolkit path above discovers versions without compiled-in locations. */
  if (tb_cuda.compiler == NULL) tb_cuda.compiler = LoadLibraryExW(L"nvrtc64_130_0.dll", NULL, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS);
  if (tb_cuda.compiler == NULL) tb_cuda.compiler = LoadLibraryExW(L"nvrtc64_120_0.dll", NULL, LOAD_LIBRARY_SEARCH_DEFAULT_DIRS);
#else
  tb_cuda.driver = dlopen("libcuda.so.1", RTLD_NOW | RTLD_LOCAL);
  if (tb_cuda.driver == NULL) return tb_cuda_message(TB_CUDA_ERROR_UNAVAILABLE, "CUDA driver library is unavailable");
  tb_cuda.compiler = dlopen("libnvrtc.so", RTLD_NOW | RTLD_LOCAL);
  if (tb_cuda.compiler == NULL) tb_cuda.compiler = dlopen("libnvrtc.so.13", RTLD_NOW | RTLD_LOCAL);
  if (tb_cuda.compiler == NULL) tb_cuda.compiler = dlopen("libnvrtc.so.12", RTLD_NOW | RTLD_LOCAL);
#endif
  if (tb_cuda.compiler == NULL) return tb_cuda_message(TB_CUDA_ERROR_UNAVAILABLE, "NVRTC library is unavailable; configure the CUDA toolkit library path");
#define TB_CUDA_LOAD(field, name) if (!tb_cuda_symbol(tb_cuda.driver, name, &tb_cuda.field, sizeof(tb_cuda.field))) return false
  TB_CUDA_LOAD(cuInit, "cuInit"); TB_CUDA_LOAD(cuDriverGetVersion, "cuDriverGetVersion");
  TB_CUDA_LOAD(cuDeviceGet, "cuDeviceGet"); TB_CUDA_LOAD(cuDeviceGetName, "cuDeviceGetName");
  TB_CUDA_LOAD(cuDeviceGetAttribute, "cuDeviceGetAttribute"); TB_CUDA_LOAD(cuGetErrorString, "cuGetErrorString");
  TB_CUDA_LOAD(cuCtxCreate, "cuCtxCreate_v2"); TB_CUDA_LOAD(cuCtxDestroy, "cuCtxDestroy_v2");
  TB_CUDA_LOAD(cuCtxPushCurrent, "cuCtxPushCurrent_v2"); TB_CUDA_LOAD(cuCtxPopCurrent, "cuCtxPopCurrent_v2");
  TB_CUDA_LOAD(cuCtxSynchronize, "cuCtxSynchronize");
  TB_CUDA_LOAD(cuModuleLoadData, "cuModuleLoadData"); TB_CUDA_LOAD(cuModuleUnload, "cuModuleUnload");
  TB_CUDA_LOAD(cuModuleGetFunction, "cuModuleGetFunction"); TB_CUDA_LOAD(cuStreamCreate, "cuStreamCreate");
  TB_CUDA_LOAD(cuStreamDestroy, "cuStreamDestroy_v2"); TB_CUDA_LOAD(cuStreamSynchronize, "cuStreamSynchronize");
  TB_CUDA_LOAD(cuMemAlloc, "cuMemAlloc_v2"); TB_CUDA_LOAD(cuMemFree, "cuMemFree_v2");
  TB_CUDA_LOAD(cuMemcpyHtoD, "cuMemcpyHtoD_v2"); TB_CUDA_LOAD(cuMemcpyDtoH, "cuMemcpyDtoH_v2");
  TB_CUDA_LOAD(cuLaunchKernel, "cuLaunchKernel");
#undef TB_CUDA_LOAD
#define TB_NVRTC_LOAD(field) if (!tb_cuda_symbol(tb_cuda.compiler, #field, &tb_cuda.field, sizeof(tb_cuda.field))) return false
  TB_NVRTC_LOAD(nvrtcVersion); TB_NVRTC_LOAD(nvrtcCreateProgram); TB_NVRTC_LOAD(nvrtcDestroyProgram);
  TB_NVRTC_LOAD(nvrtcCompileProgram); TB_NVRTC_LOAD(nvrtcGetProgramLogSize); TB_NVRTC_LOAD(nvrtcGetProgramLog);
  TB_NVRTC_LOAD(nvrtcGetCUBINSize); TB_NVRTC_LOAD(nvrtcGetCUBIN); TB_NVRTC_LOAD(nvrtcGetErrorString);
#undef TB_NVRTC_LOAD
  return true;
}
static bool tb_cuda_push(void) {
  if (!tb_cuda.initialized) return tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "CUDA session is not initialized");
  return tb_cuda_status(tb_cuda.cuCtxPushCurrent(tb_cuda.context), "cuCtxPushCurrent", TB_CUDA_ERROR_RUNTIME);
}
static bool tb_cuda_pop(bool success) {
  void *context = NULL;
  int status = tb_cuda.cuCtxPopCurrent(&context);
  if (!success) return false;
  if (!tb_cuda_status(status, "cuCtxPopCurrent", TB_CUDA_ERROR_RUNTIME)) return false;
  return context == tb_cuda.context || tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "CUDA current-context ownership mismatch");
}
static inline const char *tb_cuda_error(void) { return tb_cuda.error; }
static inline int tb_cuda_error_class(void) { return tb_cuda.error_class; }
static inline const TBCudaInfo *tb_cuda_info(void) { return &tb_cuda.info; }
static inline void tb_cuda_shutdown(void) {
  if (tb_cuda.context != NULL) {
    int pushed_status = tb_cuda.cuCtxPushCurrent(tb_cuda.context);
    bool pushed = pushed_status == 0;
    tb_cuda_cleanup_status(pushed_status, "cuCtxPushCurrent during shutdown");
    if (pushed) {
      if (tb_cuda.stream != NULL) tb_cuda_cleanup_status(tb_cuda.cuStreamSynchronize(tb_cuda.stream), "cuStreamSynchronize during shutdown");
      for (unsigned int i = 0; i < TB_CUDA_BUFFERS; ++i)
        if (tb_cuda.buffers[i].address != 0) tb_cuda_cleanup_status(tb_cuda.cuMemFree(tb_cuda.buffers[i].address), "cuMemFree during shutdown");
      if (tb_cuda.stream != NULL) tb_cuda_cleanup_status(tb_cuda.cuStreamDestroy(tb_cuda.stream), "cuStreamDestroy during shutdown");
      if (tb_cuda.module != NULL) tb_cuda_cleanup_status(tb_cuda.cuModuleUnload(tb_cuda.module), "cuModuleUnload during shutdown");
      { void *previous = NULL; tb_cuda_cleanup_status(tb_cuda.cuCtxPopCurrent(&previous), "cuCtxPopCurrent during shutdown"); }
    }
    /* Destroying the owned context also releases allocations after a device
     * failure that prevents normal per-resource cleanup. Never reset a device. */
    tb_cuda_cleanup_status(tb_cuda.cuCtxDestroy(tb_cuda.context), "cuCtxDestroy during shutdown");
  }
  tb_cuda.context = NULL; tb_cuda.module = NULL; tb_cuda.stream = NULL;
  memset(tb_cuda.buffers, 0, sizeof(tb_cuda.buffers));
  free(tb_cuda.source); tb_cuda.source = NULL; tb_cuda.initialized = false;
#ifdef _WIN32
  if (tb_cuda.compiler != NULL) (void)FreeLibrary(tb_cuda.compiler);
  if (tb_cuda.driver != NULL) (void)FreeLibrary(tb_cuda.driver);
#else
  if (tb_cuda.compiler != NULL) (void)dlclose(tb_cuda.compiler);
  if (tb_cuda.driver != NULL) (void)dlclose(tb_cuda.driver);
#endif
  tb_cuda.compiler = NULL; tb_cuda.driver = NULL;
  /* Diagnostics and counters survive shutdown for the caller's receipt. */
}
static inline bool tb_cuda_initialize(const char *source) {
  void *program = NULL;
  char *image = NULL;
  size_t size = 0, source_size = 0;
  bool current = false, success = false;
  char architecture[64];
  const char *options[3];
  if (source == NULL) return tb_cuda_message(TB_CUDA_ERROR_COMPILE, "CUDA device source is missing");
  if (tb_cuda.initialized) return strcmp(tb_cuda.source, source) == 0 || tb_cuda_message(TB_CUDA_ERROR_COMPILE, "CUDA session already owns a different device program");
  tb_cuda.error[0] = 0; tb_cuda.error_class = TB_CUDA_ERROR_NONE;
  while (source_size < BEND_MAX_HOST_BUFFER && source[source_size] != 0) ++source_size;
  if (source_size == BEND_MAX_HOST_BUFFER) return tb_cuda_message(TB_CUDA_ERROR_COMPILE, "CUDA device source exceeds host buffer limit");
  if (!tb_cuda_libraries()) goto done;
  if (!tb_cuda_status(tb_cuda.cuInit(0), "cuInit", TB_CUDA_ERROR_UNAVAILABLE)
      || !tb_cuda_status(tb_cuda.cuDeviceGet(&tb_cuda.info.device, 0), "cuDeviceGet", TB_CUDA_ERROR_UNAVAILABLE)
      || !tb_cuda_status(tb_cuda.cuDriverGetVersion(&tb_cuda.info.driver_version), "cuDriverGetVersion", TB_CUDA_ERROR_UNAVAILABLE)
      || !tb_cuda_status(tb_cuda.cuDeviceGetName(tb_cuda.info.device_name, (int)sizeof(tb_cuda.info.device_name), tb_cuda.info.device), "cuDeviceGetName", TB_CUDA_ERROR_UNAVAILABLE)) goto done;
  if (!tb_cuda_status(tb_cuda.cuDeviceGetAttribute(&tb_cuda.info.major, 75, tb_cuda.info.device), "compute capability major", TB_CUDA_ERROR_UNAVAILABLE)
      || !tb_cuda_status(tb_cuda.cuDeviceGetAttribute(&tb_cuda.info.minor, 76, tb_cuda.info.device), "compute capability minor", TB_CUDA_ERROR_UNAVAILABLE)
      || !tb_cuda_status(tb_cuda.cuDeviceGetAttribute(&tb_cuda.info.max_block, 1, tb_cuda.info.device), "maximum block size", TB_CUDA_ERROR_UNAVAILABLE)
      || !tb_cuda_status(tb_cuda.cuDeviceGetAttribute(&tb_cuda.info.max_grid, 5, tb_cuda.info.device), "maximum grid size", TB_CUDA_ERROR_UNAVAILABLE)
      || !tb_cuda_compile_status(tb_cuda.nvrtcVersion(&tb_cuda.info.nvrtc_major, &tb_cuda.info.nvrtc_minor), "nvrtcVersion")) goto done;
  if (tb_cuda.info.major < 7 || tb_cuda.info.max_block <= 0 || tb_cuda.info.max_grid <= 0) {
    (void)tb_cuda_message(TB_CUDA_ERROR_UNAVAILABLE, "Bend CUDA requires compute capability 7.0 or newer"); goto done;
  }
  (void)snprintf(architecture, sizeof(architecture), "--gpu-architecture=sm_%d%d", tb_cuda.info.major, tb_cuda.info.minor);
  options[0] = architecture; options[1] = "--fmad=false"; options[2] = "--std=c++17";
  if (!tb_cuda_compile_status(tb_cuda.nvrtcCreateProgram(&program, source, "bend-device.cu", 0, NULL, NULL), "nvrtcCreateProgram")) goto done;
  if (!tb_cuda_compile_status(tb_cuda.nvrtcCompileProgram(program, 3, options), "nvrtcCompileProgram")) {
    size_t log_size = 0;
    if (tb_cuda.nvrtcGetProgramLogSize(program, &log_size) == 0 && log_size > 1 && log_size <= BEND_MAX_HOST_BUFFER) {
      char *log = (char *)malloc(log_size);
      if (log != NULL) { if (tb_cuda.nvrtcGetProgramLog(program, log) == 0) (void)snprintf(tb_cuda.error, sizeof(tb_cuda.error), "NVRTC compilation failed: %s", log); free(log); }
    }
    goto done;
  }
  if (!tb_cuda_compile_status(tb_cuda.nvrtcGetCUBINSize(program, &size), "nvrtcGetCUBINSize")) goto done;
  if (size == 0 || size > BEND_MAX_HOST_BUFFER) { (void)tb_cuda_message(TB_CUDA_ERROR_COMPILE, "CUDA compiled image exceeds host buffer limit"); goto done; }
  image = (char *)malloc(size); tb_cuda.source = (char *)malloc(source_size + 1);
  if (image == NULL || tb_cuda.source == NULL) { (void)tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "CUDA compilation allocation failed"); goto done; }
  memcpy(tb_cuda.source, source, source_size + 1);
  if (!tb_cuda_compile_status(tb_cuda.nvrtcGetCUBIN(program, image), "nvrtcGetCUBIN")) goto done;
  if (!tb_cuda_status(tb_cuda.cuCtxCreate(&tb_cuda.context, 0, tb_cuda.info.device), "cuCtxCreate", TB_CUDA_ERROR_RUNTIME)) goto done;
  current = true;
  if (!tb_cuda_status(tb_cuda.cuModuleLoadData(&tb_cuda.module, image), "cuModuleLoadData", TB_CUDA_ERROR_RUNTIME)
      || !tb_cuda_status(tb_cuda.cuStreamCreate(&tb_cuda.stream, 1), "cuStreamCreate", TB_CUDA_ERROR_RUNTIME)) goto done;
  success = true;
done:
  if (program != NULL) {
    int destroyed = tb_cuda.nvrtcDestroyProgram(&program);
    if (success && !tb_cuda_compile_status(destroyed, "nvrtcDestroyProgram")) success = false;
  }
  free(image);
  if (current && !tb_cuda_pop(success)) success = false;
  if (!success) tb_cuda_shutdown();
  else { tb_cuda.initialized = true; tb_cuda_counter(&tb_cuda.info.compilations, 1); }
  return success;
}
static inline bool tb_cuda_synchronize(void) {
  if (!tb_cuda_push()) return false;
  return tb_cuda_pop(tb_cuda_status(tb_cuda.cuStreamSynchronize(tb_cuda.stream), "cuStreamSynchronize", TB_CUDA_ERROR_RUNTIME));
}
static inline bool tb_cuda_reserve(unsigned int slot, size_t bytes) {
  uint64_t total = 0, replacement = 0;
  bool success;
  if (slot >= TB_CUDA_BUFFERS) return tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "CUDA buffer slot is invalid");
  if (!tb_cuda.initialized) return tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "CUDA session is not initialized");
  if (bytes <= tb_cuda.buffers[slot].capacity) return true;
  for (unsigned int i = 0; i < TB_CUDA_BUFFERS; ++i) total += tb_cuda.buffers[i].capacity;
  /* Account for replacement overlap too: retain the old buffer until its
   * replacement is successfully allocated, without exceeding the hard cap. */
  if (bytes > BEND_MAX_GPU_ALLOC || total > BEND_MAX_GPU_ALLOC - bytes)
    return tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "CUDA device allocation budget exhausted");
  if (!tb_cuda_push()) return false;
  success = tb_cuda_status(tb_cuda.cuStreamSynchronize(tb_cuda.stream), "cuStreamSynchronize before allocation", TB_CUDA_ERROR_RUNTIME)
    && tb_cuda_status(tb_cuda.cuMemAlloc(&replacement, bytes), "cuMemAlloc", TB_CUDA_ERROR_RUNTIME);
  if (success && tb_cuda.buffers[slot].address != 0)
    success = tb_cuda_status(tb_cuda.cuMemFree(tb_cuda.buffers[slot].address), "cuMemFree replaced allocation", TB_CUDA_ERROR_RUNTIME);
  if (success) { tb_cuda.buffers[slot].address = replacement; tb_cuda.buffers[slot].capacity = bytes; tb_cuda_counter(&tb_cuda.info.allocations, 1); }
  else if (replacement != 0) (void)tb_cuda.cuMemFree(replacement);
  return tb_cuda_pop(success);
}
static inline uint64_t tb_cuda_address(unsigned int slot) {
  if (!tb_cuda.initialized || slot >= TB_CUDA_BUFFERS) { (void)tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "CUDA buffer slot is unavailable"); return 0; }
  return tb_cuda.buffers[slot].address;
}
static bool tb_cuda_span(unsigned int slot, size_t offset, size_t bytes, const void *host) {
  if (slot >= TB_CUDA_BUFFERS || offset > tb_cuda.buffers[slot].capacity || bytes > tb_cuda.buffers[slot].capacity - offset || (bytes != 0 && host == NULL))
    return tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "CUDA transfer span is invalid");
  return true;
}
static inline bool tb_cuda_upload(unsigned int slot, size_t offset, const void *host, size_t bytes) {
  bool success;
  if (!tb_cuda_span(slot, offset, bytes, host) || !tb_cuda_push()) return false;
  /* Blocking copies accept ordinary pageable host storage. Drain our explicit
   * stream before using the driver's default-stream transfer, then drain again
   * before releasing that storage or queuing more nonblocking-stream work. */
  success = tb_cuda_status(tb_cuda.cuStreamSynchronize(tb_cuda.stream), "cuStreamSynchronize before upload", TB_CUDA_ERROR_RUNTIME);
  if (success && bytes != 0) success = tb_cuda_status(tb_cuda.cuMemcpyHtoD(tb_cuda.buffers[slot].address + offset, host, bytes), "cuMemcpyHtoD", TB_CUDA_ERROR_RUNTIME);
  if (success) success = tb_cuda_status(tb_cuda.cuCtxSynchronize(), "cuCtxSynchronize after upload", TB_CUDA_ERROR_RUNTIME);
  if (success) tb_cuda_counter(&tb_cuda.info.upload_bytes, bytes);
  return tb_cuda_pop(success);
}
static inline bool tb_cuda_download(unsigned int slot, size_t offset, void *host, size_t bytes) {
  bool success;
  if (!tb_cuda_span(slot, offset, bytes, host) || !tb_cuda_push()) return false;
  success = tb_cuda_status(tb_cuda.cuStreamSynchronize(tb_cuda.stream), "cuStreamSynchronize before download", TB_CUDA_ERROR_RUNTIME);
  if (success && bytes != 0) success = tb_cuda_status(tb_cuda.cuMemcpyDtoH(host, tb_cuda.buffers[slot].address + offset, bytes), "cuMemcpyDtoH", TB_CUDA_ERROR_RUNTIME);
  if (success) tb_cuda_counter(&tb_cuda.info.download_bytes, bytes);
  return tb_cuda_pop(success);
}
static inline bool tb_cuda_launch(const char *name, unsigned int grid, unsigned int block, void **arguments) {
  void *function = NULL;
  bool success;
  if (name == NULL || grid == 0 || block == 0 || grid > (unsigned int)tb_cuda.info.max_grid || block > (unsigned int)tb_cuda.info.max_block)
    return tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "CUDA kernel launch dimensions are invalid");
  if (!tb_cuda_push()) return false;
  success = tb_cuda_status(tb_cuda.cuModuleGetFunction(&function, tb_cuda.module, name), "cuModuleGetFunction", TB_CUDA_ERROR_RUNTIME)
    && tb_cuda_status(tb_cuda.cuLaunchKernel(function, grid, 1, 1, block, 1, 1, 0, tb_cuda.stream, arguments, NULL), "cuLaunchKernel", TB_CUDA_ERROR_RUNTIME);
  if (success) tb_cuda_counter(&tb_cuda.info.launches, 1);
  return tb_cuda_pop(success);
}
