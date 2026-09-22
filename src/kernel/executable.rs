// SPDX-License-Identifier: Apache-2.0
// Derived from Bend 2.0.5, Copyright 2026 HigherOrderCO, Apache-2.0.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
//! Execution contracts are deliberately distinct from strict proof evidence.

use super::Declaration;
use super::DefDecl;
use super::KernelError;
use super::Quant;
use super::Term;
use super::TermRef;
use super::check::Engine;
use super::check::empty_engine;
use super::reduce::spine;
use super::term;
use crate::syntax::ExecutableSource;
use crate::syntax::executable::ForeignDefinition;
use crate::syntax::executable::ForeignTarget;
use crate::syntax::executable::NumericIntrinsic;
use crate::syntax::executable::OpaqueType;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;

/// The execution behavior of a filled, nullary `main` definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutableEntry {
    /// There is no `main`; running reports successful checking.
    Missing,
    /// The ordinary `main` value is normalized and printed.
    Pure,
    /// The actual bundled Base IO action is handled by the effect driver.
    Io,
}

/// A checked program that may rely on foreign contracts or unsafe definitions.
///
/// This type is not a proof certificate and cannot become a [`super::CheckedBook`].
/// Foreign results and unsafe definitions remain execution assumptions even if
/// their types mention equalities. Unsafe definitions can diverge and can grant
/// reusable access to values whose types are ordinarily affine.
/// There is intentionally no public proof-normalization or evaluation API.
///
/// ```compile_fail
/// use teamy_bend::kernel::{CheckedBook, ExecutableBook};
/// fn require_proof(_: CheckedBook) {}
/// fn invalid_conversion(executable: ExecutableBook) {
///     require_proof(executable);
/// }
/// ```
///
/// ```compile_fail
/// use teamy_bend::kernel::ExecutableBook;
/// fn invalid_proof_evaluation(executable: ExecutableBook) {
///     executable.evaluate("foreign_proof", &[]);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ExecutableBook {
    engine: Engine,
    runtime: crate::runtime::Program,
    foreign: BTreeMap<String, ForeignDefinition>,
    numeric: BTreeMap<String, NumericIntrinsic>,
    opaque: BTreeMap<String, OpaqueType>,
    constructor_tags: BTreeMap<String, String>,
    base_names: BTreeSet<String>,
}

impl ExecutableBook {
    pub(crate) fn lower_for_compilation(
        &self,
    ) -> Result<super::elaborate::ExecutableProgram, KernelError> {
        super::elaborate::lower(
            &self.engine,
            &self.foreign,
            &self.numeric,
            &self.opaque,
            &self.constructor_tags,
            &self.base_names,
            self.entry_kind()?,
        )
    }

    /// Look up a checked signature; foreign and unsafe signatures are assumptions.
    #[must_use]
    pub fn definition_type(&self, name: &str) -> Option<&TermRef> {
        self.engine.defs.get(name).map(|definition| &definition.ty)
    }

    /// Names of ordinary and foreign definitions, in lexical order.
    pub fn definition_names(&self) -> impl Iterator<Item = &str> {
        self.engine.defs.keys().map(String::as_str)
    }

    /// Names of the foreign contracts assumed by this executable program.
    pub fn foreign_names(&self) -> impl Iterator<Item = &str> {
        self.foreign.keys().map(String::as_str)
    }

    /// Names of definitions carrying execution-only `@unsafe` assumptions.
    pub fn unsafe_names(&self) -> impl Iterator<Item = &str> {
        self.engine
            .defs
            .values()
            .filter(|definition| definition.unsafe_)
            .map(|definition| definition.name.as_str())
    }

    /// Names of opaque numeric runtime contracts; these are not checked proofs.
    pub fn numeric_names(&self) -> impl Iterator<Item = &str> {
        self.numeric.keys().map(String::as_str)
    }

    /// Names of sealed execution-only type families, with no constructors or body.
    pub fn opaque_names(&self) -> impl Iterator<Item = &str> {
        self.opaque.keys().map(String::as_str)
    }

    /// The host-language symbol used for a foreign contract, before namespacing.
    #[must_use]
    pub fn foreign_symbol(&self, name: &str) -> Option<&str> {
        self.foreign
            .get(name)
            .map(|metadata| metadata.local_symbol.as_str())
    }

    /// Ordered target labels (`c` or `js`) and resolved foreign import paths.
    ///
    /// Paths describe assumed implementations; querying them does not execute or
    /// certify the target source. Bundled intrinsic paths are virtual resources.
    #[must_use]
    pub fn foreign_imports(
        &self,
        name: &str,
    ) -> Option<impl Iterator<Item = (&'static str, &Path)>> {
        self.foreign.get(name).map(|metadata| {
            metadata.imports.iter().map(|import| {
                let target = match import.target {
                    ForeignTarget::C => "c",
                    ForeignTarget::JavaScript => "js",
                };
                (target, import.path.as_path())
            })
        })
    }

    /// Classify `main`, unfolding type aliases while keeping actual Base IO opaque.
    ///
    /// # Errors
    /// Rejects foreign, unfilled or parameterized entry definitions, and exhausted
    /// reduction limits while identifying the return type.
    pub fn entry_kind(&self) -> Result<ExecutableEntry, KernelError> {
        let Some(main) = self.engine.defs.get("main") else {
            return Ok(ExecutableEntry::Missing);
        };
        if main.foreign || main.body.is_none() {
            return Err(KernelError::new(
                "main must be a filled ordinary definition; a foreign main cannot anchor execution",
            ));
        }
        if !main.parameters.is_empty() {
            return Err(KernelError::new("main must have no declared parameters"));
        }
        if !self.base_names.contains("IO") {
            return Ok(ExecutableEntry::Pure);
        }
        let Some(io) = self.engine.defs.get("IO") else {
            return Ok(ExecutableEntry::Pure);
        };
        let mut opaque = io.clone();
        opaque.body = None;
        let mut engine = self.engine.clone();
        engine.reset();
        Rc::make_mut(&mut engine.defs).insert("IO".into(), opaque);
        let result = engine.whnf(&main.ty)?;
        let (head, arguments) = spine(&result);
        if matches!(head.as_ref(), Term::Ref(name) | Term::GpuRef(name) if name == "IO")
            && arguments.len() == 1
        {
            Ok(ExecutableEntry::Io)
        } else {
            Ok(ExecutableEntry::Pure)
        }
    }

    /// Run the nullary main while preserving separate stdout and stderr streams.
    ///
    /// IO payloads are discarded at successful completion; only Halt supplies a
    /// nonzero program exit code. Pure output is the canonical core term syntax.
    ///
    /// # Errors
    /// Rejects invalid entry points, unsupported effects, cancellation, write
    /// failures, and exhausted checking or execution limits.
    pub fn run_main(
        &self,
        stdout: &mut dyn Write,
        stderr: &mut dyn Write,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<u32, KernelError> {
        if cancelled() {
            return Err(KernelError::new("execution cancelled"));
        }
        match self.entry_kind()? {
            ExecutableEntry::Missing => {
                stdout
                    .write_all(b"All terms check.\n")
                    .map_err(|error| KernelError::new(format!("stdout write failed: {error}")))?;
                Ok(0)
            }
            ExecutableEntry::Pure => {
                let mut engine = self.engine.clone();
                engine.reset();
                let result = engine.normalize(&term(Term::Ref("main".into())))?;
                if cancelled() {
                    return Err(KernelError::new("execution cancelled"));
                }
                writeln!(stdout, "{result}")
                    .map_err(|error| KernelError::new(format!("stdout write failed: {error}")))?;
                Ok(0)
            }
            ExecutableEntry::Io => self.runtime.run_io("main", stdout, stderr, cancelled),
        }
    }
}

fn strip_annotations(mut value: &TermRef) -> &TermRef {
    while let Term::Ann(inner, _) = value.as_ref() {
        value = inner;
    }
    value
}

fn foreign_contract(
    source: &ExecutableSource,
    engine: &Engine,
    definition: &DefDecl,
) -> Result<(), KernelError> {
    let metadata = source.foreign.get(&definition.name).ok_or_else(|| {
        KernelError::new("foreign definition has no trusted loader-origin metadata")
    })?;
    if definition.body.is_some()
        || metadata.declared_arity != definition.parameters.len()
        || metadata.parameters.len() != metadata.declared_arity
        || metadata.imports.is_empty()
    {
        return Err(KernelError::new(
            "inconsistent foreign declaration metadata",
        ));
    }
    if metadata.builtin.is_some() && !source.base_names.contains(&definition.name) {
        return Err(KernelError::new(
            "intrinsic contract did not originate in bundled Base",
        ));
    }
    if metadata.builtin.is_some() && definition.unsafe_ {
        return Err(KernelError::new(
            "bundled foreign contracts cannot acquire unsafe assumptions",
        ));
    }
    // This is intentionally syntactic. In contrast to entry detection, upstream
    // does not unfold aliases in a foreign declaration's final return position.
    let mut result = strip_annotations(&definition.ty);
    while let Term::All { body, .. } = result.as_ref() {
        result = strip_annotations(body);
    }
    let (head, arguments) = spine(result);
    let actual_io = source.base_names.contains("IO")
        && engine
            .defs
            .get("IO")
            .is_some_and(|io| !io.foreign && io.body.is_some());
    if !actual_io
        || !matches!(head.as_ref(), Term::Ref(name) | Term::GpuRef(name) if name == "IO")
        || arguments.len() != 1
    {
        return Err(KernelError::new(
            "a foreign definition must return actual Base IO(...) directly; return aliases are not unfolded",
        ));
    }
    Ok(())
}

fn opaque_contract(
    source: &ExecutableSource,
    definition: &DefDecl,
    opaque: OpaqueType,
) -> Result<(), KernelError> {
    if !source.base_names.contains(&definition.name)
        || OpaqueType::bundled(&definition.name) != Some(opaque)
        || definition.body.is_some()
        || definition.foreign
        || definition.unsafe_
    {
        return Err(KernelError::new("invalid bundled opaque type metadata"));
    }
    let is_kind = |value: &TermRef, expected| {
        matches!(strip_annotations(value).as_ref(), Term::Typ(quantity)
            if matches!(quantity.as_ref(), Term::Qua(actual) if *actual == expected))
    };
    match opaque {
        OpaqueType::File | OpaqueType::Socket | OpaqueType::Listener => {
            return if definition.parameters.is_empty() && is_kind(&definition.ty, Quant::Lone) {
                Ok(())
            } else {
                Err(KernelError::new(format!(
                    "opaque {} signature must be exactly Type",
                    definition.name
                )))
            };
        }
        OpaqueType::Chan => {}
    }
    if definition.parameters.len() != 1 {
        return Err(KernelError::new(
            "opaque Chan requires exactly one erased type parameter",
        ));
    }
    let Term::All {
        quant,
        id,
        domain,
        body,
        ..
    } = strip_annotations(&definition.ty).as_ref()
    else {
        return Err(KernelError::new(
            "opaque Chan requires exactly one erased type parameter",
        ));
    };
    let parameter = &definition.parameters[0];
    if *quant != Quant::None
        || parameter.quant != Quant::None
        || *id != parameter.id
        || !is_kind(domain, Quant::Lone)
        || !is_kind(&parameter.ty, Quant::Lone)
        || !is_kind(body, Quant::Many)
    {
        return Err(KernelError::new(
            "opaque Chan signature must be exactly @-A:Type -> Data",
        ));
    }
    Ok(())
}

fn numeric_contract(
    source: &ExecutableSource,
    engine: &Engine,
    definition: &DefDecl,
    intrinsic: NumericIntrinsic,
) -> Result<(), KernelError> {
    if !source.base_names.contains(&definition.name)
        || NumericIntrinsic::bundled(&definition.name) != Some(intrinsic)
        || definition.parameters.len() != intrinsic.arity()
    {
        return Err(KernelError::new(
            "invalid bundled numeric contract metadata",
        ));
    }
    // Read additionally carries F32 in the actual Maybe<&2, F32> family.
    for name in [intrinsic.input_type(), intrinsic.output_type(), "F32"] {
        if !source.base_names.contains(name) || !engine.adts.contains_key(name) {
            return Err(KernelError::new(
                "numeric signatures require actual Base datatypes",
            ));
        }
    }
    let mut result = strip_annotations(&definition.ty);
    for parameter in &definition.parameters {
        let Term::All {
            quant,
            id,
            domain,
            body,
            ..
        } = result.as_ref()
        else {
            return Err(KernelError::new(
                "numeric contract has an invalid function telescope",
            ));
        };
        if *quant != intrinsic.input_quant()
            || parameter.quant != intrinsic.input_quant()
            || *id != parameter.id
            || !matches!(strip_annotations(domain).as_ref(), Term::Ref(name) | Term::GpuRef(name) if name == intrinsic.input_type())
            || !matches!(strip_annotations(&parameter.ty).as_ref(), Term::Ref(name) | Term::GpuRef(name) if name == intrinsic.input_type())
        {
            return Err(KernelError::new(
                "numeric contract has an invalid parameter",
            ));
        }
        result = strip_annotations(body);
    }
    let valid_result = if intrinsic == NumericIntrinsic::Read {
        matches!(result.as_ref(), Term::Adt { name, args, excluded }
            if name == "Maybe" && excluded.is_empty() && args.len() == 2
                && matches!(strip_annotations(&args[0]).as_ref(), Term::Qua(Quant::Many))
                && matches!(strip_annotations(&args[1]).as_ref(), Term::Ref(name) | Term::GpuRef(name) if name == "F32"))
    } else {
        matches!(result.as_ref(), Term::Ref(name) | Term::GpuRef(name) if name == intrinsic.output_type())
    };
    if !valid_result {
        return Err(KernelError::new(
            "numeric contract has an invalid result type",
        ));
    }
    Ok(())
}

/// Check ordinary definitions and executable foreign IO contracts in source order.
///
/// The loader alone constructs [`ExecutableSource`] and identifies actual Base
/// declarations. Annotated definitions use upstream's execution-only exceptions
/// to structural descent and reusable-domain formation; quantities, equality and
/// declaration order remain checked. Foreign bodies remain opaque assumptions.
///
/// # Errors
/// Rejects invalid signatures, false proofs, holes, unfilled ordinary laws,
/// forged effect contracts and exhausted checking limits. Accepted unsafe
/// definitions do not become strict proof evidence.
pub fn check_executable(source: &ExecutableSource) -> Result<ExecutableBook, KernelError> {
    let mut engine = empty_engine(&source.book)?;
    // Upstream stores a law and its later body in one definition object. Its
    // eventual annotation also governs signature formation at the earlier law
    // event. Retain our source-order events: this pre-scan supplies no bodies,
    // and unfilled laws still cannot be used as live evidence.
    let unsafe_definitions = source
        .book
        .declarations
        .iter()
        .filter_map(|declaration| match declaration {
            Declaration::Def(definition) if definition.unsafe_ => Some(&definition.name),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    for declaration in &source.book.declarations {
        engine.reset();
        let (name, result) = match declaration {
            Declaration::Adt(adt) => (&adt.name, engine.validate_adt(adt)),
            Declaration::Def(definition) => {
                let result = if let Some(opaque) = source.opaque.get(&definition.name) {
                    opaque_contract(source, definition, *opaque)
                        .and_then(|()| engine.validate_opaque_definition(definition))
                } else if definition.foreign {
                    foreign_contract(source, &engine, definition)
                        .and_then(|()| engine.validate_foreign_definition(definition))
                } else if let Some(intrinsic) = source.numeric.get(&definition.name) {
                    numeric_contract(source, &engine, definition, *intrinsic)
                        .and_then(|()| engine.validate_numeric_definition(definition))
                } else {
                    engine.validate_executable_definition(
                        definition,
                        unsafe_definitions.contains(&definition.name),
                    )
                };
                (&definition.name, result)
            }
        };
        result.map_err(|error| KernelError::new(format!("{name}: {error}")))?;
    }
    let open = engine
        .defs
        .values()
        .filter(|definition| {
            definition.body.is_none()
                && !engine.foreign_contracts.contains(&definition.name)
                && !engine.numeric_contracts.contains(&definition.name)
                && !engine.opaque_contracts.contains(&definition.name)
        })
        .map(|definition| definition.name.as_str())
        .collect::<Vec<_>>();
    if !open.is_empty() {
        return Err(KernelError::new(format!(
            "unfilled laws: {}",
            open.join(", ")
        )));
    }
    if source
        .foreign
        .keys()
        .any(|name| !engine.foreign_contracts.contains(name))
    {
        return Err(KernelError::new(
            "foreign metadata has no corresponding declaration",
        ));
    }
    if source
        .numeric
        .keys()
        .any(|name| !engine.numeric_contracts.contains(name))
    {
        return Err(KernelError::new(
            "numeric metadata has no corresponding declaration",
        ));
    }
    if source
        .opaque
        .keys()
        .any(|name| !engine.opaque_contracts.contains(name))
    {
        return Err(KernelError::new(
            "opaque metadata has no corresponding declaration",
        ));
    }
    let runtime = crate::runtime::Program::from_executable(
        &engine.defs,
        &engine.adts,
        &source.foreign,
        &source.numeric,
        &source.base_names,
    );
    Ok(ExecutableBook {
        engine,
        runtime,
        foreign: source.foreign.clone(),
        numeric: source.numeric.clone(),
        opaque: source.opaque.clone(),
        constructor_tags: source.constructor_tags.clone(),
        base_names: source.base_names.clone(),
    })
}
