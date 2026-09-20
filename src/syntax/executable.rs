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
///
/// ```compile_fail
/// use teamy_bend::syntax::ExecutableSource;
/// fn forge_numeric_origin(source: &mut ExecutableSource) {
///     source.numeric.clear();
/// }
/// ```
#[derive(Debug)]
pub struct ExecutableSource {
    pub(crate) book: Book,
    pub(crate) foreign: BTreeMap<String, ForeignDefinition>,
    pub(crate) numeric: BTreeMap<String, NumericIntrinsic>,
    pub(crate) constructor_tags: BTreeMap<String, String>,
    pub(crate) base_names: BTreeSet<String>,
}

/// Execution-only numeric operations; the loader mints these only for bundled laws.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumericIntrinsic {
    U32ToF32,
    F32ToU32,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    Abs,
    Bits,
    IsEq,
    IsNe,
    IsLt,
    IsLe,
    IsGt,
    IsGe,
}

impl NumericIntrinsic {
    pub(crate) fn bundled(name: &str) -> Option<Self> {
        Some(match name {
            "U32.to_f32" => Self::U32ToF32,
            "F32.to_u32" => Self::F32ToU32,
            "F32.add" => Self::Add,
            "F32.sub" => Self::Sub,
            "F32.mul" => Self::Mul,
            "F32.div" => Self::Div,
            "F32.mod" => Self::Mod,
            "F32.neg" => Self::Neg,
            "F32.abs" => Self::Abs,
            "F32.bits" => Self::Bits,
            "F32.is_eq" => Self::IsEq,
            "F32.is_ne" => Self::IsNe,
            "F32.is_lt" => Self::IsLt,
            "F32.is_le" => Self::IsLe,
            "F32.is_gt" => Self::IsGt,
            "F32.is_ge" => Self::IsGe,
            _ => return None,
        })
    }

    pub(crate) const fn arity(self) -> usize {
        match self {
            Self::U32ToF32 | Self::F32ToU32 | Self::Neg | Self::Abs | Self::Bits => 1,
            _ => 2,
        }
    }

    pub(crate) const fn input_type(self) -> &'static str {
        match self {
            Self::U32ToF32 => "U32",
            _ => "F32",
        }
    }

    pub(crate) const fn output_type(self) -> &'static str {
        match self {
            Self::F32ToU32 | Self::Bits => "U32",
            Self::IsEq | Self::IsNe | Self::IsLt | Self::IsLe | Self::IsGt | Self::IsGe => "Bool",
            _ => "F32",
        }
    }
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
