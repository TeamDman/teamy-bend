// SPDX-License-Identifier: MPL-2.0
//! Native code generation from complete, checked Bend books.
//!
//! Standalone JavaScript and portable C11 programs preserve lazy evaluation and
//! higher-order closures for closed data-producing entries. Both represent
//! erased types and proofs with an explicit erased marker.

mod c;
mod executable_c;
mod executable_c_foreign;
mod executable_c_layout;
mod executable_foreign;
mod executable_javascript;
mod javascript;

pub use c::compile_c;
pub use executable_c::compile_executable_c;
pub use executable_javascript::compile_executable_javascript;
pub use javascript::CompileError;
pub use javascript::compile_javascript;
