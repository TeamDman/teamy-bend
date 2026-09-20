# Environment and file effects

Native `run` and generated executable JavaScript implement IO.get_env and
File.open/read/read_bytes/write/close through the executable contract checker.
File is a sealed opaque affine Type. Its declarations are absent from strict
proof Base; external results are execution assumptions, not proof certificates.

## File ownership and results

Open accepts exactly `r`, `w` and `a`. Write truncates or creates; append preserves
existing contents or creates. Embedded NUL in a path is rejected before mode
validation. Reads perform one host read at the current cursor, so short reads
and EOF remain observable. Byte reads produce List<&2, U32> octets. Text reads
decode each chunk independently. Writes finish successive short writes and
preserve already-written bytes if a later host call fails. Empty writes succeed
without checking the handle's access mode, following upstream. A zero-progress
write stops with a runtime error instead of repeating forever.

Read and write return `(file, result)`, keeping the same live handle outside
Result on both success and failure. Close consumes it and ignores host close
failure. Native handles are private monotonically assigned identities whose
host resources reside in the Machine or its pending worker job; source code
cannot inspect or manufacture them. The checker rejects copying an affine File.
Generated JavaScript retains upstream's raw descriptor representation for foreign
interoperability, including descriptors supplied by trusted foreign code.

Environment lookup distinguishes an existing empty value from a missing name.
Native lookup rejects embedded NUL with ENOENT. Names and values use native OS
byte representation on Unix and UTF-8 conversion on Windows.

## Native scheduling and cleanup

Native valid open, read and write requests park the current task and submit owned
host data to worker threads. Invalid open arguments, environment lookup and close
finish synchronously, matching the upstream C effect boundary. Workers do not
receive Bend values, arena pointers, continuations or output writers. Completed
host answers are packed and continuations resumed on the VM thread.

Ready tasks run before host completions are collected. When the ready queue is
empty, available host completions are enqueued before due timers. The collector
traces pending continuations directly. Waiting for host work wakes on completion
and checks cancellation at intervals no longer than 100 ms; idle wait time does
not consume evaluation steps. Child work remains live after main completes.

A shared pool grows lazily to at most 64 workers, as in upstream IO_HELP. Driver
exit discards pending VM continuations, removes its queued jobs and closes files
owned by the Machine. Already-running OS calls cannot be undone by dropping an
invocation: they may complete after Halt/cancellation. Their returned resources
are then dropped without resuming the VM. Teardown does not join a blocked host
call. Such calls retain a worker slot until they return; the process-wide limits
also cover these cancelled but still-running operations.

## Target differences

| Behavior | Native Rust | Generated JavaScript |
| --- | --- | --- |
| File work | Suspends through host workers, matching native C | Synchronous calls, matching upstream JS |
| Error numbers | Unix native errno; explicit Windows CRT-style mapping | Upstream JS/libuv numbers, including Windows differences |
| Error text | Native strerror/CRT strerror_s | Common file-error strings and Node/libuv fallback; supplied BEND_SYS remains authoritative |
| Malformed UTF-8 | Exact native C io_str reverse decoding into raw Char codes | Fresh TextDecoder replacement/BOM handling for each read |
| NUL environment names | ENOENT | Actual Windows Node lookup can resolve the name before NUL, as upstream JS does |
| NUL path | Native EILSEQ (Windows CRT 42, Linux 84, macOS 92) | Upstream hardcoded 84 except macOS 92 |

Native effect output uses the matching C io_utf8 algorithm for all U32 Char codes,
including surrogates and values above Unicode's scalar range. This preserves
bytes produced by native file decoding; numeric parsing keeps its own separate
validation. Windows accepts UTF-8 path/environment conversion and maps common
Win32 failures into the CRT errno domain, with EIO for unmapped failures. This
is a stated Windows adaptation, not a claim that Win32 is POSIX. Zero-byte reads
on a write-only handle follow the observed CRT success on Windows and EBADF on
Unix. Unix code is implemented but has not been runtime-validated on Unix here.
Default Node error text does not establish arbitrary libc locale equivalence.

## Resource bounds

Read requests first clamp to INT32_MAX, then fail closed if the requested buffer
exceeds 8 MiB. They are never silently shortened to the resource ceiling. Write
buffers and individual decoded text inputs also have an 8 MiB limit. Open paths
and modes each use the existing text limit. The pool retains at most 64 MiB of
reserved request/response byte buffers and 131,072 jobs across invocations;
reservations survive until replies are packed or discarded. Text decoding has
additional bounded temporary allocations, and worker stacks are separate from
this payload budget. These figures are not a total-process memory limit.

Files and pending continuations also obey the existing 131,072-entry arena/table
limits, evaluation budget and output bounds. Large valid host results can still
exhaust the bounded Bend arena while being packed. Resource failures remain
runtime errors and cannot masquerade as an ordinary successful IO result.

## Evidence

Focused tests cover exact contracts and origin validation, File affinity,
round trips, Unicode/NUL/CRLF, modes, cursor and EOF behavior, short reads/writes,
failed-operation handle retention, native raw characters, worker ordering,
cancellation, reservation reuse and forced garbage collection. Independent codec
comparison runs the production Rust implementation against verbatim upstream C.
Whole-program comparisons execute the actual upstream checker/compiler/JS runtime
in isolated copied fixture directories and state target differences explicitly.
The implementation plan records publication-specific counts and retained receipts.

Descriptor readiness, sockets, interactive window/audio effects, executable C
and GPU execution remain separate unfinished engine work.
