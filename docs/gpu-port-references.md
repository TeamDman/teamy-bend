# References for the future Bend2 GPU port

Recorded 2026-09-20 for guidance U11-U13 in the implementation plan. Read this
when GPU work becomes the active phase; native scheduling remains the next
engine milestone.

## User guidance and provenance

The user identifies Rik Arends's Makepad GPU strategies as useful precedent and
reports that applying them to teamy-tts improved performance. Preserve that
relationship as user-provided context. A bounded source inspection found relevant
mechanisms in both projects, but did not independently establish that historical
attribution or reproduce the performance gains. Transferability to Bend remains
an open design question.

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

## Decision gate for Bend

Start from the actual upstream requirements in `bend2/comp.ts`: Metal/CUDA
runtime generation, GPU pool handling, device-program compilation/cache and
offload calls. Inspect the surrounding GPU fixtures and CLI contracts before
mapping mechanisms from either reference onto them.

Evaluate keeping data resident, reusing/suballocating buffers, batching compatible
work, reducing transfers and synchronization, and selective kernel fusion or
specialization. Profile the Bend workload first: an optimization for tensor
inference need not benefit graph evaluation or preserve its semantics.

Record the selected targets, lifetime/ownership model and synchronization
boundaries. Validate against upstream behavior and representative release-build
workloads; distinguish cold compilation/upload, warm execution, host submission,
device time and readback. Preserve required numerical behavior and document any
justified target-specific differences. Report unavailable hardware as unverified.

Consultation does not select Makepad as a dependency or mandate CUDA, Vulkan,
Metal, a UI framework or a specific performance gain. Review provenance and
licensing before copying code. Do not broaden Poche or rebuild Bevy to exercise
this work; use focused Bend compatibility workloads.
