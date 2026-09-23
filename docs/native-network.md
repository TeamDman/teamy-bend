# Native TCP, UDP and descriptor readiness

The native engine implements TCP.listen/accept/connect/send/recv,
UDP.bind/send_to/recv_from/poll and Socket.close/Listener.close. These use the
separate executable contract checker. Socket and Listener are loader-sealed,
nullary affine Types; host results cannot become strict proof evidence.
The native slice is validated on Windows. Generated JavaScript has a separate
[network provider and scheduler](executable-network.md); task 5.14 of the
[implementation plan](implementation-plan.md) tracks its validation.

## Ownership and results

Every native socket remains nonblocking. Private monotonically assigned handles
refer to resources owned by the VM or its parked request. Source code cannot
manufacture or copy handles. Accept returns its listener outside Result; send
and receive return their socket outside Result, including on ordinary host
failure. Close consumes the handle and discards close errors. Halt, cancellation
and runtime failure discard registrations and close both live and parked sockets.

Addresses are strict numeric IPv4. DNS names, embedded NUL, leading-zero octets,
extra characters and ports above 65535 fail. Listen binds the unspecified IPv4
address with backlog 16 and attempts address reuse. UDP.bind also binds the
unspecified address. Both accept port zero for an OS-selected port.
Windows integration tests set `TEAMY_BEND_TEST_LOOPBACK_NETWORK=1` only on
their disposable child processes; internal host-adapter tests bind loopback
under `cfg(test)`. This keeps test listeners local while leaving the default
production bind address unchanged.

TCP.send completes successive partial writes and retains its unsent suffix while
waiting for writability. Empty sends succeed without an OS call. Zero progress
fails closed instead of looping forever. TCP.recv performs one receive and
preserves short reads and empty EOF. Text uses the native C io_str/io_utf8 codec
described in [native files](native-files.md), including malformed bytes and raw
Char codes.

UDP sends one datagram. Receive truncates an oversized datagram, discards its
tail and returns sender address/port with the received text. Zero-length receives
still consume a datagram. UDP.poll returns immediately: Done(None) means no
datagram, while Done(Some(...)) can contain an empty datagram. TCP and UDP share
the upstream Socket type; mixed operations receive the actual host outcome.

## Scheduling and garbage collection

TCP.accept, TCP.recv and UDP.recv_from always park before their first syscall,
even when data is already ready. Connect and send first try immediately and park
only when the host requires it. An operation that races with readiness and finds
no data parks again at the end of the registration order.

When the ready queue is empty, the scheduler collects completed file jobs. It
then traverses ready socket
registrations and due timers in their common registration order. During sustained
runnable work it also collects file completions every 64 driver turns, as native
upstream does. Continuations stay on the VM thread and are direct collector roots;
parked requests retain rooted continuations, owned bytes and socket state
between readiness attempts.

Network waits use a descriptor poller, not the 64-thread file-worker pool. When
file jobs and network waits coexist, a lazily created connected loopback UDP pair
wakes the same poller on worker completion. The notifier is installed before
checking the completion queue, including for workers already running. Waiting
checks cancellation at intervals no longer than 100 ms and does not consume
evaluation steps.

## Platform policy and bounds

Windows sockets use Winsock ownership and native WSA error numbers/messages;
invalid address syntax uses EINVAL 22. Unix uses errno/strerror. These are explicit
target differences, not a claim that Winsock errors equal POSIX errors. The owned
socket2 wrapper closes resources; no CRT file descriptor or CloseHandle wrapper
owns a Windows socket. Unix source still requires runtime validation on Unix.
Windows address reuse also follows Winsock's live-port sharing rules rather than
POSIX exclusivity rules. Readiness uses WSAPoll on Windows and poll on Unix.
Three raw Windows probes observed refused loopback connections become ready
after 2.03–2.05 seconds with error/hangup/writability flags and WSAECONNREFUSED;
the original two-second test deadline was too short. No alternative poller was
needed on the validated host.

Receive lengths clamp to INT32_MAX and then fail closed above 8 MiB. Text sends
use the existing 8 MiB text ceiling. Retained network request buffers/reserved
receive sizes have a separate 64 MiB aggregate limit. Handles and parked requests
share a 131,072-entry bound. Decoding temporaries, socket kernel buffers and the
file-worker budget are additional; these limits are not a total-process memory
cap. The existing evaluation and arena limits also apply.

Generated JavaScript networking uses a separate synchronous syscall and readiness
provider preserving raw descriptors and callbacks. See its documentation for
Node-API packaging and the supplied BEND_SYS interface. Standalone
[executable C](executable-c.md) has its own packed runtime and socket adapters;
its bounded CPU runtime now reclaims values through reference counts and reuses
freed allocation blocks. Optimized parallel C execution, window/audio effects
and GPU execution remain unfinished.

## Validation boundary

The retained native milestone's quality gate passed 487 tests, including five compile-fail API boundary
examples, with two optional local profilers ignored. Strict library/test Clippy
passes. Nine public networking tests exercise checked source programs and
same-process cancellation; eight CLI cases also pass with the frozen release.
Ten host-adapter tests cover socket operations and platform adaptation.

Seven private VM tests cover mixed timer/socket ordering, captured continuations,
stale readiness and re-parking, file-worker wake notification, exit cleanup and
worker completion during sustained runnable work. Six force garbage collection
at every safe point. One parks 70 real backpressured 512 KiB sends while a
synthetic worker canary progresses, then verifies Halt cleanup. A separate source
fixture parks 70 receivers alongside a real file operation and timer. Another
private test verifies an initially backpressured 512 KiB send eventually delivers
the exact bytes through EOF and retains its captured continuation under forced
collection. It does not assert a particular sequence of partial-send retries.
The sustained-runnable fairness test uses normal GC and detects idle-only worker
collection; it does not measure the exact 64-turn cadence.

The production IPv4 parser matches 3,090 cases against verbatim upstream
io_sys_addr using actual Winsock InetPton. A separate adapter executes all eleven
upstream C effect bodies: six real loopback semantic groups, three explicitly
scripted send-transition groups and an adapter handle-cleanup group pass.
The production Rust host adapter directly matches those six loopback groups.
These adapters do not execute the full generated C runtime, collector or
scheduler, and are not whole-program Rust/C equivalence evidence.

The frozen candidate records 97 compiled-source fingerprints. Its strict
1,302-fixture audit preserves 362 accepted positives, 491 rejected positives and
449 rejected negatives, with no accepted negatives, crashes or new rejections.
Audit acceptance is not an overall compatibility percentage. Ignored receipts
are retained under target/audit-native-network,
target/native-network-release-comparison, target/native-network-address-oracle
and target/native-network-c-effects.
