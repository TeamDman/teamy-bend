/* SPDX-License-Identifier: MPL-2.0
 * Bounded CUDA cubin sidecars. Cache identity includes the exact device source,
 * compiler options, target architecture and reported compiler/driver versions.
 * A complete file is published by replacing a sibling temporary file atomically.
 * Include at TB_CUDA_CACHE after the CUDA state and diagnostic helpers. */
#include <errno.h>
#ifdef _WIN32
#include <process.h>
#else
#include <fcntl.h>
#include <sys/stat.h>
#include <unistd.h>
#endif
#define TB_CUDA_CACHE_VERSION 1u
#define TB_CUDA_CACHE_HEADER 112u
#define TB_CUDA_CACHE_PATH_LIMIT 131072u

typedef struct {
  uint32_t words[8];
  uint64_t bytes;
  size_t used;
  unsigned char block[64];
} TBCacheHash;
static uint32_t tb_cache_rotr(uint32_t word, unsigned int count) {
  return (word >> count) | (word << (32u - count));
}
static void tb_cache_hash_block(TBCacheHash *hash, const unsigned char *block) {
  static const uint32_t constants[64] = {
    0x428a2f98u,0x71374491u,0xb5c0fbcfu,0xe9b5dba5u,0x3956c25bu,0x59f111f1u,0x923f82a4u,0xab1c5ed5u,
    0xd807aa98u,0x12835b01u,0x243185beu,0x550c7dc3u,0x72be5d74u,0x80deb1feu,0x9bdc06a7u,0xc19bf174u,
    0xe49b69c1u,0xefbe4786u,0x0fc19dc6u,0x240ca1ccu,0x2de92c6fu,0x4a7484aau,0x5cb0a9dcu,0x76f988dau,
    0x983e5152u,0xa831c66du,0xb00327c8u,0xbf597fc7u,0xc6e00bf3u,0xd5a79147u,0x06ca6351u,0x14292967u,
    0x27b70a85u,0x2e1b2138u,0x4d2c6dfcu,0x53380d13u,0x650a7354u,0x766a0abbu,0x81c2c92eu,0x92722c85u,
    0xa2bfe8a1u,0xa81a664bu,0xc24b8b70u,0xc76c51a3u,0xd192e819u,0xd6990624u,0xf40e3585u,0x106aa070u,
    0x19a4c116u,0x1e376c08u,0x2748774cu,0x34b0bcb5u,0x391c0cb3u,0x4ed8aa4au,0x5b9cca4fu,0x682e6ff3u,
    0x748f82eeu,0x78a5636fu,0x84c87814u,0x8cc70208u,0x90befffau,0xa4506cebu,0xbef9a3f7u,0xc67178f2u
  };
  uint32_t words[64], work[8];
  for (unsigned int i = 0; i < 16; ++i)
    words[i] = ((uint32_t)block[4*i] << 24) | ((uint32_t)block[4*i+1] << 16)
      | ((uint32_t)block[4*i+2] << 8) | block[4*i+3];
  for (unsigned int i = 16; i < 64; ++i) {
    uint32_t a = words[i-15], b = words[i-2];
    words[i] = words[i-16] + (tb_cache_rotr(a,7) ^ tb_cache_rotr(a,18) ^ (a >> 3))
      + words[i-7] + (tb_cache_rotr(b,17) ^ tb_cache_rotr(b,19) ^ (b >> 10));
  }
  memcpy(work, hash->words, sizeof(work));
  for (unsigned int i = 0; i < 64; ++i) {
    uint32_t first = work[7] + (tb_cache_rotr(work[4],6) ^ tb_cache_rotr(work[4],11)
      ^ tb_cache_rotr(work[4],25)) + ((work[4] & work[5]) ^ (~work[4] & work[6]))
      + constants[i] + words[i];
    uint32_t second = (tb_cache_rotr(work[0],2) ^ tb_cache_rotr(work[0],13)
      ^ tb_cache_rotr(work[0],22)) + ((work[0] & work[1]) ^ (work[0] & work[2]) ^ (work[1] & work[2]));
    for (unsigned int j = 7; j != 0; --j) work[j] = work[j-1];
    work[4] += first; work[0] = first + second;
  }
  for (unsigned int i = 0; i < 8; ++i) hash->words[i] += work[i];
}
static void tb_cache_hash_begin(TBCacheHash *hash) {
  static const uint32_t initial[8] = {
    0x6a09e667u,0xbb67ae85u,0x3c6ef372u,0xa54ff53au,
    0x510e527fu,0x9b05688cu,0x1f83d9abu,0x5be0cd19u
  };
  memcpy(hash->words, initial, sizeof(initial)); hash->bytes = 0; hash->used = 0;
}
static void tb_cache_hash_update(TBCacheHash *hash, const void *input, size_t bytes) {
  const unsigned char *data = (const unsigned char *)input;
  hash->bytes += bytes;
  while (bytes != 0) {
    size_t take = sizeof(hash->block) - hash->used;
    if (take > bytes) take = bytes;
    memcpy(hash->block + hash->used, data, take);
    hash->used += take; data += take; bytes -= take;
    if (hash->used == sizeof(hash->block)) {
      tb_cache_hash_block(hash, hash->block); hash->used = 0;
    }
  }
}
static void tb_cache_hash_end(TBCacheHash *hash, unsigned char output[32]) {
  uint64_t bits = hash->bytes * 8;
  hash->block[hash->used++] = 0x80;
  if (hash->used > 56) {
    memset(hash->block + hash->used, 0, 64 - hash->used);
    tb_cache_hash_block(hash, hash->block); hash->used = 0;
  }
  memset(hash->block + hash->used, 0, 56 - hash->used);
  for (unsigned int i = 0; i < 8; ++i) hash->block[63-i] = (unsigned char)(bits >> (8*i));
  tb_cache_hash_block(hash, hash->block);
  for (unsigned int i = 0; i < 32; ++i)
    output[i] = (unsigned char)(hash->words[i/4] >> (24 - 8*(i%4)));
}
static void tb_cache_sha256(const void *input, size_t bytes, unsigned char output[32]) {
  TBCacheHash hash; tb_cache_hash_begin(&hash);
  tb_cache_hash_update(&hash, input, bytes); tb_cache_hash_end(&hash, output);
}
static void tb_cache_put32(unsigned char *out, uint32_t value) {
  for (unsigned int i = 0; i < 4; ++i) out[i] = (unsigned char)(value >> (8*i));
}
static void tb_cache_put64(unsigned char *out, uint64_t value) {
  for (unsigned int i = 0; i < 8; ++i) out[i] = (unsigned char)(value >> (8*i));
}
static uint32_t tb_cache_get32(const unsigned char *in) {
  uint32_t value = 0;
  for (unsigned int i = 0; i < 4; ++i) value |= (uint32_t)in[i] << (8*i);
  return value;
}
static uint64_t tb_cache_get64(const unsigned char *in) {
  uint64_t value = 0;
  for (unsigned int i = 0; i < 8; ++i) value |= (uint64_t)in[i] << (8*i);
  return value;
}
static void tb_cache_hash_field(TBCacheHash *hash, const void *data, size_t bytes) {
  unsigned char size[8]; tb_cache_put64(size, bytes);
  tb_cache_hash_update(hash, size, sizeof(size)); tb_cache_hash_update(hash, data, bytes);
}
static void tb_cuda_cache_identity(const char *source, size_t bytes,
    const char *const *options, unsigned int count, unsigned char identity[32]) {
  static const char domain[] = "teamy-bend CUDA cubin cache";
  TBCacheHash hash;
  unsigned char fields[28];
  tb_cache_put32(fields, TB_CUDA_CACHE_VERSION);
  tb_cache_put32(fields + 4, (uint32_t)tb_cuda.info.major);
  tb_cache_put32(fields + 8, (uint32_t)tb_cuda.info.minor);
  tb_cache_put32(fields + 12, (uint32_t)tb_cuda.info.nvrtc_major);
  tb_cache_put32(fields + 16, (uint32_t)tb_cuda.info.nvrtc_minor);
  tb_cache_put32(fields + 20, (uint32_t)tb_cuda.info.driver_version);
  tb_cache_put32(fields + 24, count);
  tb_cache_hash_begin(&hash);
  tb_cache_hash_field(&hash, domain, sizeof(domain) - 1);
  tb_cache_hash_field(&hash, fields, sizeof(fields));
  tb_cache_hash_field(&hash, source, bytes);
  for (unsigned int i = 0; i < count; ++i) tb_cache_hash_field(&hash, options[i], strlen(options[i]));
  tb_cache_hash_end(&hash, identity);
}

#ifdef _WIN32
static wchar_t *tb_cache_wide(const char *path) {
  int count = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path, -1, NULL, 0);
  if (count <= 0 || count > (int)TB_CUDA_CACHE_PATH_LIMIT) return NULL;
  wchar_t *wide = (wchar_t *)malloc((size_t)count * sizeof(wchar_t));
  if (wide != NULL && MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path, -1, wide, count) == 0) {
    free(wide); return NULL;
  }
  return wide;
}
typedef HANDLE TBCacheFile;
#define TB_CACHE_INVALID INVALID_HANDLE_VALUE
static TBCacheFile tb_cache_open(const char *path, bool create) {
  wchar_t *wide = tb_cache_wide(path);
  if (wide == NULL) return TB_CACHE_INVALID;
  HANDLE file = CreateFileW(wide, create ? GENERIC_WRITE : GENERIC_READ,
    FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE, NULL,
    create ? CREATE_NEW : OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, NULL);
  free(wide); return file;
}
static bool tb_cache_file_size(TBCacheFile file, uint64_t *bytes) {
  LARGE_INTEGER size;
  if (GetFileType(file) != FILE_TYPE_DISK || !GetFileSizeEx(file, &size) || size.QuadPart < 0) return false;
  *bytes = (uint64_t)size.QuadPart; return true;
}
static bool tb_cache_transfer(TBCacheFile file, void *data, size_t bytes, bool write) {
  unsigned char *at = (unsigned char *)data;
  while (bytes != 0) {
    DWORD chunk = bytes > 0x7ffff000u ? 0x7ffff000u : (DWORD)bytes, done = 0;
    BOOL success = write ? WriteFile(file, at, chunk, &done, NULL) : ReadFile(file, at, chunk, &done, NULL);
    if (!success || done == 0 || done > chunk) return false;
    at += done; bytes -= done;
  }
  return true;
}
static bool tb_cache_close(TBCacheFile file) { return CloseHandle(file) != 0; }
static bool tb_cache_flush(TBCacheFile file) { return FlushFileBuffers(file) != 0; }
static void tb_cache_remove(const char *path) {
  wchar_t *wide = tb_cache_wide(path);
  if (wide != NULL) { (void)DeleteFileW(wide); free(wide); }
}
static bool tb_cache_replace(const char *from, const char *to) {
  wchar_t *source = tb_cache_wide(from), *destination = tb_cache_wide(to);
  bool success = source != NULL && destination != NULL
    && MoveFileExW(source, destination, MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH) != 0;
  free(source); free(destination); return success;
}
static unsigned long tb_cache_pid(void) { return (unsigned long)_getpid(); }
#else
typedef int TBCacheFile;
#define TB_CACHE_INVALID (-1)
static TBCacheFile tb_cache_open(const char *path, bool create) {
  int flags = create ? O_WRONLY | O_CREAT | O_EXCL : O_RDONLY | O_NONBLOCK;
#ifdef O_CLOEXEC
  flags |= O_CLOEXEC;
#endif
  return open(path, flags, 0666);
}
static bool tb_cache_file_size(TBCacheFile file, uint64_t *bytes) {
  struct stat info;
  if (fstat(file, &info) != 0 || !S_ISREG(info.st_mode) || info.st_size < 0) return false;
  *bytes = (uint64_t)info.st_size; return true;
}
static bool tb_cache_transfer(TBCacheFile file, void *data, size_t bytes, bool writing) {
  unsigned char *at = (unsigned char *)data;
  while (bytes != 0) {
    size_t chunk = bytes > 0x7ffff000u ? 0x7ffff000u : bytes;
    ssize_t done = writing ? write(file, at, chunk) : read(file, at, chunk);
    if (done < 0 && errno == EINTR) continue;
    if (done <= 0 || (size_t)done > chunk) return false;
    at += (size_t)done; bytes -= (size_t)done;
  }
  return true;
}
static bool tb_cache_close(TBCacheFile file) { return close(file) == 0; }
static bool tb_cache_flush(TBCacheFile file) { return fsync(file) == 0; }
static void tb_cache_remove(const char *path) { (void)unlink(path); }
static bool tb_cache_replace(const char *from, const char *to) { return rename(from, to) == 0; }
static unsigned long tb_cache_pid(void) { return (unsigned long)getpid(); }
#endif

static inline bool tb_cuda_set_cache_path(const char *path) {
  if (tb_cuda.initialized) return tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "cannot change an active CUDA cache path");
  char *copy = NULL;
  if (path != NULL) {
    size_t length = 0;
    while (length < TB_CUDA_CACHE_PATH_LIMIT && path[length] != 0) ++length;
    if (length == 0 || length == TB_CUDA_CACHE_PATH_LIMIT)
      return tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "invalid CUDA cache path");
#ifdef _WIN32
    wchar_t *wide = tb_cache_wide(path);
    if (wide == NULL) return tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "invalid UTF-8 CUDA cache path");
    free(wide);
#endif
    copy = (char *)malloc(length + 1);
    if (copy == NULL) return tb_cuda_message(TB_CUDA_ERROR_RUNTIME, "CUDA cache path allocation failed");
    memcpy(copy, path, length + 1);
  }
  free(tb_cuda.cache_path); tb_cuda.cache_path = copy; return true;
}
static const unsigned char tb_cache_magic[8] = {'T','B','C','U','B','I','N',0};
static char *tb_cuda_cache_read(const unsigned char identity[32], size_t *bytes) {
  unsigned char header[TB_CUDA_CACHE_HEADER], digest[32];
  uint64_t file_bytes = 0, payload = 0;
  char *image = NULL;
  TBCacheFile file = tb_cache_open(tb_cuda.cache_path, false);
  if (file == TB_CACHE_INVALID) return NULL;
  if (!tb_cache_file_size(file, &file_bytes) || file_bytes < sizeof(header)
      || !tb_cache_transfer(file, header, sizeof(header), false)) goto done;
  payload = tb_cache_get64(header + 16);
  if (memcmp(header, tb_cache_magic, sizeof(tb_cache_magic)) != 0
      || tb_cache_get32(header + 8) != TB_CUDA_CACHE_VERSION
      || tb_cache_get32(header + 12) != sizeof(header)
      || memcmp(header + 24, identity, 32) != 0
      || payload == 0 || payload > BEND_MAX_HOST_BUFFER || payload > SIZE_MAX
      || payload > UINT64_MAX - sizeof(header) || payload + sizeof(header) != file_bytes) goto done;
  for (unsigned int i = 88; i < sizeof(header); ++i) if (header[i] != 0) goto done;
  image = (char *)malloc((size_t)payload);
  if (image == NULL || !tb_cache_transfer(file, image, (size_t)payload, false)) goto invalid;
  tb_cache_sha256(image, (size_t)payload, digest);
  if (memcmp(header + 56, digest, sizeof(digest)) != 0) goto invalid;
  *bytes = (size_t)payload;
  goto done;
invalid:
  free(image); image = NULL;
done:
  if (!tb_cache_close(file)) { free(image); image = NULL; }
  return image;
}
static bool tb_cuda_cache_write(const unsigned char identity[32], const char *image, size_t bytes) {
  static uint64_t sequence;
  unsigned char header[TB_CUDA_CACHE_HEADER] = {0};
  TBCacheFile file = TB_CACHE_INVALID;
  size_t length = strlen(tb_cuda.cache_path);
  char *temporary = (char *)malloc(length + 80);
  if (temporary == NULL || bytes == 0 || bytes > BEND_MAX_HOST_BUFFER) { free(temporary); return false; }
  memcpy(temporary, tb_cuda.cache_path, length);
  for (unsigned int attempt = 0; attempt < 16; ++attempt) {
    if (sequence == UINT64_MAX) break;
    int written = snprintf(temporary + length, 80, ".tmp.%lu.%llu", tb_cache_pid(), (unsigned long long)++sequence);
    if (written <= 0 || written >= 80) break;
    file = tb_cache_open(temporary, true);
    if (file != TB_CACHE_INVALID) break;
  }
  if (file == TB_CACHE_INVALID) { free(temporary); return false; }
  memcpy(header, tb_cache_magic, sizeof(tb_cache_magic));
  tb_cache_put32(header + 8, TB_CUDA_CACHE_VERSION); tb_cache_put32(header + 12, sizeof(header));
  tb_cache_put64(header + 16, bytes); memcpy(header + 24, identity, 32);
  tb_cache_sha256(image, bytes, header + 56);
  bool success = tb_cache_transfer(file, header, sizeof(header), true)
    && tb_cache_transfer(file, (void *)image, bytes, true) && tb_cache_flush(file);
  if (!tb_cache_close(file)) success = false;
  if (success) success = tb_cache_replace(temporary, tb_cuda.cache_path);
  if (!success) tb_cache_remove(temporary);
  free(temporary); return success;
}
