// SPDX-License-Identifier: Apache-2.0
// Derived from Bend 2.0.5's Apache-2.0 licensed kernel, Copyright 2026 HigherOrderCO.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
//! The Bend dependent affine proof kernel.
//!
//! Terms use unique binder identifiers rather than host-language closures. No
//! unchecked definition is unfolded during checking. Resource limits reject a
//! check; they never turn an incomplete calculation into evidence.

mod check;
mod fresh;
mod reduce;
mod term;

pub use check::CheckedBook;
pub use check::KernelError;
pub use check::check_book;
pub use term::AdtDecl;
pub use term::Binder;
pub use term::Book;
pub use term::ConstructorDecl;
pub use term::Declaration;
pub use term::DefDecl;
pub use term::LetBinding;
pub use term::Quant;
pub use term::Term;
pub use term::TermRef;
pub use term::arrows;
pub use term::lambdas;
pub use term::substitute;
pub use term::term;
