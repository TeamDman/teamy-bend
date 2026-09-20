# Native socket provider

This crate builds the synchronous Node-API provider used by generated Bend
JavaScript. It uses maintained napi-rs bindings with Node-API 8 and dynamic host
symbol loading. No Node headers, import library, node-gyp, C++ build, npm package,
or worker pool is required. The tested host is Node 24 on Windows; Unix code is
provided but still requires its own platform qualification.

Build with `cargo build -p teamy-bend-sys`. Load the resulting shared library with
`process.dlopen`, or copy it to a `.node` extension for `require`. The compiler's
host loader creates and retains one `create_sys()` instance. `mac` is false on
every platform: this adapter translates the portable Linux-shaped ABI into the
host's native sockaddr, option, fcntl, readiness and error constants.

The provider performs real synchronous socket syscalls and readiness waits.
It returns actual descriptors, represented as safe JavaScript numbers or
unsigned BigInts when a SOCKET exceeds the number precision limit. Owned socket2
resources are bounded to 131072 and close with their provider. Descriptor
operations also accept foreign raw sockets and let the OS validate them, without
constructing Rust borrowed/owned socket values for untrusted descriptors. Calling
`close` on a foreign descriptor explicitly releases that socket; its foreign
owner must relinquish ownership. Owners of provider-created descriptors must use
the provider's close operation rather than independently closing and reusing
their numeric values.

`poll_descriptors([{fd, events}], timeoutMs)` preserves up to 131072 registrations,
their order and duplicates. READ=1 and WRITE=4; ERR=8, HUP=16 and NVAL=32 are
reported regardless of interests. It accepts timeouts from zero through 1000 ms.
The legacy `poll(view, count, timeoutMs)` accepts signed-i32 descriptors packed
into eight-byte records, with interest bits in the low16 and output bits in the
high16 of the second word. Its timeout accepts -1 for indefinite waiting and
finite i32 durations. Legacy records cannot represent wide Windows sockets;
foreign callers must range-check before packing, or use `poll_descriptors`.

All pointer arguments are ordinary TypedArray views, including Buffer and
subarrays. Their element widths, byte lengths and offsets are checked before
the syscall. SharedArrayBuffer-backed views are rejected. View pointers exist
only during a synchronous callback; there are no retained JavaScript buffers or
asynchronous jobs. IPv4 sockaddr arguments are sixteen bytes: family `[2,0]`,
network-order port, four address bytes, then padding. Optional accept/recvfrom
address outputs accept paired nulls; otherwise they need a sixteen-byte view
and a four-byte length view containing at least16.

Supported control operations are GETFL=3/SETFL=4 with NONBLOCK=0x800, and
SOL_SOCKET=1 with REUSEADDR=2/SO_ERROR=4. Other options or flags fail closed.
On Windows, GETFL validates the descriptor and reports the mode most recently
set through this provider. Winsock cannot query an arbitrary foreign socket's
initial nonblocking mode; foreign callers must establish it with SETFL first.
Unix GETFL reads the actual host flag. Would-block is normalized to11 and
connect-in-progress to115. Other errors retain native codes and OS text; Windows
network errors therefore usually use Winsock100xx codes. The surrounding JS
host composes Node's file provider for `read` and file error descriptions.

Windows `WSAEMSGSIZE` receives return the copied datagram prefix and sender,
including zero-length buffers, and discard the remainder. This matches POSIX
datagram truncation. Windows all-invalid poll failures become per-position NVAL.
An empty Windows poll uses a synchronous timer wait because WSAPoll rejects an
empty list. No process-wide signal handlers are installed; Unix sends suppress
SIGPIPE per call or socket.

Run the real host contract test after building:

```text
node native/node-sys/tests/provider.cjs <path-to-built-shared-library>
```

It covers loopback TCP/UDP, sender metadata, zero-length and truncated datagrams,
seventy duplicate registrations, EOF, BigInt descriptors, legacy polling,
typed-array view offsets, shared-buffer rejection, and argument bounds.
`cargo test -p teamy-bend-sys` also checks lossless descriptor conversion.
