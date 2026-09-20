// SPDX-License-Identifier: MPL-2.0
//! Loader-owned execution contracts, deliberately separate from proof books.

use crate::kernel::Book;
use crate::kernel::Quant;
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
    Pow,
    Atan2,
    Neg,
    Abs,
    Sqrt,
    Exp,
    Log,
    Log2,
    Log10,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Floor,
    Ceil,
    Trunc,
    Show,
    Read,
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
            "F32.pow" => Self::Pow,
            "F32.atan2" => Self::Atan2,
            "F32.neg" => Self::Neg,
            "F32.abs" => Self::Abs,
            "F32.sqrt" => Self::Sqrt,
            "F32.exp" => Self::Exp,
            "F32.log" => Self::Log,
            "F32.log2" => Self::Log2,
            "F32.log10" => Self::Log10,
            "F32.sin" => Self::Sin,
            "F32.cos" => Self::Cos,
            "F32.tan" => Self::Tan,
            "F32.asin" => Self::Asin,
            "F32.acos" => Self::Acos,
            "F32.atan" => Self::Atan,
            "F32.sinh" => Self::Sinh,
            "F32.cosh" => Self::Cosh,
            "F32.tanh" => Self::Tanh,
            "F32.floor" => Self::Floor,
            "F32.ceil" => Self::Ceil,
            "F32.trunc" => Self::Trunc,
            "F32.show" => Self::Show,
            "F32.read" => Self::Read,
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
            Self::Add
            | Self::Sub
            | Self::Mul
            | Self::Div
            | Self::Mod
            | Self::Pow
            | Self::Atan2
            | Self::IsEq
            | Self::IsNe
            | Self::IsLt
            | Self::IsLe
            | Self::IsGt
            | Self::IsGe => 2,
            _ => 1,
        }
    }

    pub(crate) const fn input_quant(self) -> Quant {
        match self {
            Self::Show => Quant::Many,
            _ => Quant::Lone,
        }
    }

    pub(crate) const fn input_type(self) -> &'static str {
        match self {
            Self::U32ToF32 => "U32",
            Self::Read => "String",
            _ => "F32",
        }
    }

    pub(crate) const fn output_type(self) -> &'static str {
        match self {
            Self::F32ToU32 | Self::Bits => "U32",
            Self::Show => "String",
            Self::Read => "Maybe",
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
    Spawn,
    Sleep,
    Now,
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
