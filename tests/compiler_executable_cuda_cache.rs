// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture {
    directory: PathBuf,
    executable: PathBuf,
    cache: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        Self::with_definitions(&[])
    }

    fn with_definitions(extra: &[&str]) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-cuda-cache-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        let cache = directory.join("cache-é space.gpu");
        let mut source = include_str!("../src/compiler/executable_cuda.c").replace(
            "/* TB_CUDA_CACHE */",
            include_str!("../src/compiler/executable_cuda_cache.c"),
        );
        writeln!(
            source,
            "static const char *cache_path = {};\nstatic const char *directory_path = {};",
            c_string(&cache.to_string_lossy()),
            c_string(&directory.to_string_lossy())
        )
        .unwrap();
        source.push_str(PROBE);
        let mut definitions = vec!["BEND_MAX_ALLOC=1048576", "BEND_MAX_HOST_BUFFER=8388608"];
        definitions.extend_from_slice(extra);
        let executable = executable_c_compiler::compile(&directory, &source, &definitions);
        Self {
            directory,
            executable,
            cache,
        }
    }

    fn run(&self, operation: &str) -> Output {
        let output = executable_c_compiler::bounded(
            Command::new(&self.executable)
                .arg(operation)
                .current_dir(&self.directory),
            Duration::from_secs(45),
        );
        assert!(
            output.status.success(),
            "{operation}: {:?}\n{}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    fn no_temporary_files(&self) {
        assert!(fs::read_dir(&self.directory).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .contains(".tmp.")
        }));
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::env::var_os("TEAMY_BEND_KEEP_GPU_TESTS").is_some() {
            eprintln!("retained CUDA cache test: {}", self.directory.display());
            return;
        }
        let _removed = fs::remove_dir_all(&self.directory);
    }
}

fn c_string(value: &str) -> String {
    let mut result = String::from("\"");
    for byte in value.bytes() {
        write!(result, "\\{byte:03o}").unwrap();
    }
    result.push('"');
    result
}

#[test]
fn cache_hashes_match_sha256_vectors_and_identity_covers_compiler_inputs() {
    let fixture = Fixture::new();
    for operation in ["hash", "identity", "allocation"] {
        let output = fixture.run(operation);
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn a_configured_cuda_allocation_cap_cannot_be_raised_at_runtime() {
    let fixture = Fixture::with_definitions(&["BEND_MAX_GPU_ALLOC=128"]);
    fixture.run("allocation");
}

#[test]
fn cache_files_reject_corruption_and_publish_complete_atomic_replacements() {
    let fixture = Fixture::new();
    fixture.run("miss");
    fixture.run("write");
    fixture.run("read");
    let valid = fs::read(&fixture.cache).unwrap();
    for offset in [0, 8, 12, 16, 24, 56, 88, valid.len() - 1] {
        let mut damaged = valid.clone();
        damaged[offset] ^= 1;
        fs::write(&fixture.cache, damaged).unwrap();
        fixture.run("miss");
    }
    for length in [0, 7, 111, valid.len() - 1] {
        fs::write(&fixture.cache, &valid[..length]).unwrap();
        fixture.run("miss");
    }
    let mut extended = valid;
    extended.push(0);
    fs::write(&fixture.cache, extended).unwrap();
    fixture.run("miss");
    let writers: Vec<_> = (0..4)
        .map(|_| {
            let executable = fixture.executable.clone();
            std::thread::spawn(move || {
                executable_c_compiler::bounded(
                    Command::new(executable).arg("write"),
                    Duration::from_secs(20),
                )
            })
        })
        .collect();
    for writer in writers {
        assert!(writer.join().unwrap().status.success());
    }
    fixture.run("read");
    fixture.run("write-failure");
    fixture.no_temporary_files();
}

#[derive(Debug)]
struct Receipt {
    success: u64,
    compilations: u64,
    hits: u64,
    misses: u64,
    rejections: u64,
    writes: u64,
    write_failures: u64,
    value: u64,
}

impl Receipt {
    fn read(output: &Output) -> Self {
        let values: Vec<u64> = String::from_utf8_lossy(&output.stdout)
            .split_whitespace()
            .map(|value| value.parse().unwrap())
            .collect();
        assert_eq!(values.len(), 8, "{output:?}");
        Self {
            success: values[0],
            compilations: values[1],
            hits: values[2],
            misses: values[3],
            rejections: values[4],
            writes: values[5],
            write_failures: values[6],
            value: values[7],
        }
    }
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn cuda_cache_warm_processes_skip_compilation_and_invalid_cubins_recompile() {
    let fixture = Fixture::new();
    let cold = fixture.run("gpu");
    let receipt = Receipt::read(&cold);
    assert_eq!(
        (receipt.success, receipt.compilations, receipt.misses),
        (1, 1, 1)
    );
    assert_eq!((receipt.writes, receipt.value), (1, 7));
    assert!(String::from_utf8_lossy(&cold.stderr).contains("is missing or stale"));
    let before = fs::read(&fixture.cache).unwrap();
    let warm = fixture.run("gpu");
    let receipt = Receipt::read(&warm);
    assert_eq!(
        (receipt.compilations, receipt.hits, receipt.value),
        (0, 1, 7)
    );
    assert!(warm.stderr.is_empty());
    assert_eq!(fs::read(&fixture.cache).unwrap(), before);
    fixture.run("poison");
    let rejected = Receipt::read(&fixture.run("gpu"));
    assert_eq!(
        (rejected.compilations, rejected.rejections, rejected.value),
        (1, 1, 7)
    );
    let changed = Receipt::read(&fixture.run("variant"));
    assert_eq!(
        (changed.compilations, changed.misses, changed.value),
        (1, 1, 8)
    );
    let warm_variant = Receipt::read(&fixture.run("variant"));
    assert_eq!(
        (
            warm_variant.compilations,
            warm_variant.hits,
            warm_variant.value
        ),
        (0, 1, 8)
    );
    fixture.no_temporary_files();
}

#[test]
#[ignore = "requires an installed CUDA driver, NVRTC, and compute capability 7.0 or newer"]
fn cuda_prebuild_requires_a_written_cache_but_ordinary_execution_tolerates_write_failure() {
    let fixture = Fixture::new();
    let prebuilt = Receipt::read(&fixture.run("build"));
    assert_eq!(
        (
            prebuilt.success,
            prebuilt.compilations,
            prebuilt.writes,
            prebuilt.value
        ),
        (1, 1, 1, 0)
    );
    let repeated = Receipt::read(&fixture.run("build"));
    assert_eq!(
        (repeated.compilations, repeated.hits, repeated.writes),
        (1, 0, 1)
    );
    let warmed = Receipt::read(&fixture.run("gpu"));
    assert_eq!((warmed.compilations, warmed.hits, warmed.value), (0, 1, 7));
    let optional = Receipt::read(&fixture.run("optional-write-failure"));
    assert_eq!(
        (
            optional.success,
            optional.compilations,
            optional.write_failures,
            optional.value
        ),
        (1, 1, 1, 7)
    );
    let required = Receipt::read(&fixture.run("required-write-failure"));
    assert_eq!(
        (
            required.success,
            required.compilations,
            required.write_failures
        ),
        (0, 1, 1)
    );
    fixture.no_temporary_files();
}

const PROBE: &str = r#"
static const char device_source[] = "extern \"C\" __global__ void answer(unsigned long long *value) { *value = 7; }";
static const char variant_source[] = "extern \"C\" __global__ void answer(unsigned long long *value) { *value = 8; }";
static const char payload[] = "portable cubin payload\0\1\2";
static void identity(unsigned char key[32]) {
  const char *options[] = {"--gpu-architecture=sm_89", "--fmad=false", "--std=c++17"};
  tb_cuda_cache_identity(device_source, strlen(device_source), options, 3, key);
}
static bool digest_matches(const char *source, const char *hex) {
  unsigned char digest[32];
  static const char digits[] = "0123456789abcdef";
  tb_cache_sha256(source, strlen(source), digest);
  for (unsigned int i = 0; i < 32; ++i)
    if (hex[2*i] != digits[digest[i] >> 4] || hex[2*i+1] != digits[digest[i] & 15]) return false;
  return true;
}
static int hash_test(void) {
  if (!digest_matches("", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")) return 11;
  if (!digest_matches("abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")) return 12;
  if (!digest_matches("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
      "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1")) return 13;
  return 0;
}
static int identity_test(void) {
  unsigned char original[32], changed[32];
  const char *options[] = {"--gpu-architecture=sm_89", "--fmad=false", "--std=c++17"};
  int *fields[] = {&tb_cuda.info.major, &tb_cuda.info.minor, &tb_cuda.info.nvrtc_major,
    &tb_cuda.info.nvrtc_minor, &tb_cuda.info.driver_version};
  identity(original);
  for (unsigned int i = 0; i < sizeof(fields)/sizeof(fields[0]); ++i) {
    ++*fields[i]; identity(changed); --*fields[i];
    if (memcmp(original, changed, 32) == 0) return 21;
  }
  tb_cuda_cache_identity(variant_source, strlen(variant_source), options, 3, changed);
  if (memcmp(original, changed, 32) == 0) return 22;
  options[1] = "--fmad=true";
  tb_cuda_cache_identity(device_source, strlen(device_source), options, 3, changed);
  if (memcmp(original, changed, 32) == 0) return 23;
  return 0;
}
static int allocation_test(void) {
  if (tb_cuda_set_allocation_limit(0)) return 24;
  tb_cuda.buffers[0].capacity = 64;
  if (tb_cuda_set_allocation_limit(63)) return 25;
  if (!tb_cuda_set_allocation_limit(128) || tb_cuda.allocation_limit != 128) return 26;
#if TB_CUDA_GPU_ALLOC_EXPLICIT
  if (tb_cuda_set_allocation_limit(BEND_MAX_GPU_ALLOC + 1)) return 28;
#else
  if (!tb_cuda_set_allocation_limit(BEND_MAX_GPU_ALLOC + 1)) return 29;
#endif
  tb_cuda_shutdown();
  return tb_cuda.allocation_limit == 0 ? 0 : 27;
}
static int gpu_test(const char *operation) {
  bool prebuild = strcmp(operation, "build") == 0 || strcmp(operation, "required-write-failure") == 0;
  bool required_failure = strcmp(operation, "required-write-failure") == 0;
  bool optional_failure = strcmp(operation, "optional-write-failure") == 0;
  const char *source = strcmp(operation, "variant") == 0 ? variant_source : device_source;
  if ((required_failure || optional_failure) && !tb_cuda_set_cache_path(directory_path)) return 30;
  bool success = prebuild ? tb_cuda_build_cache(source) : tb_cuda_initialize(source);
  uint64_t value = 0;
  if (required_failure) {
    if (success || tb_cuda_error_class() != TB_CUDA_ERROR_RUNTIME) return 31;
  } else if (!success || !tb_cuda.initialized) {
    fprintf(stderr, "GPU test initialization: %s\n", tb_cuda_error()); return 32;
  }
  if (success && !prebuild) {
    if (!tb_cuda_reserve(0, sizeof(value))) return 33;
    uint64_t address = tb_cuda_address(0); void *arguments[] = {&address};
    if (!tb_cuda_launch("answer", 1, 1, arguments) || !tb_cuda_synchronize()
        || !tb_cuda_download(0, 0, &value, sizeof(value))) return 34;
    if (strcmp(operation, "poison") == 0) {
      unsigned char key[32]; char architecture[64];
      snprintf(architecture, sizeof(architecture), "--gpu-architecture=sm_%d%d", tb_cuda.info.major, tb_cuda.info.minor);
      const char *options[] = {architecture, "--fmad=false", "--std=c++17"};
      tb_cuda_cache_identity(source, strlen(source), options, 3, key);
      if (!tb_cuda_cache_write(key, payload, sizeof(payload))) return 35;
    }
  }
  tb_cuda_shutdown();
  if (tb_cuda.cache_path != NULL || tb_cuda.context != NULL || tb_cuda.module != NULL
      || tb_cuda.stream != NULL || tb_cuda.source != NULL || tb_cuda.driver != NULL
      || tb_cuda.compiler != NULL || tb_cuda.initialized) return 36;
  const TBCudaInfo *info = tb_cuda_info();
  printf("%u %llu %llu %llu %llu %llu %llu %llu\n", (unsigned int)success,
    (unsigned long long)info->compilations, (unsigned long long)info->cache_hits,
    (unsigned long long)info->cache_misses, (unsigned long long)info->cache_rejections,
    (unsigned long long)info->cache_writes, (unsigned long long)info->cache_write_failures,
    (unsigned long long)value);
  return 0;
}
int main(int argc, char **argv) {
  if (argc != 2) return 1;
  if (strcmp(argv[1], "hash") == 0) return hash_test();
  if (strcmp(argv[1], "identity") == 0) return identity_test();
  if (strcmp(argv[1], "allocation") == 0) return allocation_test();
  if (!tb_cuda_set_cache_path(cache_path)) return 2;
  unsigned char key[32]; identity(key);
  if (strcmp(argv[1], "write") == 0) {
    bool written = tb_cuda_cache_write(key, payload, sizeof(payload)); tb_cuda_shutdown();
    return written ? 0 : 3;
  }
  if (strcmp(argv[1], "write-failure") == 0) {
    if (!tb_cuda_set_cache_path(directory_path)) return 4;
    bool written = tb_cuda_cache_write(key, payload, sizeof(payload)); tb_cuda_shutdown();
    return written ? 5 : 0;
  }
  if (strcmp(argv[1], "read") == 0 || strcmp(argv[1], "miss") == 0) {
    size_t size = 0; char *image = tb_cuda_cache_read(key, &size);
    bool wanted = strcmp(argv[1], "read") == 0;
    bool valid = image != NULL && size == sizeof(payload) && memcmp(image, payload, sizeof(payload)) == 0;
    bool result = wanted ? valid : image == NULL;
    free(image); tb_cuda_shutdown(); return result ? 0 : 6;
  }
  return gpu_test(argv[1]);
}
"#;
