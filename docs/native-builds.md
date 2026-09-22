# Native builds and generated-program controls

`compile --target native` builds a native binary through the Rust C generator and
an installed C11 toolchain. Add `--executable` for IO, foreign implementations and
GPU calls. Strict pure compilation remains a separate checked-data path.
This build/cache integration has Windows validation with MSVC and CUDA in plan
milestone 5.15.6.

```text
teamy-bend compile main.bend --executable --target native --output program.exe
program.exe --help
program.exe --threads 4 --gpu off
program.exe --gpu on
program.exe --gpu 64MB
program.exe --gpu-build
```

Use the platform's executable name and invocation convention. `--target c` and
`--target javascript` continue to write source without invoking a native compiler.
`TEAMY_BEND_CC`, then `CC`, can select a compiler executable. Arguments in those
variables are not shell-expanded. Otherwise the tool discovers a supported host
compiler; Windows can use its installed Visual Studio developer toolchain.

Compilation stages output beside its destination and publishes completed files
after successful compilation and GPU prebuild. Replacing an existing output
requires `--force`. Source inputs, imported Bend modules, selected foreign source,
their hard links, directories and symlinks are protected from output replacement.
The GPU sidecar receives the same checks. A failed build preserves the previous
binary. If cache publication succeeds but binary publication fails, the previous
binary can reject that cache through its source/compiler identity.

## Controls on an executable program

The generated executable handles these options before Bend main or foreign
registration initializers run:

- `--help` prints usage and exits.
- `--threads N` chooses a positive worker count, clamped to 128. Without it,
  the compiled default or detected CPU count applies.
- `--gpu off` disables GPU initialization. `--gpu on` requires a usable device
  when the binary contains an emitted device program; an unmarked binary still
  runs successfully. A CLI choice overrides `BEND_GPU`.
- `--gpu NMB` or `--gpu NGB` requests a corpus reservation using binary units.
  Fractions are accepted and rounded down to 16 KiB. Nonfinite, overflowing,
  too-small and unrepresentable spans fail before execution. Metadata and scratch
  storage are additional allocations. An explicitly compiled `BEND_MAX_GPU_ALLOC`
  remains a hard total allocation limit; an explicit span replaces the default
  corpus size and otherwise derives a checked total allowance. Startup reserves
  the device buffers before program effects run.
- `--gpu-build` compiles and writes GPU code, then exits without running main or
  reserving its corpus. Like upstream, an unavailable GPU produces a successful
  no-op. The option acts immediately when encountered, as does `--help`.

Without a CLI GPU choice, existing `BEND_GPU=auto|off|on` behavior remains.
The default corpus size is still bounded by the port's compiled configuration;
it does not yet automatically reserve all available device memory as upstream
CUDA does. Metal and Unix execution validation remain open.

## Persistent CUDA cache

The cache is `<actual executable path>.gpu`, independent of the current working
directory or the temporary generated C path. Moving the binary and sidecar
together preserves reuse on a compatible device/compiler installation.

A versioned header binds the cubin to the exact device source, compiler options,
target architecture and reported NVRTC/driver versions. Length and SHA-256 checks
reject malformed, truncated or changed payloads. A missing, stale, corrupt or
driver-rejected cache triggers compilation and one diagnostic. A warm cache loads
the module without an NVRTC compilation. CUDA execution still requires the
driver and NVRTC runtime. Files are written through a sibling
temporary file and replaced atomically; interrupted files do not replace a
previous complete cache.

The loaded module must expose all required Bend kernels before program effects
run. A driver-loadable module missing an entry fails at startup without CPU
replay. Freshly compiled modules receive the same check before cache publication.

Ordinary execution can continue with its compiled in-memory module if saving the
cache fails. Explicit `--gpu-build` reports that failure. Native compilation
invokes prebuild for an emitted GPU program before installing its output. A
successful build without a usable GPU has no new GPU artifact to report.

Compilation and runtime failures retain their errors and do not replay
consumed work on the CPU. Reduced compilation counts establish cache reuse, not a
measured Bend execution speedup.
