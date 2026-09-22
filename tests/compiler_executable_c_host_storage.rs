// SPDX-License-Identifier: MPL-2.0
#[path = "support/executable_c_compiler.rs"]
mod executable_c_compiler;

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use teamy_bend::compiler::compile_executable_c;
use teamy_bend::kernel::check_executable;
use teamy_bend::syntax::load_executable;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let directory = std::env::temp_dir().join(format!(
            "teamy-bend-host-storage-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).unwrap();
        Self(directory)
    }

    fn run(&self, probe: &str, inject_failures: bool) -> Output {
        self.run_program(
            "import Base\ndef main() -> U32: 42\n",
            probe,
            inject_failures,
        )
    }

    fn run_program(&self, program: &str, probe: &str, inject_failures: bool) -> Output {
        let path = self.0.join("main.bend");
        fs::write(&path, program).unwrap();
        let checked = check_executable(&load_executable(path).unwrap()).unwrap();
        let mut generated = compile_executable_c(&checked).unwrap();
        if inject_failures {
            let reserve =
                "static bool tb_host_storage_reserve(TBHostStorage *storage, size_t bytes) {";
            let commit =
                "static bool tb_host_storage_commit(TBHostStorage *storage, size_t bytes) {";
            assert_eq!(generated.matches(reserve).count(), 1);
            assert_eq!(generated.matches(commit).count(), 1);
            generated = generated.replace(
                reserve,
                &format!("{FAILURE_STATE}\n{reserve}\n{RESERVE_FAILURE}"),
            );
            generated = generated.replace(commit, &format!("{commit}\n{COMMIT_FAILURE}"));
        }
        // Keep the real runtime and generated program; only replace its outer CLI.
        let source = format!("#define TB_NO_MAIN 1\n{generated}\n{COMMON}\n{probe}");
        let executable = executable_c_compiler::compile(
            &self.0,
            &source,
            &["BEND_MAX_ALLOC=268435456", "BEND_CPU_WORKERS=4"],
        );
        executable_c_compiler::bounded(
            Command::new(executable)
                .current_dir(&self.0)
                .env("BEND_GPU", "off"),
            Duration::from_secs(20),
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::env::var_os("TEAMY_BEND_KEEP_HOST_STORAGE_TESTS").is_some() {
            eprintln!("retained host storage test: {}", self.0.display());
            return;
        }
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn expect(output: &Output, stdout: &str, stderr: &str) {
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, stdout.as_bytes(), "{output:?}");
    assert_eq!(output.stderr, stderr.as_bytes(), "{output:?}");
}

const COMMON: &str = r#"
#define STORAGE_REQUIRE(condition) do { \
  if (!(condition)) { \
    (void)fprintf(stderr, "host storage assertion at line %d: %s\n", __LINE__, #condition); \
    exit(90); \
  } \
} while (0)
static inline void storage_test_binary_output(void) {
#ifdef _WIN32
  (void)_setmode(_fileno(stdout), _O_BINARY);
  (void)_setmode(_fileno(stderr), _O_BINARY);
#endif
}
static inline bool storage_test_empty(const TBHostStorage *storage) {
  return storage->address == NULL && storage->capacity == 0 && storage->reserved == 0
    && storage->committed == 0 && storage->page_size == 0;
}
static inline bool storage_test_cleanup(void) {
  return storage_test_empty(&tb_corpus_storage) && storage_test_empty(&tb_metadata_storage)
    && tb_memory == NULL && tb_heap_meta == NULL && tb_host_current == NULL
    && tb_failure_guard == NULL && !tb_vm_held && !tb_cpu_active && tb_cpu_live_workers == 0
    && tb_live_words == 0 && tb_live_blocks == 0 && tb_tasks == 0
    && tb_continuations == 0 && tb_frames == 0 && tb_depth == 0;
}
static inline void storage_test_mapping(const TBHostStorage *storage, size_t offset, bool committed) {
#ifdef _WIN32
  MEMORY_BASIC_INFORMATION info;
  STORAGE_REQUIRE(VirtualQuery((const unsigned char *)storage->address + offset, &info, sizeof(info)) == sizeof(info));
  STORAGE_REQUIRE(info.AllocationBase == storage->address);
  STORAGE_REQUIRE(info.State == (DWORD)(committed ? MEM_COMMIT : MEM_RESERVE));
  if (committed) STORAGE_REQUIRE(info.Protect == PAGE_READWRITE);
  else STORAGE_REQUIRE(info.Protect == 0 || info.Protect == PAGE_NOACCESS);
#else
  (void)storage; (void)offset; (void)committed;
#endif
}
static inline void storage_test_released(void *address) {
#ifdef _WIN32
  MEMORY_BASIC_INFORMATION info;
  if (address == NULL) return;
  STORAGE_REQUIRE(VirtualQuery(address, &info, sizeof(info)) == sizeof(info));
  STORAGE_REQUIRE(info.State == MEM_FREE);
#else
  (void)address;
#endif
}
"#;

#[test]
fn large_virtual_reservation_commits_only_stable_zeroed_page_prefixes() {
    let output = Fixture::new().run(
        r"
int main(void) {
  TBHostStorage storage = {0};
  const size_t capacity = (size_t)268435456 + 17u;
  storage_test_binary_output();
  STORAGE_REQUIRE(!tb_host_storage_reserve(&storage, 0));
  STORAGE_REQUIRE(!tb_host_storage_reserve(&storage, SIZE_MAX));
  STORAGE_REQUIRE(!tb_host_storage_commit(&storage, 0));
  STORAGE_REQUIRE(storage_test_empty(&storage));
  STORAGE_REQUIRE(tb_host_storage_reserve(&storage, capacity));
  void *address = storage.address;
  size_t page = storage.page_size;
  STORAGE_REQUIRE(page > 17 && storage.capacity == capacity && storage.committed == 0);
  STORAGE_REQUIRE(storage.reserved >= capacity && storage.reserved - capacity < page);
  storage_test_mapping(&storage, 0, false);
  storage_test_mapping(&storage, capacity - 1, false);
  STORAGE_REQUIRE(tb_host_storage_commit(&storage, 0) && storage.committed == 0);
  STORAGE_REQUIRE(tb_host_storage_commit(&storage, 1) && storage.committed == page);
  unsigned char *bytes = (unsigned char *)address;
  for (size_t i = 0; i < page; ++i) STORAGE_REQUIRE(bytes[i] == 0);
  bytes[0] = 0x3c; bytes[page - 1] = 0xa5;
  STORAGE_REQUIRE(tb_host_storage_commit(&storage, page + 1));
  STORAGE_REQUIRE(storage.address == address && storage.committed == 2 * page);
  STORAGE_REQUIRE(bytes[0] == 0x3c && bytes[page - 1] == 0xa5);
  for (size_t i = page; i < 2 * page; ++i) STORAGE_REQUIRE(bytes[i] == 0);
  storage_test_mapping(&storage, page, true);
  storage_test_mapping(&storage, 2 * page, false);
  storage_test_mapping(&storage, capacity - 1, false);
  STORAGE_REQUIRE(tb_host_storage_commit(&storage, 1) && storage.committed == 2 * page);
  STORAGE_REQUIRE(!tb_host_storage_commit(&storage, capacity + 1));
  STORAGE_REQUIRE(!tb_host_storage_reserve(&storage, capacity));
  STORAGE_REQUIRE(storage.address == address && storage.committed == 2 * page);
  tb_host_storage_release(&storage);
  STORAGE_REQUIRE(storage_test_empty(&storage));
  storage_test_released(address);
  tb_host_storage_release(&storage);
  STORAGE_REQUIRE(storage_test_empty(&storage));

  /* The logical limit also applies inside the already committed final page. */
  STORAGE_REQUIRE(tb_host_storage_reserve(&storage, page + 17));
  address = storage.address;
  STORAGE_REQUIRE(tb_host_storage_commit(&storage, storage.capacity));
  STORAGE_REQUIRE(storage.committed == 2 * page);
  STORAGE_REQUIRE(!tb_host_storage_commit(&storage, storage.capacity + 1));
  STORAGE_REQUIRE(storage.address == address && storage.committed == 2 * page);
  tb_host_storage_release(&storage);
  STORAGE_REQUIRE(storage_test_empty(&storage));
  storage_test_released(address);
  STORAGE_REQUIRE(tb_program_main() == 0);
  STORAGE_REQUIRE(storage_test_cleanup());
  return 0;
}
",
        false,
    );
    expect(&output, "42\n", "");
}

const GROWTH_CALLBACKS: &str = r"
static void *storage_test_corpus_address, *storage_test_metadata_address;
static void storage_test_initialize(Env e) {
  STORAGE_REQUIRE(e.mem == tb_memory && e.mem == tb_corpus_storage.address);
  STORAGE_REQUIRE(tb_heap_meta == tb_metadata_storage.address);
  STORAGE_REQUIRE(tb_corpus_storage.capacity == (size_t)BEND_MAX_ALLOC);
  STORAGE_REQUIRE(tb_metadata_storage.capacity == (size_t)BEND_MAX_ALLOC);
  size_t page = tb_corpus_storage.page_size;
  size_t initial = ((size_t)HEAP_OFF * sizeof(Term) + page - 1) / page * page;
  STORAGE_REQUIRE(tb_corpus_storage.committed == initial && tb_metadata_storage.committed == initial);
  STORAGE_REQUIRE(initial < tb_corpus_storage.capacity / 1024);
  storage_test_mapping(&tb_corpus_storage, initial - 1, true);
  storage_test_mapping(&tb_corpus_storage, initial, false);
  storage_test_mapping(&tb_metadata_storage, initial, false);
  storage_test_corpus_address = tb_corpus_storage.address;
  storage_test_metadata_address = tb_metadata_storage.address;
}
static Term storage_test_grow(Env e) {
  Loc first = heap_alloc(e, 0);
  e.mem[first] = UINT64_C(0x123456789abcdef0);
  u64 metadata = tb_heap_meta[first];
  Cls cls = cls_fit((u32)(3 * tb_corpus_storage.page_size / sizeof(Term)));
  Loc grown = heap_alloc(e, cls);
  STORAGE_REQUIRE(e.mem == storage_test_corpus_address && tb_heap_meta == storage_test_metadata_address);
  STORAGE_REQUIRE(e.mem[first] == UINT64_C(0x123456789abcdef0) && tb_heap_meta[first] == metadata);
  for (u64 i = 0; i < (UINT64_C(1) << cls); ++i) STORAGE_REQUIRE(e.mem[grown + i] == 0);
  STORAGE_REQUIRE(tb_corpus_storage.committed >= (size_t)tb_bump * sizeof(Term));
  STORAGE_REQUIRE(tb_metadata_storage.committed == tb_corpus_storage.committed);
  STORAGE_REQUIRE(tb_corpus_storage.committed < tb_corpus_storage.capacity / 1024);
  storage_test_mapping(&tb_corpus_storage, tb_corpus_storage.committed, false);
  storage_test_mapping(&tb_metadata_storage, tb_metadata_storage.committed, false);
  heap_free(e, cls, grown);
  heap_free(e, 0, first);
  return 0;
}
";

#[test]
fn runtime_grows_both_arenas_without_moving_live_values_or_eagerly_backing_capacity() {
    let probe = format!(
        r"{GROWTH_CALLBACKS}
int main(void) {{
  storage_test_binary_output();
  STORAGE_REQUIRE(tb_run(storage_test_grow, 0, NULL, storage_test_initialize) == 0);
  STORAGE_REQUIRE(storage_test_cleanup());
  storage_test_released(storage_test_corpus_address);
  storage_test_released(storage_test_metadata_address);
  STORAGE_REQUIRE(tb_program_main() == 0);
  STORAGE_REQUIRE(storage_test_cleanup());
  return 0;
}}
"
    );
    expect(&Fixture::new().run(&probe, false), "42\n", "");
}

const FAILURE_STATE: &str = r"
static unsigned int storage_failure_mode, storage_failure_hits;
static unsigned int storage_initializer_entries, storage_program_entries;
static bool storage_failure_locked, storage_failure_worker;
static Loc storage_failure_bump;
static void *storage_failure_corpus, *storage_failure_metadata;
static size_t storage_failure_corpus_committed, storage_failure_metadata_committed;
static void storage_failure_record(void) {
  ++storage_failure_hits;
  storage_failure_locked = tb_vm_held; storage_failure_bump = tb_bump;
  storage_failure_worker = tb_worker_failure_capture;
  storage_failure_corpus = tb_corpus_storage.address;
  storage_failure_metadata = tb_metadata_storage.address;
  storage_failure_corpus_committed = tb_corpus_storage.committed;
  storage_failure_metadata_committed = tb_metadata_storage.committed;
}
";

const RESERVE_FAILURE: &str = r"
  if (storage_failure_mode == 1 && storage == &tb_metadata_storage) {
    storage_failure_record(); return false;
  }
";

const COMMIT_FAILURE: &str = r"
  if ((storage_failure_mode == 2 || storage_failure_mode == 3
      || (storage_failure_mode == 4 && tb_worker_failure_capture && storage_failure_hits == 0))
      && storage == &tb_metadata_storage && bytes > storage->committed) {
    storage_failure_record(); return false;
  }
";

#[test]
fn second_reservation_or_initial_commit_failure_releases_partial_ownership_before_recovery() {
    let output = Fixture::new().run(
        r"
static void storage_count_initialize(Env e) { (void)e; ++storage_initializer_entries; }
static Term storage_count_entry(Env e) { (void)e; ++storage_program_entries; return 0; }
int main(void) {
  storage_test_binary_output();
  for (unsigned int mode = 1; mode <= 2; ++mode) {
    storage_failure_mode = mode; storage_failure_hits = 0;
    STORAGE_REQUIRE(tb_run(storage_count_entry, 0, NULL, storage_count_initialize) == 1);
    STORAGE_REQUIRE(storage_failure_hits == 1 && storage_initializer_entries == 0 && storage_program_entries == 0);
    STORAGE_REQUIRE(!storage_failure_locked && storage_failure_bump == HEAP_OFF && tb_bump == HEAP_OFF);
    STORAGE_REQUIRE(storage_failure_corpus != NULL && storage_failure_metadata_committed == 0);
    if (mode == 1) STORAGE_REQUIRE(storage_failure_metadata == NULL && storage_failure_corpus_committed == 0);
    else STORAGE_REQUIRE(storage_failure_metadata != NULL && storage_failure_corpus_committed > 0);
    STORAGE_REQUIRE(storage_test_cleanup());
    storage_test_released(storage_failure_corpus);
    storage_test_released(storage_failure_metadata);
    storage_failure_mode = 0;
    STORAGE_REQUIRE(tb_program_main() == 0);
    STORAGE_REQUIRE(storage_test_cleanup());
  }
  return 0;
}
",
        true,
    );
    expect(
        &output,
        "42\n42\n",
        "teamy-bend executable C: VM initialization failed\nteamy-bend executable C: host corpus commitment failed\n",
    );
}

#[test]
fn failed_locked_bump_commit_does_not_publish_allocation_and_the_same_process_recovers() {
    let probe = format!(
        r"{GROWTH_CALLBACKS}
static Term storage_fail_growing(Env e) {{
  ++storage_program_entries;
  storage_failure_mode = 3;
  Cls cls = cls_fit((u32)(3 * tb_corpus_storage.page_size / sizeof(Term)));
  (void)heap_alloc(e, cls);
  STORAGE_REQUIRE(false);
  return 0;
}}
int main(void) {{
  storage_test_binary_output();
  STORAGE_REQUIRE(tb_run(storage_fail_growing, 0, NULL, NULL) == 1);
  STORAGE_REQUIRE(storage_failure_hits == 1 && storage_program_entries == 1 && storage_initializer_entries == 0);
  STORAGE_REQUIRE(storage_failure_locked && storage_failure_bump == HEAP_OFF && tb_bump == HEAP_OFF);
  STORAGE_REQUIRE(storage_failure_corpus_committed > storage_failure_metadata_committed);
  STORAGE_REQUIRE(storage_failure_metadata_committed >= HEAP_OFF * sizeof(Term));
  STORAGE_REQUIRE(storage_test_cleanup());
  storage_test_released(storage_failure_corpus);
  storage_test_released(storage_failure_metadata);
  storage_failure_mode = 0;
  /* Reacquiring the allocator lock and crossing the same page boundary must work. */
  STORAGE_REQUIRE(tb_run(storage_test_grow, 0, NULL, storage_test_initialize) == 0);
  STORAGE_REQUIRE(storage_test_cleanup());
  STORAGE_REQUIRE(tb_program_main() == 0);
  STORAGE_REQUIRE(storage_test_cleanup());
  return 0;
}}
"
    );
    expect(
        &Fixture::new().run(&probe, true),
        "42\n",
        "teamy-bend executable C: host corpus commitment failed\n",
    );
}

#[test]
fn worker_commit_failure_releases_the_lock_and_joins_workers_before_runtime_reuse() {
    let output = Fixture::new().run_program(
        r"import Base
def make(depth: Nat) -> Array<U32>: Array.new(U32, depth, 7)
def read(pair: Array<U32> & U32) -> U32:
  (array, value) = pair
  value
def main() -> U32:
  first second = make(12n) make(12n)
  U32.add(read(Array.get(U32, first, 4095)), read(Array.get(U32, second, 4095)))
",
        r"
int main(void) {
  storage_test_binary_output();
  storage_failure_mode = 4;
  STORAGE_REQUIRE(tb_program_main() == 1);
  STORAGE_REQUIRE(storage_failure_hits == 1 && storage_failure_worker && storage_failure_locked);
  STORAGE_REQUIRE(storage_failure_corpus_committed > storage_failure_metadata_committed);
  STORAGE_REQUIRE(storage_failure_corpus != NULL && storage_failure_metadata != NULL);
  STORAGE_REQUIRE(storage_initializer_entries == 0 && storage_program_entries == 0);
  STORAGE_REQUIRE(storage_test_empty(&tb_corpus_storage) && storage_test_empty(&tb_metadata_storage));
  STORAGE_REQUIRE(tb_memory == NULL && tb_heap_meta == NULL && tb_host_current == NULL && tb_failure_guard == NULL);
  STORAGE_REQUIRE(!tb_vm_held && !tb_cpu_active && tb_cpu_live_workers == 0 && tb_cpu.created == 0);
  STORAGE_REQUIRE(tb_task_current == NULL);
  storage_test_released(storage_failure_corpus);
  storage_test_released(storage_failure_metadata);
  storage_failure_mode = 0;
  /* The same generated fork must now finish both worker results successfully. */
  STORAGE_REQUIRE(tb_program_main() == 0);
  STORAGE_REQUIRE(storage_failure_hits == 1 && storage_test_cleanup());
  return 0;
}
",
        true,
    );
    expect(
        &output,
        "14\n",
        "teamy-bend executable C: host corpus commitment failed\n",
    );
}
