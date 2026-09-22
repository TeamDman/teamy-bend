# References for the Bend2 GPU port

Recorded 2026-09-20 for guidance U11-U13 in the implementation plan. These
references now inform the generated CUDA implementation in milestone 5.15.5.
The first CUDA execution milestone is validated. The plan retains full GPU
parity, cache/build controls, platform coverage and performance requirements.

## User guidance and provenance

The user identifies Rik Arends's Makepad GPU strategies as useful precedent and
reports that applying them to teamy-tts improved performance. Preserve that
relationship as user-provided context. A bounded source inspection found relevant
mechanisms in both projects, but did not independently establish that historical
attribution or reproduce the performance gains. The first CUDA integration
applies the resource-lifetime and submission principles described below.
Their performance benefit for Bend remains unmeasured.

Use the existing checkouts read-only. Exact machine-specific locations and their
roles are stored in the ignored `.local/gpu-port-references.md` file at this
repository's root. If it is absent on another machine, locate the two checkouts;
do not turn their former locations into source-code defaults.

## Verified reference entry points

Makepad was inspected at `362ac594078fe4b60e24ce8c8a21f29b4123a7b3`.
The listed files had no local changes at inspection:

| Relative path in Makepad | What to examine |
| --- | --- |
| `libs/ai/models/common/src/gpu.rs` | Device-resident operations and backend-specific execution boundaries; some Metal paths combine work into one command buffer. |
| `libs/ai/models/speech/src/indextts_bigvgan_cuda.rs` | Resident parameter buffers, cached weights, fused device kernels and avoiding intermediate host round trips. Domain-specific tensor tricks are examples, not Bend requirements. |
| `libs/ai/models/common/src/metal_accel.rs` | An explicit workload-size threshold before GPU offload and a CPU reference path. The threshold is specific to these operations. |
| `libs/ai/metal/src/bin/metal_residency_bench.rs` | Measuring residency, per-buffer costs and teardown/synchronization rather than assuming unified memory makes them free. |
| `libs/ai/cuda/src/launch.rs` | Explicit streams, device operations and graph/runtime interfaces. |

teamy-tts main was clean at
`c2c90cb7a433a34d077f5f8a77b92ad2d768465a`. Prefer current code and
`native/README.md` over the historical opening status of its `PLAN.md`.

| Relative path in teamy-tts | What to examine |
| --- | --- |
| `native/README.md`, `native/build.rs`, `native/kernels/ops.cu` | Rust-defined computation with ahead-of-time CUDA kernels and vendor primitives; an ordered stream with asynchronous allocations and a retained memory pool. Pool retention is not a hard memory cap. |
| `native/src/cuda.rs` | Owned device/buffer lifetimes and explicit synchronization at host transfer boundaries. |
| `src/runtime_native.rs` | Retaining model data across requests and warming execution before readiness. |
| `native/src/cuda_tests.rs`, `native/tools/compare_cli.py`, `native/tools/benchmark_interactive.py` | Independent correctness checks and distinct launch, first-result and resident-operation measurements. |

The existing `backend-comparison` branch is a historical source of Vulkan
examples (`src/vulkan.rs`, `resources/vulkan/*.comp`); those implementations are
absent from current main. Read them through Git without switching the user's
checkout. The historical `PLAN.md` discussion around lines 1215-1274 records
batched submissions, persistent buffers, device-local suballocation, explicit
staging, host/GPU timing separation and dependency-aware barriers. It also records
failed fusion attempts and a Vulkan candidate that remained slower than its
LibTorch comparison. These are historical reports, not newly reproduced results.

## Decisions applied to Bend

The [generated CUDA runtime](executable-gpu.md) now executes eligible marked Bend
calls. The following decisions apply the inspected principles without making
either reference project a dependency:

| Principle | Current implementation and remaining work |
| --- | --- |
| Retain device resources | One module, stream and set of buffers serve repeated offloads within an invocation. A persistent compilation cache remains open. |
| Keep intermediate work resident | Task graphs and their intermediate values stay on the device until root completion. Each CPU boundary still copies the used corpus and metadata prefix; dirty-region tracking remains open. |
| Order submissions and synchronize explicitly | CPU workers drain before export. Initialization, dispatch and graph delivery use ordered launches with explicit host transfer boundaries. Kernels do not wait for another block's dependency. |
| Bound storage | Device frames and graphs use bounded scratch storage. Allocation budgets include temporary growth; allocator contention and buffer reuse need further performance work. |
| Measure before optimizing | Correctness tests cover actual Bend programs. Cold/warm timing, transfer costs, offload thresholds, fusion and speedup claims still require representative benchmarks. |

CUDA is the first implemented target. Unix CUDA remains unexecuted on the
current validation host, and Metal remains required work. These results do not
establish platform parity or reproduce the reported teamy-tts speedup.

## Remaining evaluation

Continue from the actual upstream requirements in `bend2/comp.ts`: Metal/CUDA
runtime generation, GPU pool handling, device-program compilation/cache and
offload calls. Broaden validation against the surrounding GPU fixtures and CLI
contracts before treating the current execution milestone as parity.

Evaluate keeping data resident, reusing/suballocating buffers, batching compatible
work, reducing transfers and synchronization, and selective kernel fusion or
specialization. Profile the Bend workload first: an optimization for tensor
inference need not benefit graph evaluation or preserve its semantics.

Keep the selected targets, lifetime/ownership model and synchronization
boundaries documented. Validate against upstream behavior and representative release-build
workloads; distinguish cold compilation/upload, warm execution, host submission,
device time and readback. Preserve required numerical behavior and document any
justified target-specific differences. Report unavailable hardware as unverified.

Consultation does not select Makepad as a dependency or mandate CUDA, Vulkan,
Metal, a UI framework or a specific performance gain. Review provenance and
licensing before copying code. Do not broaden Poche or rebuild Bevy to exercise
this work; use focused Bend compatibility workloads.
