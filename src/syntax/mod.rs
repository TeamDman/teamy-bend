// SPDX-License-Identifier: Apache-2.0
// Derived from Bend 2.0.5, Copyright 2026 HigherOrderCO, Apache-2.0.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
//! Bend source parsing and local module loading.

pub(crate) mod executable;
#[cfg(test)]
mod executable_tests;
mod parser;
mod surface;

pub use executable::ExecutableSource;
pub use parser::ParseError;
pub use parser::load;
pub use parser::load_executable;
pub use parser::load_executable_with_sources;
pub use parser::load_with_sources;
pub use parser::parse;
pub use parser::parse_term;
