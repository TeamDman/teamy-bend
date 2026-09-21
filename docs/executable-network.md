# Generated JavaScript networking

Generated executable JavaScript implements the eleven TCP, UDP and close effects
through a synchronous syscall provider. These execution contracts do not enter
strict proofs. The Windows implementation is validated under task 5.14 of the
[implementation plan](implementation-plan.md); Unix execution remains unverified.

## Build and package the provider

`cargo build --release` builds the CLI and the `teamy-bend-sys` Node-API library.
When the library is beside the CLI, `compile --executable` copies it beside the
generated program as `teamy-bend-sys.node`. An identical existing copy is reused.
A different existing provider produces an error before the program is replaced;
choose another output directory or update that provider explicitly. Compilation
never loads or runs the provider.

Generated programs can instead load a provider from `TEAMY_BEND_SYS_MODULE`.
Native libraries use Node's addon loader; JavaScript modules can supply the same
interface. An explicit module setting takes precedence over automatic sibling
discovery. A supplied `globalThis.BEND_SYS` takes precedence over both, including
its errno and error-message behavior. Missing or invalid providers fail when a
network operation first needs them. File-only and ordinary timer-only programs
continue to work without the addon.

`io_sys()` returns a stable managed view of the selected provider. The supplied
global object is unchanged, and its getters and methods run with that original
object as their receiver. The view records explicit raw closes so reused
descriptor values cannot remain in the invocation's cleanup table. Its object
identity therefore differs from `globalThis.BEND_SYS`; foreign code should use
the runtime accessor for socket operations and close aliases.

The addon is specific to the destination operating system and architecture.
When moving a generated program, include a provider built for that destination
or supply a compatible BEND_SYS. Installing only the CLI with `cargo install`
does not build and package the separate workspace library.

## Descriptor and scheduling interface

The addon exposes real OS socket descriptors, represented as lossless JavaScript
numbers or BigInts. Foreign functions can pass those values to bundled effects
and use descriptors returned by those effects. Typed-array views act as the
provider's pointer arguments, including their offsets and element widths; this
adapter does not expose arbitrary native memory addresses.

The portable syscall interface uses Linux-style constants and sockaddr bytes
on every host (`mac` is false). WouldBlock becomes errno 11 and a pending connect
becomes 115. Other Windows failures retain their WSA numbers and native messages.
Windows sockets remain Winsock resources and do not become CRT file descriptors.
The Node file adapter remains responsible for file reads. Only the fcntl modes
needed by the socket effects are supported. On Windows, GETFL reports the
provider's recorded nonblocking flag after validating the socket; it cannot
discover a mode previously changed outside the provider.

Descriptor waits share registration order with timers. Ready tasks run first;
when they drain, a synchronous poll resumes every ready registration or due
timer in order. Error, hangup and invalid-descriptor events also retry the saved
operation. Descriptor-only waits poll for at most 1,000 ms at a time, unlike
upstream's indefinite wait. A raced WouldBlock result parks again with its continuation. Initial
read hooks park before the first syscall; a write-only initial hook does not
park. Explicit `io_park_on` supports both read and write waits.

The preferred `poll_descriptors([{fd, events}], milliseconds)` interface returns
one revents mask per registration and preserves full Windows SOCKET width.
Portable event bits are read 1, write 4, error 8, hangup 16 and invalid 32.
Legacy `poll(ptr(buffer), count, milliseconds)` uses the upstream two-i32 layout:
descriptor, followed by low-16 event bits and high-16 returned bits. It requires
signed-i32 descriptors; the runtime refuses to narrow a larger value. Supplied
poll providers also handle timer-only waits with zero descriptors. The default
timer-only path uses Atomics.wait without loading the addon. Neither path pumps
Node's ordinary event-loop callbacks; Promise results remain unsupported.

## Ownership and bounds

The provider owns sockets it creates or accepts. Closing them releases that
ownership; provider collection also drops retained sockets. Operations on raw
foreign descriptors do not manufacture Rust socket ownership. Foreign code must
respect the lifetime of descriptors it borrows from the provider.

Bundled effects separately retain each acquired handle and its original provider
for cleanup when the IO invocation exits, halts or fails. A later BEND_SYS change
does not redirect that cleanup. Ordinary send/receive errors return the original
socket outside Result. Accept returns its listener outside Result. Closing through
the original BEND_SYS object or an alias taken from that object bypasses the
managed view's bookkeeping; use io_sys() for invocation-tracked descriptors.

Transfers are limited to 8 MiB and retained network buffers to 64 MiB. Socket
ownership and scheduler registrations have 131,072-entry bounds. Encoding
temporaries, JavaScript objects and OS socket buffers are additional; these are
not total-process memory limits. SharedArrayBuffer syscall views are rejected so
another thread cannot race native reads or writes. No view pointer is retained
after the synchronous callback.

TCP sends preserve their partial-write offset across waits. A zero-progress host
send fails instead of looping. Receive preserves short reads and EOF. UDP receive
returns the truncated prefix and sender, discards the datagram's remaining tail,
and consumes a datagram even for a zero-byte receive. UDP.poll distinguishes an
empty datagram from no datagram. Text follows upstream JavaScript TextEncoder
and TextDecoder behavior; native C's malformed-text policy is separate.

Validation matches 17 actual upstream checked programs and seven scheduler
traces. The program set includes nine original upstream networking fixtures,
five real peer/refusal/raw-descriptor groups and three scripted syscall groups.
The scripted groups establish partial-write/error transitions, not OS
backpressure. Direct provider tests also cover 70 ordered registrations, real
TCP/UDP, truncation, raw BigInt handles, typed-array views and argument bounds.
These comparisons use the same Windows provider for both generated programs;
they do not qualify upstream Bun/FFI or other operating systems.

Standalone [generated C](executable-c.md) uses its own native socket runtime.
Optimized C execution, broader native foreign coverage and GPU execution remain
separate work. Platform and oracle coverage is recorded in the
implementation plan; source implementations on other hosts are not execution
evidence for those hosts.
