// SPDX-License-Identifier: MPL-2.0
//! Native code generation from complete, checked Bend books.
//!
//! The initial backend emits standalone JavaScript for closed data-producing
//! entry points. It preserves lazy evaluation and higher-order closures, while
//! representing erased types and proofs with an explicit erased marker.

mod javascript;

pub use javascript::CompileError;
pub use javascript::compile_javascript;
