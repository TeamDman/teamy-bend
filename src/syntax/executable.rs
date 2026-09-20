// SPDX-License-Identifier: MPL-2.0
//! Loader-owned execution contracts, deliberately separate from proof books.

use crate::kernel::Book;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// A parsed program whose foreign signatures are external execution contracts.
///
/// This value is created only by [`super::load_executable`]. Its private fields
/// prevent callers from granting their own declarations bundled-library origin.
///
/// ```compile_fail
/// use teamy_bend::syntax::ExecutableSource;
/// fn forge_base_origin(source: &mut ExecutableSource) {
///     source.base_names.insert("IO".into());
/// }
/// ```
#[derive(Debug)]
pub struct ExecutableSource {
    pub(crate) book: Book,
    pub(crate) foreign: BTreeMap<String, ForeignDefinition>,
    pub(crate) base_names: BTreeSet<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BuiltinForeign {
    Print,
    Write,
    PrintErr,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ForeignTarget {
    C,
    JavaScript,
}

#[derive(Clone, Debug)]
pub(crate) struct ForeignImport {
    pub(crate) target: ForeignTarget,
    pub(crate) path: PathBuf,
}

#[derive(Clone, Debug)]
pub(crate) struct ForeignDefinition {
    pub(crate) imports: Vec<ForeignImport>,
    pub(crate) local_symbol: String,
    pub(crate) declared_arity: usize,
    pub(crate) parameters: Vec<String>,
    pub(crate) builtin: Option<BuiltinForeign>,
}
