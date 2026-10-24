// SPDX-License-Identifier: Apache-2.0
// Derived from Bend 2.0.5, Copyright 2026 HigherOrderCO, Apache-2.0.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
use super::AdtDecl;
use super::Binder;
use super::Book;
use super::ConstructorDecl;
use super::Declaration;
use super::DefDecl;
use super::Quant;
use super::Term;
use super::TermRef;
use super::arrows;
use super::lambdas;
use super::reduce::Comparison;
use super::reduce::apply;
use super::reduce::spine;
use super::substitute;
use super::term;
use super::term::inspect;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;
use std::rc::Rc;

type Uses = BTreeMap<usize, Quant>;
// Semantic lookup and dead-context traversal retain their original ID order.
// Diagnostic contexts follow binder admission, like upstream de Bruijn indices;
// parser IDs can precede fresh field IDs even when admitted after those fields.
#[derive(Clone, Debug, Default)]
struct Context {
    bindings: BTreeMap<usize, Binder>,
    diagnostic_order: Vec<usize>,
}
impl Context {
    fn new() -> Self {
        Self::default()
    }
    fn insert(&mut self, id: usize, binder: Binder) -> Option<Binder> {
        let previous = self.bindings.insert(id, binder);
        if previous.is_none() {
            self.diagnostic_order.push(id);
        }
        previous
    }
    fn get(&self, id: usize) -> Option<&Binder> {
        self.bindings.get(&id)
    }
    fn contains_key(&self, id: usize) -> bool {
        self.bindings.contains_key(&id)
    }
    fn values(&self) -> impl Iterator<Item = &Binder> {
        self.bindings.values()
    }
    fn diagnostic_values(&self) -> impl Iterator<Item = &Binder> {
        self.diagnostic_order.iter().map(|id| &self.bindings[id])
    }
}
impl<const N: usize> From<[(usize, Binder); N]> for Context {
    fn from(bindings: [(usize, Binder); N]) -> Self {
        let mut context = Self::new();
        for (id, binder) in bindings {
            context.insert(id, binder);
        }
        context
    }
}

/// A rejected check or execution, retaining internal diagnostic context.
///
/// Classification is private: callers can read the message or use Display,
/// but cannot construct an error with a struct literal. In particular, diagnostic
/// text alone never grants an internal error the language-exit contract.
#[derive(Clone, Debug, Eq)]
pub struct KernelError {
    pub message: String,
    kind: FailureKind,
    diagnostic: Option<Box<CheckerDiagnostic>>,
}

// Optional presentation metadata does not change the existing error equality
// contract or make errors depend on a thread-local, non-Send checker engine.
impl PartialEq for KernelError {
    fn eq(&self, other: &Self) -> bool {
        self.message == other.message && self.kind == other.kind
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CheckerDiagnostic {
    pub(crate) term_identity: usize,
    pub(crate) match_site: Option<super::origins::MatchSite>,
    pub(crate) definition: String,
    pub(crate) expected: String,
    pub(crate) observed: Option<String>,
    pub(crate) context: Vec<(String, String)>,
}

// Structured checker facts, not interpretation of a human-readable error string.
enum DiagnosticExpression<'a> {
    Text(String),
    Term(&'a TermRef),
    Typeless(&'a TermRef),
}
impl DiagnosticExpression<'_> {
    fn render(self, engine: &mut Engine, scope: &[(usize, &str)]) -> Result<String, KernelError> {
        match self {
            Self::Text(text) => Ok(text),
            Self::Term(value) => {
                Ok(super::term::scoped(engine.normalize(value)?.as_ref(), scope).to_string())
            }
            Self::Typeless(value) => Ok(format!(
                "non-inferrable term '{}'",
                super::term::scoped(engine.normalize(value)?.as_ref(), scope)
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FailureKind {
    Internal,
    RequestOutOfBand,
    NatOverflow,
    ArrayDepth,
}
impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl Error for KernelError {}
impl KernelError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: FailureKind::Internal,
            diagnostic: None,
        }
    }

    pub(crate) fn request_out_of_band(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: FailureKind::RequestOutOfBand,
            diagnostic: None,
        }
    }

    pub(crate) fn nat_overflow(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: FailureKind::NatOverflow,
            diagnostic: None,
        }
    }

    pub(crate) fn array_depth(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: FailureKind::ArrayDepth,
            diagnostic: None,
        }
    }

    pub(crate) fn in_definition(mut self, name: &str) -> Self {
        self.message = format!("{name}: {}", self.message);
        if let Some(report) = &mut self.diagnostic
            && report.definition.is_empty()
        {
            name.clone_into(&mut report.definition);
        }
        self
    }

    pub(crate) fn checker_diagnostic(&self) -> Option<&CheckerDiagnostic> {
        self.diagnostic.as_deref()
    }

    pub(crate) const fn language_failure(&self) -> Option<(&'static str, &'static [u8])> {
        match self.kind {
            FailureKind::Internal => None,
            FailureKind::RequestOutOfBand => {
                Some(("request_out_of_band", b"bend: runtime fail-stop\n"))
            }
            FailureKind::NatOverflow => Some((
                "nat_overflow",
                b"bend: a Nat past the largest immediate 2^48-1\n",
            )),
            FailureKind::ArrayDepth => Some((
                "array_depth",
                b"bend: an array past the deepest block class 31\n",
            )),
        }
    }
}

/// Definitions can be evaluated only after the complete book passes checking.
#[derive(Clone, Debug)]
pub struct CheckedBook {
    engine: Engine,
    runtime: crate::runtime::Program,
}

impl CheckedBook {
    /// Normalize a checked definition as a value, retaining its parameters.
    ///
    /// This does not call the definition. Argument-taking APIs continue to
    /// require complete, well-typed applications.
    ///
    /// # Errors
    /// Rejects unknown definitions and exhausted normalization limits.
    pub fn normalize_definition(&self, name: &str) -> Result<TermRef, KernelError> {
        let mut engine = self.engine.clone();
        engine.reset();
        engine.normalize_definition(name)
    }

    /// Look up a checked definition's complete dependent function type.
    #[must_use]
    pub fn definition_type(&self, name: &str) -> Option<&TermRef> {
        self.engine.defs.get(name).map(|d| &d.ty)
    }

    /// Evaluate a completely applied checked definition with checked arguments.
    ///
    /// # Errors
    /// Rejects unknown definitions, incorrect arguments, and exhausted limits.
    pub fn evaluate(&self, name: &str, args: &[TermRef]) -> Result<TermRef, KernelError> {
        let mut engine = self.check_arguments(name, args)?;
        engine.normalize(&apply(
            term(Term::Ref(name.into())),
            args.iter().map(Rc::clone),
        ))
    }

    /// Evaluate checked arguments with the bounded lazy constructor runtime.
    ///
    /// Argument arity and dependent types are checked on every call. Runtime
    /// thunks and memoized values belong to that call; proof checking continues
    /// to use the kernel normalizer exclusively.
    ///
    /// # Errors
    /// Rejects invalid arguments, non-data results, and exhausted runtime limits.
    pub fn evaluate_data(&self, name: &str, args: &[TermRef]) -> Result<TermRef, KernelError> {
        self.check_arguments(name, args)?;
        self.runtime.evaluate(name, args)
    }

    fn check_arguments(&self, name: &str, args: &[TermRef]) -> Result<Engine, KernelError> {
        let mut engine = self.engine.clone();
        engine.reset();
        for arg in args {
            let highest = inspect(arg)?;
            engine.fresh_id = engine.fresh_id.max(
                highest
                    .checked_add(1)
                    .ok_or_else(|| KernelError::new("binder identifier space exhausted"))?,
            );
        }
        let def = engine
            .defs
            .get(name)
            .cloned()
            .ok_or_else(|| KernelError::new(format!("undefined definition {name}")))?;
        if args.len() != def.parameters.len() {
            return Err(KernelError::new(format!(
                "{name} needs {} arguments; got {}",
                def.parameters.len(),
                args.len()
            )));
        }
        let mut ty = Rc::clone(&def.ty);
        let lhs = Lhs::default();
        for arg in args {
            let t = engine.whnf(&ty)?;
            let Term::All {
                quant,
                id,
                domain,
                body,
                ..
            } = t.as_ref()
            else {
                return Err(KernelError::new("argument supplied to a non-function"));
            };
            engine.check(
                &lhs,
                arg,
                quant.demand(Quant::Lone),
                domain,
                &Context::new(),
            )?;
            ty = substitute(body, *id, arg);
        }
        Ok(engine)
    }

    /// Names of all checked definitions, in lexical order.
    pub fn definition_names(&self) -> impl Iterator<Item = &str> {
        self.engine.defs.keys().map(String::as_str)
    }
}

#[cfg(test)]
mod benchmark {
    use super::check_book;
    use crate::protocol::DataValue;
    use crate::protocol::Request;
    use crate::protocol::arguments;
    use std::collections::BTreeMap;
    use std::time::Instant;

    fn high_natural(values: &[DataValue]) -> bool {
        let mut work: Vec<_> = values.iter().map(|value| (value, 0)).collect();
        while let Some((value, consecutive)) = work.pop() {
            let next = if value.constructor == "Succ" {
                consecutive + 1
            } else {
                0
            };
            if next >= 48 {
                return true;
            }
            work.extend(value.fields.iter().map(|field| (field, next)));
        }
        false
    }

    fn profile() {
        let model = std::env::var_os("TEAMY_BEND_BENCH_MODEL").expect("set benchmark model");
        let transcript =
            std::env::var_os("TEAMY_BEND_BENCH_TRANSCRIPT").expect("set benchmark transcript");
        let checked =
            check_book(&crate::syntax::load(std::path::Path::new(&model)).unwrap()).unwrap();
        let transcript = std::fs::read_to_string(transcript).unwrap();
        let requests = transcript
            .lines()
            .filter_map(|line| line.strip_prefix("> "))
            .map(|line| facet_json::from_str::<Request>(line).unwrap())
            .filter(|request| high_natural(&request.args))
            .map(|request| (request.entry, arguments(&request.args).unwrap()))
            .collect::<Vec<_>>();
        assert!(
            !requests.is_empty(),
            "transcript needs input naturals at least48"
        );
        let mut totals = BTreeMap::<String, (usize, u128, u128)>::new();
        for _ in 0..200 {
            for (entry, args) in &requests {
                let at = Instant::now();
                drop(checked.check_arguments(entry, args).unwrap());
                let validation = at.elapsed().as_micros();
                let at = Instant::now();
                std::hint::black_box(checked.runtime.evaluate(entry, args).unwrap());
                let runtime = at.elapsed().as_micros();
                let total = totals.entry(entry.clone()).or_default();
                total.0 += 1;
                total.1 += validation;
                total.2 += runtime;
            }
        }
        for (entry, (calls, validation, runtime)) in totals {
            eprintln!(
                "entry={entry} calls={calls} argument_validation_us={validation} lazy_runtime_us={runtime}"
            );
        }
    }

    #[test]
    #[ignore = "local profiler needs explicit trajectory/model benchmark environment"]
    fn profile_high_score_validation_and_runtime() {
        std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(profile)
            .unwrap()
            .join()
            .unwrap();
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Engine {
    pub(super) origins: Option<super::origins::TermOrigins>,
    // Declaration maps are immutable after checking. Evaluation clones only
    // these shared pointers; each call still owns its aliases and budgets.
    pub(crate) defs: Rc<BTreeMap<String, DefDecl>>,
    pub(crate) adts: Rc<BTreeMap<String, AdtDecl>>,
    pub(crate) aliases: BTreeMap<usize, TermRef>,
    // Populated only by executable checking after source provenance and the
    // direct Base IO return contract are verified. Strict proof books keep it empty.
    pub(super) foreign_contracts: BTreeSet<String>,
    // Numeric contracts are separately sealed and remain opaque to reduction.
    pub(super) numeric_contracts: BTreeSet<String>,
    // Only executable checking can seal opaque type families; never proof books.
    pub(super) opaque_contracts: BTreeSet<String>,
    fresh_id: usize,
    fuel: usize,
    pub(crate) depth: usize,
    pub(super) normalization_memo: Option<super::reduce::NormalizationMemo>,
}

impl Engine {
    fn substitute(&self, value: &TermRef, target: usize, replacement: &TermRef) -> TermRef {
        super::origins::substitute_with_origins(value, target, replacement, self.origins.as_ref())
    }

    fn reflexivity_diagnostic(
        &self,
        lhs: &Lhs,
        proof: &TermRef,
        left: &TermRef,
        right: &TermRef,
        context: &Context,
    ) -> Result<CheckerDiagnostic, KernelError> {
        self.term_diagnostic(
            lhs,
            proof,
            DiagnosticExpression::Term(left),
            DiagnosticExpression::Term(right),
            context,
        )
    }

    fn term_diagnostic(
        &self,
        lhs: &Lhs,
        value: &TermRef,
        expected: DiagnosticExpression<'_>,
        observed: DiagnosticExpression<'_>,
        context: &Context,
    ) -> Result<CheckerDiagnostic, KernelError> {
        self.diagnostic(lhs, value, expected, Some(observed), context)
    }

    fn diagnostic(
        &self,
        lhs: &Lhs,
        value: &TermRef,
        expected: DiagnosticExpression<'_>,
        observed: Option<DiagnosticExpression<'_>>,
        context: &Context,
    ) -> Result<CheckerDiagnostic, KernelError> {
        // The check has already rejected. Presentation shares only the remaining
        // fuel/depth allowance on a clone; it cannot alter acceptance or budgets.
        let mut display = self.clone();
        let scope = context
            .diagnostic_values()
            .map(|binder| (binder.id, binder.name.as_str()))
            .collect::<Vec<_>>();
        let expected = expected.render(&mut display, &scope)?;
        let observed = observed
            .map(|value| value.render(&mut display, &scope))
            .transpose()?;
        let context = context
            .diagnostic_values()
            .enumerate()
            .map(|(index, binder)| {
                Ok((
                    binder.name.clone(),
                    // A binder's type is outside its own scope. Later shadowing
                    // must not change the spelling of earlier context rows.
                    super::term::scoped(display.normalize(&binder.ty)?.as_ref(), &scope[..index])
                        .to_string(),
                ))
            })
            .collect::<Result<Vec<_>, KernelError>>()?;
        Ok(CheckerDiagnostic {
            // Never dereferenced. The loader weakly anchors source allocation identities.
            term_identity: self.origins.as_ref().map_or_else(
                || Rc::as_ptr(value) as usize,
                |origins| origins.source(value),
            ),
            match_site: self
                .origins
                .as_ref()
                .and_then(|origins| origins.match_site(value)),
            definition: lhs.name.clone(),
            expected,
            observed,
            context,
        })
    }

    fn check_usage(
        &self,
        lhs: &Lhs,
        value: &TermRef,
        binder: &Binder,
        uses: &mut Uses,
        context: &Context,
    ) -> Result<(), KernelError> {
        let observed = uses.remove(&binder.id).unwrap_or(Quant::None);
        if observed <= binder.quant {
            return Ok(());
        }
        let mut error = KernelError::new(format!(
            "binder {} permits {:?} use, observed {:?}",
            binder.name, binder.quant, observed
        ));
        // Upstream quant_used reports the closing binder in the outer context.
        // Preserve the internal cause and resource measurement; only the
        // source-facing diagnostic changes after rejection is established.
        let prefix = |quant| match quant {
            Quant::None => "-",
            Quant::Lone => "",
            Quant::Many => "+",
        };
        error.diagnostic = self
            .term_diagnostic(
                lhs,
                value,
                DiagnosticExpression::Text(format!("{}{}", prefix(binder.quant), binder.name)),
                DiagnosticExpression::Text(if observed == Quant::Many {
                    format!("{} (consumed more than once)", binder.name)
                } else {
                    format!("{}{}", prefix(observed), binder.name)
                }),
                context,
            )
            .ok()
            .map(Box::new);
        Err(error)
    }

    fn child_diagnostic(
        &self,
        mut error: KernelError,
        parent: &TermRef,
        index: usize,
        child: &TermRef,
    ) -> KernelError {
        if let Some(report) = &mut error.diagnostic
            && report.match_site.is_none()
            && report.term_identity == Rc::as_ptr(child) as usize
            && let Some(source) = self
                .origins
                .as_ref()
                .and_then(|origins| origins.child_source(parent, index, child))
        {
            report.term_identity = source;
        }
        error
    }

    pub(super) fn normalize_definition(&mut self, name: &str) -> Result<TermRef, KernelError> {
        let definition = self
            .defs
            .get(name)
            .ok_or_else(|| KernelError::new(format!("undefined definition {name}")))?;
        let body = definition
            .body
            .clone()
            .ok_or_else(|| KernelError::new(format!("unfilled definition {name}")))?;
        // Like upstream book_run, normalize the stored function body, not an
        // application with missing arguments. References inside it retain the
        // ordinary normalizer's unfolding and recursion rules.
        self.normalize(&body)
    }

    pub(super) fn reset(&mut self) {
        self.fuel = 2_000_000;
        self.depth = 0;
        self.aliases.clear();
        self.normalization_memo = None;
    }
    pub(crate) fn step(&mut self) -> Result<(), KernelError> {
        self.fuel = self.fuel.checked_sub(1).ok_or_else(|| {
            KernelError::new("kernel evaluation budget exhausted; proof not accepted")
        })?;
        Ok(())
    }
    pub(crate) fn enter(&mut self) -> Result<(), KernelError> {
        if self.depth >= 128 {
            return Err(KernelError::new(
                "kernel nesting limit exhausted; proof not accepted",
            ));
        }
        self.depth += 1;
        Ok(())
    }
    pub(crate) fn fresh(&mut self, name: &str) -> Result<TermRef, KernelError> {
        let id = self.fresh_id;
        self.fresh_id = self
            .fresh_id
            .checked_add(1)
            .ok_or_else(|| KernelError::new("binder identifier space exhausted"))?;
        Ok(term(Term::Var {
            name: name.into(),
            id,
        }))
    }
}

#[derive(Clone, Debug)]
struct Lhs {
    name: String,
    equation: TermRef,
    remaining: usize,
    quants: Vec<Quant>,
    unsafe_: bool,
}
impl Default for Lhs {
    fn default() -> Self {
        Self {
            name: String::new(),
            equation: term(Term::Ref(String::new())),
            remaining: 0,
            quants: vec![],
            unsafe_: false,
        }
    }
}

impl Lhs {
    /// Upstream's execution-only exception changes domain formation, not the
    /// binder's demand or usage count. Strict checking never sets this flag.
    fn domain_kind(&self, quant: Quant) -> TermRef {
        kind(if self.unsafe_ && quant == Quant::Many {
            Quant::Lone
        } else {
            quant
        })
    }
}

fn kind(q: Quant) -> TermRef {
    term(Term::Typ(term(Term::Qua(q))))
}
fn usage(id: usize, q: Quant) -> Uses {
    BTreeMap::from([(id, q)])
}
fn add(mut a: Uses, b: Uses) -> Uses {
    for (id, q) in b {
        let old = a.get(&id).copied().unwrap_or(Quant::None);
        a.insert(id, old.add(q));
    }
    a
}
fn join(mut a: Uses, b: Uses) -> Uses {
    for (id, q) in b {
        let old = a.get(&id).copied().unwrap_or(Quant::None);
        a.insert(id, old.max(q));
    }
    a
}
fn mismatch(expected: &TermRef, observed: &TermRef) -> KernelError {
    KernelError::new(format!(
        "type mismatch: expected {expected}; observed {observed}"
    ))
}

/// Check every declaration and require every law to have a proof.
///
/// # Errors
/// Rejects ill-typed programs, incomplete laws, holes, unsafe/foreign assumptions,
/// invalid resource use or recursion, and exhausted checking limits.
pub fn check_book(book: &Book) -> Result<CheckedBook, KernelError> {
    check_book_with_origins(book, None)
}

pub(crate) fn check_book_with_origins(
    book: &Book,
    origins: Option<super::origins::TermOrigins>,
) -> Result<CheckedBook, KernelError> {
    let (mut engine, source_holes) = prepare_book(book)?;
    engine.origins = origins;
    for (declaration, hole) in book.declarations.iter().zip(source_holes) {
        engine.fuel = 2_000_000;
        let result = match declaration {
            Declaration::Adt(d) => engine.validate_adt(d),
            Declaration::Def(d) => engine.validate_def(d),
        };
        if let Err(error) = result.and_then(|()| reject_source_hole(hole)) {
            let name = match declaration {
                Declaration::Adt(d) => &d.name,
                Declaration::Def(d) => &d.name,
            };
            return Err(error.in_definition(name));
        }
    }
    let open: Vec<_> = engine
        .defs
        .values()
        .filter(|d| d.body.is_none())
        .map(|d| d.name.as_str())
        .collect();
    if !open.is_empty() {
        return Err(KernelError::new(format!(
            "unfilled laws: {}",
            open.join(", ")
        )));
    }
    engine.origins = None;
    let runtime = crate::runtime::Program::from_checked(&engine.defs, &engine.adts);
    Ok(CheckedBook { engine, runtime })
}

// Structural safety and fresh-ID discovery must precede recursive checking.
// Holes instead follow the checker's first-error order. A declaration cannot
// escape validation even if beta reduction would erase its original hole.
pub(super) fn prepare_book(book: &Book) -> Result<(Engine, Vec<Option<String>>), KernelError> {
    let mut highest = 0;
    let mut source_holes = Vec::with_capacity(book.declarations.len());
    for declaration in &book.declarations {
        let mut hole = None;
        let mut inspect = |value: &TermRef| -> Result<(), KernelError> {
            let input = super::term::inspect_for_check(value)?;
            highest = highest.max(input.highest);
            if hole.is_none() {
                hole = input.hole;
            }
            Ok(())
        };
        match declaration {
            Declaration::Def(d) => {
                inspect(&d.ty)?;
                if let Some(b) = &d.body {
                    inspect(b)?;
                }
                highest = highest.max(
                    d.parameters
                        .iter()
                        .map(|p| p.id)
                        .chain(d.completion_parameters.iter().flatten().map(|(_, id)| *id))
                        .max()
                        .unwrap_or(0),
                );
            }
            Declaration::Adt(d) => {
                inspect(&d.kind)?;
                for b in d
                    .parameters
                    .iter()
                    .chain(d.constructors.iter().flat_map(|c| &c.fields))
                {
                    inspect(&b.ty)?;
                }
                highest = highest.max(
                    d.parameters
                        .iter()
                        .chain(d.constructors.iter().flat_map(|c| &c.fields))
                        .map(|b| b.id)
                        .max()
                        .unwrap_or(0),
                );
            }
        }
        source_holes.push(hole);
    }
    Ok((
        Engine {
            origins: None,
            defs: Rc::new(BTreeMap::new()),
            adts: Rc::new(BTreeMap::new()),
            aliases: BTreeMap::new(),
            foreign_contracts: BTreeSet::new(),
            numeric_contracts: BTreeSet::new(),
            opaque_contracts: BTreeSet::new(),
            fresh_id: highest
                .checked_add(1)
                .ok_or_else(|| KernelError::new("binder identifier space exhausted"))?,
            fuel: 2_000_000,
            depth: 0,
            normalization_memo: None,
        },
        source_holes,
    ))
}

pub(super) fn reject_source_hole(hole: Option<String>) -> Result<(), KernelError> {
    hole.map_or(Ok(()), |name| {
        Err(KernelError::new(format!("unfinished proof hole ?{name}")))
    })
}

#[cfg(test)]
fn empty_engine(book: &Book) -> Result<Engine, KernelError> {
    let (engine, holes) = prepare_book(book)?;
    for hole in holes {
        reject_source_hole(hole)?;
    }
    Ok(engine)
}

impl Engine {
    pub(super) fn validate_def(&mut self, definition: &DefDecl) -> Result<(), KernelError> {
        if definition.unsafe_ || definition.foreign {
            return Err(KernelError::new(
                "strict proof checking rejects unsafe and foreign definitions",
            ));
        }
        self.validate_definition(definition, false, false)
    }

    pub(super) fn validate_executable_definition(
        &mut self,
        definition: &DefDecl,
        unsafe_context: bool,
    ) -> Result<(), KernelError> {
        if definition.foreign {
            return Err(KernelError::new("invalid ordinary executable declaration"));
        }
        self.validate_definition(definition, false, unsafe_context)
    }

    pub(super) fn validate_foreign_definition(
        &mut self,
        definition: &DefDecl,
    ) -> Result<(), KernelError> {
        if !definition.foreign || definition.body.is_some() {
            return Err(KernelError::new("invalid foreign executable declaration"));
        }
        self.validate_definition(definition, true, definition.unsafe_)?;
        self.foreign_contracts.insert(definition.name.clone());
        Ok(())
    }

    pub(super) fn validate_numeric_definition(
        &mut self,
        definition: &DefDecl,
    ) -> Result<(), KernelError> {
        if definition.unsafe_ || definition.foreign || definition.body.is_some() {
            return Err(KernelError::new("invalid numeric executable declaration"));
        }
        self.validate_definition(definition, false, false)?;
        self.numeric_contracts.insert(definition.name.clone());
        Ok(())
    }

    pub(super) fn validate_opaque_definition(
        &mut self,
        definition: &DefDecl,
    ) -> Result<(), KernelError> {
        if definition.unsafe_ || definition.foreign || definition.body.is_some() {
            return Err(KernelError::new("invalid opaque executable declaration"));
        }
        self.validate_definition(definition, false, false)?;
        self.opaque_contracts.insert(definition.name.clone());
        Ok(())
    }

    fn validate_definition(
        &mut self,
        definition: &DefDecl,
        foreign_completion: bool,
        unsafe_context: bool,
    ) -> Result<(), KernelError> {
        if self.adts.contains_key(&definition.name) {
            return Err(KernelError::new("definition collides with a datatype"));
        }
        let previous = self.defs.get(&definition.name).cloned();
        if definition.completion_parameters.is_some()
            && (previous.is_none() || definition.body.is_none() || foreign_completion)
        {
            return Err(KernelError::new("invalid ordinary law completion metadata"));
        }
        if let Some(previous) = previous {
            if previous.body.is_some()
                || self.foreign_contracts.contains(&definition.name)
                || self.numeric_contracts.contains(&definition.name)
                || self.opaque_contracts.contains(&definition.name)
                || definition.body.is_none() && !foreign_completion
            {
                return Err(KernelError::new("duplicate declaration"));
            }
            if !self.compare(Comparison::Equal, &previous.ty, &definition.ty)? {
                return Err(KernelError::new("proof does not preserve its law's type"));
            }
        }
        let mut pending = definition.clone();
        pending.body = None;
        Rc::make_mut(&mut self.defs).insert(definition.name.clone(), pending);
        let lhs = Lhs {
            name: definition.name.clone(),
            equation: term(Term::Ref(definition.name.clone())),
            remaining: 0,
            quants: vec![],
            unsafe_: unsafe_context,
        };
        self.check(
            &lhs,
            &definition.ty,
            Quant::None,
            &kind(Quant::Lone),
            &Context::new(),
        )?;
        let (checked, quants) = self.complete_parameters(definition)?;
        // Recursive checking sees the actual arity but never an unchecked body.
        let mut pending = checked.clone();
        pending.body = None;
        Rc::make_mut(&mut self.defs).insert(checked.name.clone(), pending);
        if let Some(body) = &checked.body {
            let lhs = Lhs {
                remaining: checked.parameters.len(),
                quants,
                ..lhs
            };
            self.check(&lhs, body, Quant::Lone, &checked.ty, &Context::new())?;
        }
        Rc::make_mut(&mut self.defs).insert(checked.name.clone(), checked);
        Ok(())
    }

    fn complete_parameters(
        &mut self,
        definition: &DefDecl,
    ) -> Result<(DefDecl, Vec<Quant>), KernelError> {
        let mut checked = definition.clone();
        let mut telescope = Rc::clone(&definition.ty);
        let mut quants = Vec::new();
        let names = definition.completion_parameters.clone().unwrap_or_else(|| {
            definition
                .parameters
                .iter()
                .map(|p| (p.name.clone(), p.id))
                .collect()
        });
        let mut parameters = Vec::with_capacity(names.len());
        let mut ids = BTreeSet::new();
        for (name, id) in names {
            if !ids.insert(id) {
                return Err(KernelError::new("duplicate declared parameter identifier"));
            }
            let t = self.whnf(&telescope)?;
            let Term::All {
                quant,
                id: old_id,
                domain,
                body,
                ..
            } = t.as_ref()
            else {
                return Err(KernelError::new(
                    "parameter list exceeds the function telescope",
                ));
            };
            quants.push(*quant);
            if definition.completion_parameters.is_some() {
                let variable = term(Term::Var {
                    name: name.clone(),
                    id,
                });
                parameters.push(Binder {
                    quant: *quant,
                    name,
                    id,
                    ty: Rc::clone(domain),
                });
                telescope = substitute(body, *old_id, &variable);
            } else {
                telescope = Rc::clone(body);
            }
        }
        if checked.completion_parameters.take().is_some() {
            checked.parameters = parameters;
        }
        Ok((checked, quants))
    }

    pub(super) fn validate_adt(&mut self, declaration: &AdtDecl) -> Result<(), KernelError> {
        if self.adts.contains_key(&declaration.name) || self.defs.contains_key(&declaration.name) {
            return Err(KernelError::new("duplicate datatype"));
        }
        for constructor in &declaration.constructors {
            if self.constructor_family(&constructor.name).is_some()
                || declaration
                    .constructors
                    .iter()
                    .filter(|c| c.name == constructor.name)
                    .count()
                    != 1
            {
                return Err(KernelError::new(format!(
                    "duplicate constructor {}",
                    constructor.name
                )));
            }
        }
        Rc::make_mut(&mut self.adts).insert(declaration.name.clone(), declaration.clone());
        let lhs = Lhs::default();
        self.check(
            &lhs,
            &arrows(&declaration.parameters, Rc::clone(&declaration.kind)),
            Quant::None,
            &kind(Quant::Lone),
            &Context::new(),
        )?;
        let declared_kind = self.whnf(&declaration.kind)?;
        if !matches!(declared_kind.as_ref(), Term::Typ(_)) {
            return Err(KernelError::new(
                "datatype declaration must end in Kind(g), Type, or Data",
            ));
        }
        for constructor in &declaration.constructors {
            let mut context = Context::new();
            for (i, binder) in declaration
                .parameters
                .iter()
                .chain(&constructor.fields)
                .enumerate()
            {
                let expected = if i >= declaration.parameters.len() && binder.quant == Quant::Lone {
                    Rc::clone(&declaration.kind)
                } else {
                    kind(binder.quant)
                };
                self.check(&lhs, &binder.ty, Quant::None, &expected, &context)?;
                if context.insert(binder.id, binder.clone()).is_some() {
                    return Err(KernelError::new("duplicate binder identifier"));
                }
            }
        }
        Ok(())
    }

    fn constructor_family(&self, name: &str) -> Option<String> {
        self.adts
            .values()
            .find(|a| a.constructors.iter().any(|c| c.name == name))
            .map(|a| a.name.clone())
    }

    fn adt(&self, value: &TermRef) -> Result<AdtDecl, KernelError> {
        let Term::Adt {
            name,
            args,
            excluded,
        } = value.as_ref()
        else {
            return Err(KernelError::new(format!(
                "expected a datatype; observed {value}"
            )));
        };
        let mut adt = self
            .adts
            .get(name)
            .cloned()
            .ok_or_else(|| KernelError::new(format!("undefined datatype {name}")))?;
        if args.len() != adt.parameters.len() {
            return Err(KernelError::new(format!(
                "{name} needs {} type parameters",
                adt.parameters.len()
            )));
        }
        if excluded
            .iter()
            .any(|n| !adt.constructors.iter().any(|c| &c.name == n))
        {
            return Err(KernelError::new("unknown excluded constructor"));
        }
        adt.constructors.retain(|c| !excluded.contains(&c.name));
        Ok(adt)
    }

    fn fields(adt: &AdtDecl, constructor: &ConstructorDecl, args: &[TermRef]) -> Vec<Binder> {
        constructor
            .fields
            .iter()
            .map(|field| {
                let mut result = field.clone();
                for (parameter, arg) in adt.parameters.iter().zip(args) {
                    result.ty = substitute(&result.ty, parameter.id, arg);
                }
                result
            })
            .collect()
    }

    fn infer(
        &mut self,
        lhs: &Lhs,
        value: &TermRef,
        demand: Quant,
        context: &Context,
        arguments: &[TermRef],
    ) -> Result<(TermRef, Uses), KernelError> {
        self.step()?;
        self.enter()?;
        let result = self.infer_inner(lhs, value, demand, context, arguments);
        self.depth -= 1;
        result
    }
    #[expect(
        clippy::too_many_lines,
        reason = "Keeping the bidirectional inference rules in one exhaustive match makes the trusted kernel auditable."
    )]
    fn infer_inner(
        &mut self,
        lhs: &Lhs,
        value: &TermRef,
        demand: Quant,
        context: &Context,
        arguments: &[TermRef],
    ) -> Result<(TermRef, Uses), KernelError> {
        match value.as_ref() {
            Term::Var { id, name } => {
                let Some(b) = context.get(*id) else {
                    let mut error = KernelError::new(format!("unbound variable {value}"));
                    if self
                        .origins
                        .as_ref()
                        .is_some_and(|origins| origins.is_reusable_marker(*id))
                    {
                        // Upstream's sentinel is -1. Its printer shows the
                        // suffix only when the name also occurs in scope.
                        let observed = if context.values().any(|binder| binder.name == *name) {
                            format!("{name}^-1")
                        } else {
                            name.clone()
                        };
                        error.diagnostic = self
                            .term_diagnostic(
                                lhs,
                                value,
                                DiagnosticExpression::Text("a bound variable".into()),
                                DiagnosticExpression::Text(observed),
                                context,
                            )
                            .ok()
                            .map(Box::new);
                    }
                    return Err(error);
                };
                Ok((Rc::clone(&b.ty), usage(*id, demand)))
            }
            Term::Ref(name) | Term::GpuRef(name) => {
                if let Some(adt) = self.adts.get(name) {
                    if !adt.parameters.is_empty() {
                        return Err(KernelError::new(
                            "a parameterized datatype needs <...> arguments",
                        ));
                    }
                    return Ok((Rc::clone(&adt.kind), Uses::new()));
                }
                let def = self
                    .defs
                    .get(name)
                    .cloned()
                    .ok_or_else(|| KernelError::new(format!("undefined name {name}")))?;
                if demand != Quant::None {
                    if name == &lhs.name && !lhs.unsafe_ {
                        let (_, columns) = spine(&lhs.equation);
                        let mut order = std::cmp::Ordering::Equal;
                        for ((arg, col), quant) in arguments.iter().zip(&columns).zip(&lhs.quants) {
                            if order != std::cmp::Ordering::Equal {
                                break;
                            }
                            order = self.descend(*quant, arg, col)?;
                        }
                        if order != std::cmp::Ordering::Less {
                            return Err(KernelError::new(
                                "recursive self-call must decrease structurally, left to right",
                            ));
                        }
                    }
                    if name != &lhs.name
                        && def.body.is_none()
                        && !self.foreign_contracts.contains(name)
                        && !self.numeric_contracts.contains(name)
                        && !self.opaque_contracts.contains(name)
                    {
                        return Err(KernelError::new(format!(
                            "unfilled law {name} cannot be used as live evidence"
                        )));
                    }
                }
                Ok((def.ty, Uses::new()))
            }
            Term::Typ(g) => {
                self.check(lhs, g, Quant::None, &term(Term::Qnt), context)?;
                Ok((kind(Quant::Lone), Uses::new()))
            }
            Term::Qnt => Ok((kind(Quant::Lone), Uses::new())),
            Term::Qua(_) => Ok((term(Term::Qnt), Uses::new())),
            Term::Min(a, b) => {
                let a = self.check(lhs, a, demand, &term(Term::Qnt), context)?;
                let b = self.check(lhs, b, demand, &term(Term::Qnt), context)?;
                Ok((term(Term::Qnt), add(a, b)))
            }
            Term::All {
                quant,
                name,
                id,
                domain,
                body,
            } => {
                self.check(lhs, domain, Quant::None, &lhs.domain_kind(*quant), context)?;
                let mut context = context.clone();
                if context
                    .insert(
                        *id,
                        Binder {
                            quant: *quant,
                            name: name.clone(),
                            id: *id,
                            ty: Rc::clone(domain),
                        },
                    )
                    .is_some()
                {
                    return Err(KernelError::new("duplicate binder identifier"));
                }
                self.check(lhs, body, Quant::None, &kind(Quant::Lone), &context)?;
                Ok((kind(Quant::Lone), Uses::new()))
            }
            Term::App(f, x) => {
                if let Term::Lam { id, body, .. } = f.as_ref() {
                    return self.infer(
                        lhs,
                        &self.substitute(body, *id, x),
                        demand,
                        context,
                        arguments,
                    );
                }
                let mut args = vec![Rc::clone(x)];
                args.extend(arguments.iter().map(Rc::clone));
                let (function_type, fu) = self.infer(lhs, f, demand, context, &args)?;
                let ft = self.whnf(&function_type)?;
                let Term::All {
                    quant,
                    id,
                    domain,
                    body,
                    ..
                } = ft.as_ref()
                else {
                    let mut error =
                        KernelError::new(format!("expected a function type; observed {ft}"));
                    error.diagnostic = self
                        .term_diagnostic(
                            lhs,
                            value,
                            DiagnosticExpression::Text("a function type".into()),
                            DiagnosticExpression::Term(&function_type),
                            context,
                        )
                        .ok()
                        .map(Box::new);
                    return Err(error);
                };
                let xu = self.check(lhs, x, quant.demand(demand), domain, context)?;
                Ok((self.substitute(body, *id, x), add(fu, xu)))
            }
            Term::Adt { name, args, .. } => {
                let adt = self.adt(value)?;
                let telescope = arrows(&adt.parameters, Rc::clone(&adt.kind));
                let (ty, uses) = self.check_arguments(lhs, &telescope, args, demand, context)?;
                if name != &adt.name {
                    return Err(KernelError::new("datatype name mismatch"));
                }
                Ok((ty, uses))
            }
            Term::Eql { left, right, ty } => {
                for (index, child, expected) in [
                    (2, ty, kind(Quant::Lone)),
                    (0, left, Rc::clone(ty)),
                    (1, right, Rc::clone(ty)),
                ] {
                    self.check(lhs, child, Quant::None, &expected, context)
                        .map_err(|error| self.child_diagnostic(error, value, index, child))?;
                }
                Ok((kind(Quant::Many), Uses::new()))
            }
            Term::Ann(x, ty) => {
                self.check(lhs, ty, Quant::None, &kind(Quant::Lone), context)?;
                let uses = self.check(lhs, x, demand, ty, context)?;
                Ok((Rc::clone(ty), uses))
            }
            Term::Ctr { name, .. } => {
                let mut error = KernelError::new(format!(
                    "term needs an expected type or annotation: {value}"
                ));
                let expected = if self.constructor_family(name).is_none() {
                    "a declared constructor"
                } else {
                    "an annotated term (cannot infer)"
                };
                error.diagnostic = self
                    .term_diagnostic(
                        lhs,
                        value,
                        DiagnosticExpression::Text(expected.into()),
                        DiagnosticExpression::Term(value),
                        context,
                    )
                    .ok()
                    .map(Box::new);
                Err(error)
            }
            Term::Hole(name) => {
                let mut error = KernelError::new(format!("unfinished proof hole ?{name}"));
                error.diagnostic = self
                    .term_diagnostic(
                        lhs,
                        value,
                        DiagnosticExpression::Text("an annotated term (cannot infer)".into()),
                        DiagnosticExpression::Term(value),
                        context,
                    )
                    .ok()
                    .map(Box::new);
                Err(error)
            }
            _ => {
                let mut error = KernelError::new(format!(
                    "term needs an expected type or annotation: {value}"
                ));
                error.diagnostic = self
                    .term_diagnostic(
                        lhs,
                        value,
                        DiagnosticExpression::Text("an annotated term (cannot infer)".into()),
                        DiagnosticExpression::Term(value),
                        context,
                    )
                    .ok()
                    .map(Box::new);
                Err(error)
            }
        }
    }

    fn check_arguments(
        &mut self,
        lhs: &Lhs,
        telescope: &TermRef,
        args: &[TermRef],
        demand: Quant,
        context: &Context,
    ) -> Result<(TermRef, Uses), KernelError> {
        let mut tel = Rc::clone(telescope);
        let mut uses = Uses::new();
        for arg in args {
            let t = self.whnf(&tel)?;
            let Term::All {
                quant,
                id,
                domain,
                body,
                ..
            } = t.as_ref()
            else {
                return Err(KernelError::new("too many arguments for telescope"));
            };
            uses = add(
                uses,
                self.check(lhs, arg, quant.demand(demand), domain, context)?,
            );
            tel = self.substitute(body, *id, arg);
        }
        Ok((tel, uses))
    }

    fn check(
        &mut self,
        lhs: &Lhs,
        value: &TermRef,
        demand: Quant,
        ty: &TermRef,
        context: &Context,
    ) -> Result<Uses, KernelError> {
        self.step()?;
        self.enter()?;
        let result = self.check_inner(lhs, value, demand, ty, context);
        self.depth -= 1;
        result
    }
    #[expect(
        clippy::too_many_lines,
        reason = "Keeping the bidirectional checking rules in one exhaustive match makes the trusted kernel auditable."
    )]
    fn check_inner(
        &mut self,
        lhs: &Lhs,
        value: &TermRef,
        demand: Quant,
        ty: &TermRef,
        context: &Context,
    ) -> Result<Uses, KernelError> {
        match value.as_ref() {
            Term::Hole(name) => {
                let mut error = KernelError::new(format!("unfinished proof hole ?{name}"));
                // TODO remains an incomplete-source refusal. Upstream admits
                // it provisionally; this checker never issues that certificate.
                if name != "TODO" {
                    error.diagnostic = self
                        .term_diagnostic(
                            lhs,
                            value,
                            DiagnosticExpression::Term(ty),
                            DiagnosticExpression::Term(value),
                            context,
                        )
                        .ok()
                        .map(Box::new);
                }
                return Err(error);
            }
            Term::Lam { name, id, body } => {
                let t = self.whnf(ty)?;
                let Term::All {
                    quant,
                    id: tid,
                    domain,
                    body: codomain,
                    ..
                } = t.as_ref()
                else {
                    return Err(mismatch(ty, value));
                };
                let x = self.fresh(name)?;
                let Term::Var { id: xid, .. } = x.as_ref() else {
                    unreachable!()
                };
                let binder = Binder {
                    quant: *quant,
                    name: name.clone(),
                    id: *xid,
                    ty: Rc::clone(domain),
                };
                let mut next = context.clone();
                next.insert(*xid, binder.clone());
                let mut lhs = lhs.clone();
                if lhs.remaining > 0 {
                    lhs.equation = apply(lhs.equation, [Rc::clone(&x)]);
                    lhs.remaining -= 1;
                }
                let checked_body = self.substitute(body, *id, &x);
                let mut uses = self
                    .check(
                        &lhs,
                        &checked_body,
                        demand,
                        &self.substitute(codomain, *tid, &x),
                        &next,
                    )
                    .map_err(|error| self.child_diagnostic(error, value, 0, &checked_body))?;
                self.check_usage(&lhs, value, &binder, &mut uses, context)?;
                return Ok(uses);
            }
            Term::Let { bindings, body } => {
                let mut next = context.clone();
                let mut uses = Uses::new();
                let mut binders = Vec::new();
                for b in bindings {
                    if next.contains_key(b.id) {
                        return Err(KernelError::new("duplicate let binder identifier"));
                    }
                    let (t, u) = self.infer(lhs, &b.value, b.quant.demand(demand), context, &[])?;
                    self.check(lhs, &t, Quant::None, &lhs.domain_kind(b.quant), context)?;
                    let binder = Binder {
                        quant: b.quant,
                        name: b.name.clone(),
                        id: b.id,
                        ty: t,
                    };
                    next.insert(b.id, binder.clone());
                    binders.push(binder);
                    uses = add(uses, u);
                }
                for b in bindings {
                    self.aliases.insert(b.id, Rc::clone(&b.value));
                }
                let result = self
                    .check(lhs, body, demand, ty, &next)
                    .map_err(|error| self.child_diagnostic(error, value, 0, body));
                for b in bindings {
                    self.aliases.remove(&b.id);
                }
                let mut body_uses = result?;
                for b in &binders {
                    self.check_usage(lhs, value, b, &mut body_uses, context)?;
                }
                return Ok(add(uses, body_uses));
            }
            Term::Ctr { name, args } => {
                let t = self.whnf(ty)?;
                let adt = self.adt(&t).map_err(|mut error| {
                    if !matches!(t.as_ref(), Term::Adt { .. }) {
                        let family = self
                            .constructor_family(name)
                            .map(|name| term(Term::Ref(name)));
                        error.diagnostic = self
                            .term_diagnostic(
                                lhs,
                                value,
                                DiagnosticExpression::Term(ty),
                                family.as_ref().map_or(
                                    DiagnosticExpression::Typeless(value),
                                    DiagnosticExpression::Term,
                                ),
                                context,
                            )
                            .ok()
                            .map(Box::new);
                    }
                    error
                })?;
                let Some(constructor) = adt.constructors.iter().find(|c| &c.name == name) else {
                    let mut error = KernelError::new(format!(
                        "{name} is not a remaining constructor of {}",
                        adt.name
                    ));
                    let family = self
                        .constructor_family(name)
                        .map(|name| term(Term::Ref(name)));
                    let (expected, observed) = if let Some(family) = &family {
                        (
                            DiagnosticExpression::Term(ty),
                            DiagnosticExpression::Term(family),
                        )
                    } else {
                        let constructors = adt
                            .constructors
                            .iter()
                            .map(|c| c.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        (
                            DiagnosticExpression::Text(format!(
                                "a declared constructor ({} declares {constructors})",
                                adt.name
                            )),
                            DiagnosticExpression::Term(value),
                        )
                    };
                    error.diagnostic = self
                        .term_diagnostic(lhs, value, expected, observed, context)
                        .ok()
                        .map(Box::new);
                    return Err(error);
                };
                if constructor.fields.len() != args.len() {
                    let mut error = KernelError::new(format!(
                        "{name} needs {} fields; got {}",
                        constructor.fields.len(),
                        args.len()
                    ));
                    let count = constructor.fields.len();
                    let plural = if count == 1 { "field" } else { "fields" };
                    error.diagnostic = self
                        .term_diagnostic(
                            lhs,
                            value,
                            DiagnosticExpression::Text(format!("{name} with {count} {plural}")),
                            DiagnosticExpression::Term(value),
                            context,
                        )
                        .ok()
                        .map(Box::new);
                    return Err(error);
                }
                let Term::Adt {
                    args: parameters, ..
                } = t.as_ref()
                else {
                    unreachable!()
                };
                let telescope = arrows(&Self::fields(&adt, constructor, parameters), Rc::clone(&t));
                return self
                    .check_arguments(lhs, &telescope, args, demand, context)
                    .map(|(_, u)| u);
            }
            Term::Mat { .. } | Term::Efq => {
                return self.check_match(lhs, value, demand, ty, context);
            }
            Term::Rfl => {
                let t = self.whnf(ty)?;
                let Term::Eql { left, right, .. } = t.as_ref() else {
                    return Err(mismatch(ty, value));
                };
                if !self.compare(Comparison::Equal, left, right)? {
                    let mut error = KernelError::new(format!(
                        "reflexivity requires equal endpoints: {left} != {right}"
                    ));
                    error.diagnostic = self
                        .reflexivity_diagnostic(lhs, value, left, right, context)
                        .ok()
                        .map(Box::new);
                    return Err(error);
                }
                return Ok(Uses::new());
            }
            Term::Rwt {
                evidence,
                motive,
                body,
            } => {
                let (evidence_type, eu) = self.infer(lhs, evidence, demand, context, &[])?;
                let et = self.whnf(&evidence_type)?;
                let Term::Eql {
                    left,
                    right,
                    ty: domain,
                } = et.as_ref()
                else {
                    let mut error = KernelError::new("rewrite evidence must have equality type");
                    error.diagnostic = self
                        .term_diagnostic(
                            lhs,
                            evidence,
                            DiagnosticExpression::Text("an equation {a == b : T}".into()),
                            DiagnosticExpression::Term(&evidence_type),
                            context,
                        )
                        .ok()
                        .map(Box::new);
                    return Err(self.child_diagnostic(error, value, 0, evidence));
                };
                let x = self.fresh("_")?;
                let e = self.fresh("equation")?;
                let Term::Var { id: xi, .. } = x.as_ref() else {
                    unreachable!()
                };
                let Term::Var { id: ei, .. } = e.as_ref() else {
                    unreachable!()
                };
                let mt = arrows(
                    &[
                        Binder {
                            quant: Quant::Lone,
                            name: "_".into(),
                            id: *xi,
                            ty: Rc::clone(domain),
                        },
                        Binder {
                            quant: Quant::Lone,
                            name: "equation".into(),
                            id: *ei,
                            ty: term(Term::Eql {
                                left: Rc::clone(left),
                                right: x,
                                ty: Rc::clone(domain),
                            }),
                        },
                    ],
                    kind(Quant::Lone),
                );
                self.check(lhs, motive, Quant::None, &mt, context)?;
                let goal = apply(Rc::clone(motive), [Rc::clone(right), Rc::clone(evidence)]);
                if !self.compare(Comparison::Fits, &goal, ty)? {
                    let mut error = mismatch(ty, &goal);
                    error.diagnostic = self
                        .term_diagnostic(
                            lhs,
                            value,
                            DiagnosticExpression::Term(ty),
                            DiagnosticExpression::Term(&goal),
                            context,
                        )
                        .ok()
                        .map(Box::new);
                    return Err(error);
                }
                let premise = apply(Rc::clone(motive), [Rc::clone(left), term(Term::Rfl)]);
                let bu = self.check(lhs, body, demand, &premise, context)?;
                return Ok(add(eu, bu));
            }
            _ => (),
        }
        // Switching from checking to inference does not descend into a child:
        // infer this same term under check's active depth frame. Keep the work
        // charge; infer_inner still guards every recursive child through infer
        // or check, and normalization retains its own nesting checks.
        self.step()?;
        let (observed, uses) = self.infer_inner(lhs, value, demand, context, &[])?;
        if self.compare(Comparison::Fits, &observed, ty)? {
            Ok(uses)
        } else {
            let mut error = mismatch(ty, &observed);
            error.diagnostic = self
                .term_diagnostic(
                    lhs,
                    value,
                    DiagnosticExpression::Term(ty),
                    DiagnosticExpression::Term(&observed),
                    context,
                )
                .ok()
                .map(Box::new);
            Err(error)
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "The match rule constructs arm and residual goals together to keep quantity and termination transitions visible."
    )]
    fn check_match(
        &mut self,
        lhs: &Lhs,
        value: &TermRef,
        demand: Quant,
        ty: &TermRef,
        context: &Context,
    ) -> Result<Uses, KernelError> {
        let t = self.whnf(ty)?;
        let Term::All {
            quant,
            name,
            id,
            domain,
            body,
        } = t.as_ref()
        else {
            let mut error = mismatch(ty, value);
            error.diagnostic = self
                .term_diagnostic(
                    lhs,
                    value,
                    DiagnosticExpression::Term(ty),
                    DiagnosticExpression::Typeless(value),
                    context,
                )
                .ok()
                .map(Box::new);
            return Err(error);
        };
        if demand != Quant::None && *quant == Quant::None {
            let mut error = KernelError::new("an erased scrutinee cannot determine live evidence");
            error.diagnostic = self
                .diagnostic(
                    lhs,
                    value,
                    DiagnosticExpression::Text(
                        "a live scrutinee (a - scrutinee matches only in a dead region)".into(),
                    ),
                    None,
                    context,
                )
                .ok()
                .map(Box::new);
            return Err(error);
        }
        let dt = self.whnf(domain)?;
        let adt = self.adt(&dt).map_err(|mut error| {
            if !matches!(dt.as_ref(), Term::Adt { .. }) {
                error.diagnostic = self
                    .term_diagnostic(
                        lhs,
                        value,
                        DiagnosticExpression::Text("a datatype".into()),
                        DiagnosticExpression::Term(domain),
                        context,
                    )
                    .ok()
                    .map(Box::new);
            }
            error
        })?;
        if matches!(value.as_ref(), Term::Efq) {
            if adt.constructors.is_empty() || self.context_dead(context)? {
                return Ok(Uses::new());
            }
            let names = adt
                .constructors
                .iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let mut error = KernelError::new(format!("missing cases for {names}"));
            error.diagnostic = self
                .term_diagnostic(
                    lhs,
                    value,
                    DiagnosticExpression::Text(format!("cases for {names}")),
                    DiagnosticExpression::Term(value),
                    context,
                )
                .ok()
                .map(Box::new);
            return Err(error);
        }
        let Term::Mat {
            constructor,
            arm,
            fallback,
        } = value.as_ref()
        else {
            unreachable!()
        };
        let ctr = adt
            .constructors
            .iter()
            .find(|c| &c.name == constructor)
            .ok_or_else(|| {
                let mut error = KernelError::new(format!(
                    "constructor {constructor} missing or already matched"
                ));
                error.diagnostic = self
                    .term_diagnostic(
                        lhs,
                        value,
                        DiagnosticExpression::Text(format!(
                            "a constructor of {} (missing, or already matched)",
                            adt.name
                        )),
                        DiagnosticExpression::Term(value),
                        context,
                    )
                    .ok()
                    .map(|mut report| {
                        if let Some(source) = self
                            .origins
                            .as_ref()
                            .and_then(|origins| origins.constructor_source(value))
                        {
                            report.term_identity = source;
                            report.match_site = None;
                        }
                        Box::new(report)
                    });
                error
            })?;
        let Term::Adt { args, excluded, .. } = dt.as_ref() else {
            unreachable!()
        };
        let fields = Self::fields(&adt, ctr, args);
        let mut fresh_fields = Vec::new();
        let mut substitutions = Vec::new();
        let mut field_values = Vec::new();
        for field in fields {
            let x = self.fresh(&field.name)?;
            let Term::Var { id: xid, .. } = x.as_ref() else {
                unreachable!()
            };
            let mut domain = field.ty;
            for (old, replacement) in &substitutions {
                domain = self.substitute(&domain, *old, replacement);
            }
            fresh_fields.push(Binder {
                quant: field.quant.product(*quant),
                name: field.name,
                id: *xid,
                ty: domain,
            });
            substitutions.push((field.id, Rc::clone(&x)));
            field_values.push(x);
        }
        let constructor_value = term(Term::Ctr {
            name: constructor.clone(),
            args: field_values,
        });
        let arm_goal = arrows(
            &fresh_fields,
            self.substitute(body, *id, &constructor_value),
        );
        let mut arm_lhs = lhs.clone();
        if lhs.remaining > 0 {
            arm_lhs.equation = lambdas(
                &fresh_fields,
                apply(Rc::clone(&lhs.equation), [constructor_value]),
            );
            arm_lhs.remaining = lhs.remaining - 1 + fresh_fields.len();
        }
        let arm_uses = self
            .check(&arm_lhs, arm, demand, &arm_goal, context)
            .map_err(|error| self.child_diagnostic(error, value, 0, arm))?;
        let mut excluded = excluded.clone();
        excluded.push(constructor.clone());
        let residual = term(Term::All {
            quant: *quant,
            name: name.clone(),
            id: *id,
            domain: term(Term::Adt {
                name: adt.name,
                args: args.iter().map(Rc::clone).collect(),
                excluded,
            }),
            body: Rc::clone(body),
        });
        let fallback_uses = self
            .check(lhs, fallback, demand, &residual, context)
            .map_err(|error| self.child_diagnostic(error, value, 1, fallback))?;
        Ok(join(arm_uses, fallback_uses))
    }

    fn context_dead(&mut self, context: &Context) -> Result<bool, KernelError> {
        for b in context.values() {
            if b.quant != Quant::None {
                let t = self.whnf(&b.ty)?;
                if matches!(t.as_ref(), Term::Adt { .. }) && self.adt(&t)?.constructors.is_empty() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

impl Engine {
    // Upstream term_strip follows only annotations and the original RHS of a
    // let-bound Var. Do not unfold functions or normalize computed arguments:
    // structural descent must remain visible after these transparent aliases.
    fn strip_descent_aliases(&mut self, value: &TermRef) -> Result<TermRef, KernelError> {
        let mut value = Rc::clone(value);
        loop {
            self.step()?;
            value = match value.as_ref() {
                Term::Ann(inner, _) => Rc::clone(inner),
                Term::Var { id, .. } => match self.aliases.get(id) {
                    Some(inner) => Rc::clone(inner),
                    None => return Ok(value),
                },
                _ => return Ok(value),
            };
        }
    }

    fn descend(
        &mut self,
        quant: Quant,
        arg: &TermRef,
        column: &TermRef,
    ) -> Result<std::cmp::Ordering, KernelError> {
        self.step()?;
        self.enter()?;
        let result = self.descend_inner(quant, arg, column);
        self.depth -= 1;
        result
    }

    fn descend_inner(
        &mut self,
        quant: Quant,
        arg: &TermRef,
        column: &TermRef,
    ) -> Result<std::cmp::Ordering, KernelError> {
        use std::cmp::Ordering::Equal;
        use std::cmp::Ordering::Greater;
        use std::cmp::Ordering::Less;
        if quant == Quant::None {
            return Ok(Equal);
        }
        let a = self.strip_descent_aliases(arg)?;
        let p = self.strip_descent_aliases(column)?;
        Ok(match p.as_ref() {
            Term::Var { id: pi, .. } => {
                if matches!(a.as_ref(),Term::Var{id:ai,..}if ai==pi) {
                    Equal
                } else {
                    Greater
                }
            }
            Term::Ctr { name: pn, args: px } => {
                if let Term::Ctr { name: an, args: ax } = a.as_ref()
                    && an == pn
                    && ax.len() == px.len()
                {
                    let mut order = Equal;
                    for (x, y) in ax.iter().zip(px) {
                        let field = self.descend(Quant::Lone, x, y)?;
                        if field != Equal {
                            order = field;
                        }
                        if order == Greater {
                            break;
                        }
                    }
                    if order != Greater {
                        return Ok(order);
                    }
                }
                for field in px {
                    if self.descend(Quant::Lone, &a, field)? != Greater {
                        return Ok(Less);
                    }
                }
                Greater
            }
            _ => Greater,
        })
    }
}

#[cfg(test)]
mod depth_tests {
    use super::*;

    fn named_pattern_engine() -> Engine {
        let book = crate::syntax::parse(include_str!(
            "../../tests/fixtures/cpu-blocked-named-calls/second_field.bend"
        ))
        .unwrap();
        check_book(&book).unwrap().engine
    }

    fn named_box_call(left: TermRef, right: TermRef) -> TermRef {
        term(Term::App(
            term(Term::Ref("f".into())),
            term(Term::Ctr {
                name: "Box".into(),
                args: vec![left, right],
            }),
        ))
    }

    #[test]
    fn named_patterns_keep_shared_stuck_calls_and_do_not_force_later_fields() {
        let mut engine = named_pattern_engine();
        let cycle = term(Term::Var {
            name: "cycle".into(),
            id: 10_000,
        });
        engine.aliases.insert(10_000, Rc::clone(&cycle));
        let free = term(Term::Var {
            name: "free".into(),
            id: 10_001,
        });
        let call = named_box_call(Rc::clone(&free), Rc::clone(&cycle));
        let before = format!("{call:?}");
        let owners = Rc::strong_count(&cycle);
        engine.fuel = 512;
        for _ in 0..2 {
            assert!(Rc::ptr_eq(&engine.whnf(&call).unwrap(), &call));
            assert_eq!(engine.depth, 0);
        }
        assert!(engine.fuel < 512);
        assert_eq!(format!("{call:?}"), before);
        assert_eq!(Rc::strong_count(&cycle), owners);
        assert!(Rc::ptr_eq(&engine.aliases[&10_000], &cycle));
        let zero = term(Term::Ctr {
            name: "Z".into(),
            args: vec![],
        });
        let successor = term(Term::Ctr {
            name: "S".into(),
            args: vec![Rc::clone(&zero)],
        });
        let selected = engine.whnf(&named_box_call(successor, cycle)).unwrap();
        assert!(Rc::ptr_eq(&selected, &zero));
    }

    #[test]
    fn named_pattern_work_and_depth_limits_fail_closed_without_resetting_state() {
        let engine = named_pattern_engine();
        let zero = term(Term::Ctr {
            name: "Z".into(),
            args: vec![],
        });
        let call = named_box_call(Rc::clone(&zero), zero);
        let mut measured = engine.clone();
        measured.fuel = 512;
        assert_eq!(measured.whnf(&call).unwrap().to_string(), "Z{}");
        let required = 512 - measured.fuel;
        assert!(required > 1 && required < 512);
        for fuel in [0, required - 1, required] {
            let mut bounded = engine.clone();
            bounded.fuel = fuel;
            let result = bounded.whnf(&call);
            if fuel == required {
                assert_eq!(result.unwrap().to_string(), "Z{}");
            } else {
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains("evaluation budget exhausted")
                );
            }
            assert_eq!(bounded.fuel, 0);
            assert_eq!(bounded.depth, 0);
        }
        let mut nested = engine;
        nested.depth = 127;
        assert!(
            nested
                .whnf(&call)
                .unwrap_err()
                .to_string()
                .contains("kernel nesting limit exhausted")
        );
        assert_eq!(nested.depth, 127);
    }

    #[test]
    fn named_pattern_fields_preserve_first_demanded_failure() {
        let cycle = term(Term::Var {
            name: "cycle".into(),
            id: 10_000,
        });
        let leaf = term(Term::Qua(Quant::Lone));
        let deep = (0..128).fold(Rc::clone(&leaf), |left, _| {
            term(Term::Min(left, Rc::clone(&leaf)))
        });
        for (first, second, expected) in [
            (
                Rc::clone(&cycle),
                Rc::clone(&deep),
                "evaluation budget exhausted",
            ),
            (deep, Rc::clone(&cycle), "kernel nesting limit exhausted"),
        ] {
            let mut engine = named_pattern_engine();
            engine.aliases.insert(10_000, Rc::clone(&cycle));
            engine.fuel = 512;
            let error = engine.whnf(&named_box_call(first, second)).unwrap_err();
            assert!(error.to_string().contains(expected), "{error}");
            assert_eq!(engine.depth, 0);
        }
    }

    #[test]
    fn check_to_infer_keeps_work_charge_and_unwinds_after_exhaustion() {
        let mut engine = empty_engine(&Book {
            declarations: vec![],
        })
        .unwrap();
        let ty = term(Term::Qnt);
        let value = term(Term::Var {
            name: "q".into(),
            id: 1,
        });
        let context = Context::from([(
            1,
            Binder {
                quant: Quant::Lone,
                name: "q".into(),
                id: 1,
                ty: Rc::clone(&ty),
            },
        )]);
        engine.fuel = 2;
        let error = engine
            .check(&Lhs::default(), &value, Quant::Lone, &ty, &context)
            .expect_err("checking, inference and comparison each require work");
        assert!(
            error.to_string().contains("evaluation budget exhausted"),
            "{error}"
        );
        assert_eq!(engine.depth, 0);
        engine.reset();
        assert_eq!(
            engine
                .check(&Lhs::default(), &value, Quant::Lone, &ty, &context)
                .unwrap(),
            usage(1, Quant::Lone)
        );
    }

    #[test]
    fn beta_expansion_in_inference_still_obeys_nesting_guard() {
        let mut engine = empty_engine(&Book {
            declarations: vec![],
        })
        .unwrap();
        let x = term(Term::Var {
            name: "x".into(),
            id: 1,
        });
        let lambda = term(Term::Lam {
            name: "x".into(),
            id: 1,
            body: term(Term::App(Rc::clone(&x), x)),
        });
        let looping = term(Term::App(Rc::clone(&lambda), lambda));
        let error = engine
            .check(
                &Lhs::default(),
                &looping,
                Quant::Lone,
                &term(Term::Qnt),
                &Context::new(),
            )
            .expect_err("same-node delegation cannot bypass recursive inference's guard");
        assert!(
            error.to_string().contains("kernel nesting limit exhausted"),
            "{error}"
        );
        assert_eq!(engine.depth, 0);
    }

    #[test]
    fn tail_reduction_charges_each_step_and_unwinds_after_budget_exhaustion() {
        let mut engine = empty_engine(&Book {
            declarations: vec![],
        })
        .unwrap();
        let leaf = term(Term::Qnt);
        let chain = (0..200).fold(Rc::clone(&leaf), |body, _| {
            term(Term::Ann(body, Rc::clone(&leaf)))
        });
        engine.fuel = 201;
        assert!(Rc::ptr_eq(&engine.whnf(&chain).unwrap(), &leaf));
        assert_eq!(engine.fuel, 0);
        assert_eq!(engine.depth, 0);

        engine.fuel = 200;
        let error = engine.whnf(&chain).unwrap_err();
        assert!(error.to_string().contains("evaluation budget exhausted"));
        assert_eq!(engine.depth, 0);

        // A cyclic alias needs no pending stack, but cannot evade the work bound.
        let cycle = term(Term::Var {
            name: "cycle".into(),
            id: 1,
        });
        engine.aliases.insert(1, Rc::clone(&cycle));
        engine.fuel = 512;
        let error = engine.whnf(&cycle).unwrap_err();
        assert!(error.to_string().contains("evaluation budget exhausted"));
        assert_eq!(engine.depth, 0);
        engine.reset();
        assert!(Rc::ptr_eq(&engine.whnf(&leaf).unwrap(), &leaf));
    }

    #[test]
    fn pending_reductions_keep_the_depth_guard_and_first_error_order() {
        let mut engine = empty_engine(&Book {
            declarations: vec![],
        })
        .unwrap();
        let leaf = term(Term::Qua(Quant::Lone));
        let deep = (0..128).fold(Rc::clone(&leaf), |left, _| {
            term(Term::Min(left, Rc::clone(&leaf)))
        });
        let cycle = term(Term::Var {
            name: "cycle".into(),
            id: 1,
        });
        for (left, right, diagnostic) in [
            (
                Rc::clone(&cycle),
                Rc::clone(&deep),
                "evaluation budget exhausted",
            ),
            (
                Rc::clone(&deep),
                Rc::clone(&cycle),
                "kernel nesting limit exhausted",
            ),
        ] {
            engine.reset();
            engine.aliases.insert(1, Rc::clone(&cycle));
            engine.fuel = 512;
            let error = engine.whnf(&term(Term::Min(left, right))).unwrap_err();
            assert!(error.to_string().contains(diagnostic), "{error}");
            assert_eq!(engine.depth, 0);
        }
        engine.reset();
        assert!(Rc::ptr_eq(&engine.whnf(&leaf).unwrap(), &leaf));
    }

    #[test]
    fn tail_reduction_preserves_kernel_demand_boundaries() {
        let mut engine = empty_engine(&Book {
            declarations: vec![],
        })
        .unwrap();
        let cycle = term(Term::Var {
            name: "cycle".into(),
            id: 1,
        });
        engine.aliases.insert(1, Rc::clone(&cycle));
        let leaf = term(Term::Qua(Quant::Lone));
        let ignored = term(Term::Lam {
            name: "ignored".into(),
            id: 2,
            body: Rc::clone(&leaf),
        });
        let selected = term(Term::Mat {
            constructor: "Chosen".into(),
            arm: Rc::clone(&leaf),
            fallback: Rc::clone(&cycle),
        });
        let constructor = term(Term::Ctr {
            name: "Chosen".into(),
            args: vec![],
        });
        for value in [
            term(Term::App(ignored, Rc::clone(&cycle))),
            term(Term::App(selected, constructor)),
            term(Term::Ann(Rc::clone(&leaf), Rc::clone(&cycle))),
        ] {
            assert!(Rc::ptr_eq(&engine.whnf(&value).unwrap(), &leaf));
            assert_eq!(engine.depth, 0);
        }
        let delayed = term(Term::Ctr {
            name: "Field".into(),
            args: vec![Rc::clone(&cycle)],
        });
        assert!(Rc::ptr_eq(&engine.whnf(&delayed).unwrap(), &delayed));
        let stuck = term(Term::Rwt {
            evidence: term(Term::Var {
                name: "evidence".into(),
                id: 3,
            }),
            motive: Rc::clone(&cycle),
            body: Rc::clone(&cycle),
        });
        assert!(Rc::ptr_eq(&engine.whnf(&stuck).unwrap(), &stuck));
        engine.fuel = 512;
        let error = engine.normalize(&delayed).unwrap_err();
        assert!(error.to_string().contains("evaluation budget exhausted"));
        assert_eq!(engine.depth, 0);
    }
}

#[cfg(test)]
mod reflexivity_diagnostic_tests {
    use super::*;

    #[test]
    fn match_guard_diagnostics_preserve_first_errors_and_remaining_budget() {
        let book = crate::syntax::parse("type N is Data:\n  Z{}\n").unwrap();
        let mut engine = empty_engine(&book).unwrap();
        let Declaration::Adt(adt) = &book.declarations[0] else {
            panic!("datatype")
        };
        engine.validate_adt(adt).unwrap();
        let context = Context::from([(
            77,
            Binder {
                quant: Quant::None,
                name: "q".into(),
                id: 77,
                ty: term(Term::Qnt),
            },
        )]);
        for (goal, message) in [
            ("Type", "expected Type"),
            ("@-n:N -> N", "an erased scrutinee"),
            ("(Type -> Type) -> N", "expected a datatype"),
            ("N -> N", "constructor Wrong missing"),
        ] {
            let goal = crate::syntax::parse_term(goal).unwrap();
            let value = term(Term::Mat {
                constructor: "Wrong".into(),
                arm: term(Term::Ctr {
                    name: "Z".into(),
                    args: vec![],
                }),
                fallback: term(Term::Efq),
            });
            engine.reset();
            let before = engine.fuel;
            let error = engine
                .check(&Lhs::default(), &value, Quant::Lone, &goal, &context)
                .unwrap_err();
            assert!(error.message.contains(message), "{error}");
            assert!(error.checker_diagnostic().is_some());
            let spent = before - engine.fuel;
            engine.fuel = spent;
            let exhausted = engine
                .check(&Lhs::default(), &value, Quant::Lone, &goal, &context)
                .unwrap_err();
            assert_eq!(error, exhausted);
            assert!(exhausted.checker_diagnostic().is_none());
            assert!(exhausted.language_failure().is_none());
            assert_eq!((engine.fuel, engine.depth), (0, 0));
            // Direct rule control only: whole-book source-hole refusal remains
            // earlier. Neither child can preempt this match guard.
            let unused = term(Term::Hole("unused".into()));
            let value = term(Term::Mat {
                constructor: "Wrong".into(),
                arm: Rc::clone(&unused),
                fallback: unused,
            });
            engine.reset();
            let first = engine
                .check(&Lhs::default(), &value, Quant::Lone, &goal, &context)
                .unwrap_err();
            assert!(first.message.contains(message));
            assert!(!first.message.contains("unfinished proof hole"));
        }
    }

    #[test]
    fn diagnostic_context_follows_admission_without_changing_semantic_iteration() {
        let mut context = Context::new();
        for (id, name) in [(100, "field"), (2, "local")] {
            context.insert(
                id,
                Binder {
                    quant: Quant::Lone,
                    name: name.into(),
                    id,
                    ty: term(Term::Qnt),
                },
            );
        }
        let mut replacement = context.get(100).unwrap().clone();
        replacement.name = "renamed".into();
        context.insert(100, replacement);
        assert_eq!(context.values().map(|b| b.id).collect::<Vec<_>>(), [2, 100]);
        let engine = empty_engine(&Book {
            declarations: vec![],
        })
        .unwrap();
        let report = engine
            .term_diagnostic(
                &Lhs::default(),
                &term(Term::Efq),
                DiagnosticExpression::Text("test".into()),
                DiagnosticExpression::Term(&term(Term::Efq)),
                &context,
            )
            .unwrap();
        assert_eq!(
            report.context,
            [
                ("renamed".into(), "Quant".into()),
                ("local".into(), "Quant".into())
            ]
        );
    }

    #[test]
    fn diagnostic_scope_preserves_shadowed_ids_and_context_type_prefixes() {
        let outer = term(Term::Var {
            name: "x".into(),
            id: 100,
        });
        let global = term(Term::Ref("x".into()));
        let pair = term(Term::Ctr {
            name: "Pair".into(),
            args: vec![Rc::clone(&outer), Rc::clone(&global)],
        });
        let mut context = Context::new();
        for (id, name, ty) in [
            (100, "x", Rc::clone(&global)),
            (2, "x", Rc::clone(&outer)),
            (300, "p", pair),
        ] {
            context.insert(
                id,
                Binder {
                    quant: Quant::Lone,
                    name: name.into(),
                    id,
                    ty,
                },
            );
        }
        let mut engine = empty_engine(&Book {
            declarations: vec![],
        })
        .unwrap();
        let proof = term(Term::Rfl);
        let observed = term(Term::App(global, Rc::clone(&outer)));
        let before = format!("{context:?} {observed:?}");
        let state = (engine.fuel, engine.depth, engine.fresh_id);
        let owners = Rc::strong_count(&outer);
        let report = engine
            .term_diagnostic(
                &Lhs::default(),
                &proof,
                DiagnosticExpression::Term(&outer),
                DiagnosticExpression::Typeless(&observed),
                &context,
            )
            .unwrap();
        assert_eq!(report.expected, "x^0");
        assert_eq!(
            report.observed.as_deref(),
            Some("non-inferrable term 'x^(x^0)'")
        );
        assert_eq!(
            report.context,
            [
                ("x".into(), "x".into()),
                ("x".into(), "x".into()),
                ("p".into(), "Pair{x^0, x^}".into()),
            ]
        );
        assert_eq!(report.term_identity, Rc::as_ptr(&proof) as usize);
        assert_eq!(format!("{context:?} {observed:?}"), before);
        assert_eq!((engine.fuel, engine.depth, engine.fresh_id), state);
        assert_eq!(Rc::strong_count(&outer), owners);
        engine.fuel = 0;
        assert!(
            engine
                .reflexivity_diagnostic(&Lhs::default(), &proof, &outer, &outer, &context)
                .is_err()
        );
        assert_eq!((engine.fuel, engine.depth), (0, 0));
    }

    #[test]
    fn inference_and_missing_cases_keep_original_errors_when_presentation_exhausts() {
        let book = crate::syntax::parse("type N is Data:\n  Z{}\n").unwrap();
        let mut engine = empty_engine(&book).unwrap();
        let Declaration::Adt(adt) = &book.declarations[0] else {
            panic!("datatype")
        };
        engine.validate_adt(adt).unwrap();
        let value = term(Term::Efq);
        let goal = crate::syntax::parse_term("N -> N").unwrap();
        for missing_case in [false, true] {
            let evaluate = |engine: &mut Engine| {
                if missing_case {
                    engine.check(&Lhs::default(), &value, Quant::Lone, &goal, &Context::new())
                } else {
                    engine
                        .infer(&Lhs::default(), &value, Quant::Lone, &Context::new(), &[])
                        .map(|(_, uses)| uses)
                }
            };
            engine.reset();
            let initial = engine.fuel;
            let error = evaluate(&mut engine).unwrap_err();
            assert!(error.checker_diagnostic().is_some());
            let spent = initial - engine.fuel;
            engine.fuel = spent;
            let exhausted = evaluate(&mut engine).unwrap_err();
            assert_eq!(error, exhausted);
            assert!(exhausted.checker_diagnostic().is_none());
            assert!(exhausted.language_failure().is_none());
            assert_eq!((engine.fuel, engine.depth), (0, 0));
        }
    }

    #[test]
    fn quantity_diagnostic_origins_preserve_terms_work_and_rejection() {
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-data-grades");
        let original = std::fs::read_to_string(root.join("accept_data.bend")).unwrap();
        let temporary = std::env::temp_dir().join(format!(
            "teamy-bend-quantity-origins-{}.bend",
            std::process::id()
        ));
        let mut sources = vec![original.clone()];
        for entry in std::fs::read_dir(root.join("negative")).unwrap() {
            let path = entry.unwrap().path();
            if path
                .extension()
                .is_some_and(|extension| extension == "bend")
            {
                let fragment = std::fs::read_to_string(path).unwrap();
                sources.push(format!(
                    "{original}\n{}",
                    fragment.split_once('\n').unwrap().1
                ));
            }
        }
        assert_eq!(sources.len(), 12);
        for (index, source) in sources.iter().enumerate() {
            std::fs::write(&temporary, source).unwrap();
            let (book, origins) = crate::syntax::load_with_diagnostics(&temporary).unwrap();
            let plain = crate::syntax::load(&temporary).unwrap();
            assert_eq!(format!("{book:?}"), format!("{plain:?}"));
            let mut ordinary = empty_engine(&book).unwrap();
            let mut tracked = ordinary.clone();
            tracked.origins = Some(origins.tracker.clone());
            let mut rejected = false;
            for declaration in &book.declarations {
                let results = [&mut ordinary, &mut tracked].map(|engine| {
                    engine.reset();
                    match declaration {
                        Declaration::Def(d) => engine.validate_def(d),
                        Declaration::Adt(d) => engine.validate_adt(d),
                    }
                });
                assert_eq!(results[0], results[1]);
                assert_eq!(
                    (ordinary.fuel, ordinary.depth, ordinary.fresh_id),
                    (tracked.fuel, tracked.depth, tracked.fresh_id)
                );
                if results[0].is_err() {
                    rejected = true;
                    break;
                }
            }
            assert_eq!(rejected, index != 0);
        }
        std::fs::remove_file(temporary).unwrap();
    }

    #[test]
    fn source_origins_preserve_checker_work_terms_and_successful_evaluation() {
        for (name, entries) in [
            ("cpu-match-guards/match_not_datatype.bend", &[][..]),
            ("cpu-match-guards/empty_match_datatype.bend", &[][..]),
            ("cpu-match-guards/float_match_scrutinee.bend", &[][..]),
            ("cpu-match-guards/error_window_match_repeated.bend", &[][..]),
            ("cpu-match-guards/erased_match_empty.bend", &[][..]),
            ("cpu-match-guards/erased_match_ctor.bend", &[][..]),
            ("cpu-match-guards/unknown_constructor.bend", &[][..]),
            ("cpu-match-guards/bool_wrap_scope.bend", &[][..]),
            ("cpu-match-failures/empty_match.bend", &[][..]),
            (
                "cpu-match-failures/error_window_match_missing.bend",
                &[][..],
            ),
            ("cpu-match-failures/nontotal_multi_missing.bend", &[][..]),
            ("cpu-match-failures/match_app_scrutinee_001.bend", &[][..]),
            (
                "cpu-matched-values/error_window_matched_scrutinee.bend",
                &[][..],
            ),
            ("cpu-constructors/context-open.bend", &[][..]),
            ("cpu-checking/false_fusion.bend", &[][..]),
            ("cpu-rewrite-checking/rewrite_non_equation.bend", &[][..]),
            ("cpu-rewrite-proof/rewrite_non_equation.bend", &[][..]),
            ("cpu-rewrite-proof/error_window_rewrite_proof.bend", &[][..]),
            ("cpu-rewrite-proof/missing_ih_rewrite.bend", &[][..]),
            ("cpu-rewrite-proof/reject_rewrite.bend", &[][..]),
            ("cpu-rewrite-proof/rewrite_goal.bend", &[][..]),
            ("cpu-rewrite-proof/rewrite_type_family.bend", &[][..]),
            ("cpu-rewrite-checking/rewrite_ctor_brace.bend", &[][..]),
            ("cpu-plus-binders/plus_binder_term.bend", &[][..]),
            ("cpu-plus-binders/plus_binder_case.bend", &[][..]),
            ("cpu-plus-binders/plus_binder_lambda_do.bend", &[][..]),
            ("cpu-plus-binders/plus_binder_nested.bend", &[][..]),
            ("cpu-plus-binders/plus_binder_row.bend", &[][..]),
            (
                "cpu-checking/list_reverse_sort.bend",
                &["l123", "l321", "rev", "srt"][..],
            ),
            (
                "cpu-destructure/destructure_let_000.bend",
                &["main", "swap", "go"][..],
            ),
            (
                "cpu-destructure/destructure_then_match_001.bend",
                &["main"][..],
            ),
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(name);
            let (book, origins) = crate::syntax::load_with_diagnostics(&path).unwrap();
            let plain = crate::syntax::load(&path).unwrap();
            assert_eq!(format!("{book:?}"), format!("{plain:?}"));
            let mut ordinary = empty_engine(&book).unwrap();
            let mut tracked = ordinary.clone();
            tracked.origins = Some(origins.tracker.clone());
            for declaration in &book.declarations {
                let results = [&mut ordinary, &mut tracked].map(|engine| {
                    engine.reset();
                    match declaration {
                        Declaration::Def(d) => engine.validate_def(d),
                        Declaration::Adt(d) => engine.validate_adt(d),
                    }
                });
                assert_eq!(results[0], results[1]);
                assert_eq!(
                    (ordinary.fuel, ordinary.depth, ordinary.fresh_id),
                    (tracked.fuel, tracked.depth, tracked.fresh_id)
                );
                if results[0].is_err() {
                    break;
                }
            }
            drop(tracked);
            drop(ordinary);
            if let Ok(checked) = check_book_with_origins(&book, Some(origins.tracker.clone())) {
                assert!(checked.engine.origins.is_none());
                drop(origins);
                let baseline = check_book(&plain).unwrap();
                for entry in entries {
                    assert_eq!(
                        checked.normalize_definition(entry).unwrap().to_string(),
                        baseline.normalize_definition(entry).unwrap().to_string()
                    );
                }
            }
        }
    }

    #[test]
    fn presentation_keeps_aliases_and_uses_only_remaining_allowance() {
        let mut engine = empty_engine(&Book {
            declarations: vec![],
        })
        .unwrap();
        let proof = term(Term::Rfl);
        let left = term(Term::Var {
            name: "x".into(),
            id: 1,
        });
        let right = term(Term::Qua(Quant::Lone));
        engine.aliases.insert(1, Rc::clone(&right));
        let report = engine
            .reflexivity_diagnostic(&Lhs::default(), &proof, &left, &right, &Context::new())
            .unwrap();
        assert_eq!(Some(report.expected.as_str()), report.observed.as_deref());
        assert_eq!(report.expected, right.to_string());
        let fresh = engine.fresh_id;
        for (fuel, depth, expected) in [
            (0, 0, "evaluation budget exhausted"),
            // One endpoint can normalize with two work units. The second must
            // not get a fresh allowance of its own.
            (2, 0, "evaluation budget exhausted"),
            (100, 128, "kernel nesting limit exhausted"),
        ] {
            engine.fuel = fuel;
            engine.depth = depth;
            let error = engine
                .reflexivity_diagnostic(&Lhs::default(), &proof, &right, &right, &Context::new())
                .unwrap_err();
            assert!(error.message.contains(expected), "{error}");
            assert_eq!(
                (engine.fuel, engine.depth, engine.fresh_id),
                (fuel, depth, fresh)
            );
            assert!(Rc::ptr_eq(&engine.aliases[&1], &right));
        }
    }

    #[test]
    fn constructor_presentation_does_not_reset_exhausted_allowance() {
        let mut engine = empty_engine(&Book {
            declarations: vec![],
        })
        .unwrap();
        let value = term(Term::Ctr {
            name: "Missing".into(),
            args: vec![],
        });
        engine.fuel = 1;
        let error = engine
            .infer(&Lhs::default(), &value, Quant::None, &Context::new(), &[])
            .unwrap_err();
        assert_eq!(engine.fuel, 0);
        assert_eq!(engine.depth, 0);
        assert!(error.checker_diagnostic().is_none());
        assert_eq!(
            error,
            KernelError::new("term needs an expected type or annotation: Missing{}")
        );
        assert!(error.language_failure().is_none());
    }

    #[test]
    fn marker_diagnostic_requires_provenance_and_remaining_context_budget() {
        let mut engine = empty_engine(&Book {
            declarations: vec![],
        })
        .unwrap();
        let marker = term(Term::Var {
            name: "x".into(),
            id: 2,
        });
        let context = Context::from([(
            1,
            Binder {
                quant: Quant::Lone,
                name: "x".into(),
                id: 1,
                ty: term(Term::Qnt),
            },
        )]);
        let origins = super::super::origins::TermOrigins::default();
        engine.origins = Some(origins.clone());
        let untracked = engine
            .infer(&Lhs::default(), &marker, Quant::Lone, &context, &[])
            .unwrap_err();
        assert!(untracked.checker_diagnostic().is_none());
        origins.reusable_marker(2);
        engine.reset();
        let tracked = engine
            .infer(&Lhs::default(), &marker, Quant::Lone, &context, &[])
            .unwrap_err();
        assert_eq!(tracked, untracked);
        assert_eq!(tracked.to_string(), "unbound variable x");
        assert_eq!(
            tracked.checker_diagnostic().unwrap().observed.as_deref(),
            Some("x^-1")
        );
        let unshadowed = engine
            .infer(&Lhs::default(), &marker, Quant::Lone, &Context::new(), &[])
            .unwrap_err();
        assert_eq!(
            unshadowed.checker_diagnostic().unwrap().observed.as_deref(),
            Some("x")
        );
        engine.fuel = 1;
        let exhausted = engine
            .infer(&Lhs::default(), &marker, Quant::Lone, &context, &[])
            .unwrap_err();
        assert_eq!(exhausted, untracked);
        assert!(exhausted.checker_diagnostic().is_none());
        assert_eq!((engine.fuel, engine.depth), (0, 0));
        engine.reset();
        let mut bound = context;
        let mut binding = bound.get(1).unwrap().clone();
        binding.id = 2;
        bound.insert(2, binding);
        assert_eq!(
            engine
                .infer(&Lhs::default(), &marker, Quant::Lone, &bound, &[])
                .unwrap()
                .1,
            usage(2, Quant::Lone)
        );
    }

    #[test]
    fn rewrite_and_application_diagnostics_preserve_first_error_and_remaining_budget() {
        let variable = term(Term::Var {
            name: "n".into(),
            id: 1,
        });
        let context = Context::from([(
            1,
            Binder {
                quant: Quant::Lone,
                name: "n".into(),
                id: 1,
                ty: term(Term::Qnt),
            },
        )]);
        // The unused child deliberately fails if visited. This tests the rule
        // boundary directly; whole-book validation still rejects every hole.
        let unused = term(Term::Hole("unused".into()));
        for (value, expected) in [
            (
                term(Term::App(Rc::clone(&variable), Rc::clone(&unused))),
                "expected a function type; observed Quant",
            ),
            (
                term(Term::Rwt {
                    evidence: Rc::clone(&variable),
                    motive: Rc::clone(&unused),
                    body: Rc::clone(&unused),
                }),
                "rewrite evidence must have equality type",
            ),
        ] {
            let evaluate = |engine: &mut Engine| {
                if matches!(value.as_ref(), Term::App(..)) {
                    engine
                        .infer(&Lhs::default(), &value, Quant::Lone, &context, &[])
                        .map(|(_, uses)| uses)
                } else {
                    engine.check(
                        &Lhs::default(),
                        &value,
                        Quant::Lone,
                        &term(Term::Qnt),
                        &context,
                    )
                }
            };
            let mut engine = empty_engine(&Book {
                declarations: vec![],
            })
            .unwrap();
            let initial = engine.fuel;
            let error = evaluate(&mut engine).unwrap_err();
            assert_eq!(error, KernelError::new(expected));
            assert!(error.checker_diagnostic().is_some());
            let spent = initial - engine.fuel;
            engine.fuel = spent;
            let exhausted = evaluate(&mut engine).unwrap_err();
            assert_eq!(exhausted, error);
            assert!(exhausted.checker_diagnostic().is_none());
            assert!(exhausted.language_failure().is_none());
            assert_eq!((engine.fuel, engine.depth), (0, 0));
        }
    }

    #[test]
    fn hole_diagnostics_keep_exhausted_budget_and_underlying_errors() {
        let mut engine = empty_engine(&Book {
            declarations: vec![],
        })
        .unwrap();
        let hole = term(Term::Hole("missing".into()));
        let expected = term(Term::Qnt);
        let before = engine.fuel;
        let error = engine
            .check(
                &Lhs::default(),
                &hole,
                Quant::None,
                &expected,
                &Context::new(),
            )
            .unwrap_err();
        assert_eq!(error, KernelError::new("unfinished proof hole ?missing"));
        assert_eq!(
            error.checker_diagnostic().unwrap().observed.as_deref(),
            Some("?missing")
        );
        let spent = before - engine.fuel;
        engine.fuel = spent;
        let exhausted = engine
            .check(
                &Lhs::default(),
                &hole,
                Quant::None,
                &expected,
                &Context::new(),
            )
            .unwrap_err();
        assert_eq!(error, exhausted);
        assert!(exhausted.checker_diagnostic().is_none());
        assert_eq!((engine.fuel, engine.depth), (0, 0));
        assert!(exhausted.language_failure().is_none());
    }

    #[test]
    fn constructor_non_datatype_diagnostics_keep_remaining_budget_and_original_error() {
        let mut engine = empty_engine(&Book {
            declarations: vec![],
        })
        .unwrap();
        let value = term(Term::Ctr {
            name: "Missing".into(),
            args: vec![],
        });
        let ty = kind(Quant::Lone);
        let before = engine.fuel;
        let error = engine
            .check(&Lhs::default(), &value, Quant::None, &ty, &Context::new())
            .unwrap_err();
        assert_eq!(
            error,
            KernelError::new("expected a datatype; observed Type")
        );
        assert_eq!(
            error.checker_diagnostic().unwrap().observed.as_deref(),
            Some("non-inferrable term 'Missing{}'")
        );
        let spent = before - engine.fuel;
        engine.fuel = spent;
        let exhausted = engine
            .check(&Lhs::default(), &value, Quant::None, &ty, &Context::new())
            .unwrap_err();
        assert_eq!(error, exhausted);
        assert!(exhausted.checker_diagnostic().is_none());
        assert_eq!((engine.fuel, engine.depth), (0, 0));
        assert!(exhausted.language_failure().is_none());
    }

    #[test]
    fn unavailable_presentation_retains_the_original_reflexivity_rejection() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<KernelError>();
        let mut engine = empty_engine(&Book {
            declarations: vec![],
        })
        .unwrap();
        let deep = (0..128).fold(term(Term::Qua(Quant::Lone)), |value, _| {
            term(Term::Ctr {
                name: "A".into(),
                args: vec![value],
            })
        });
        let right = term(Term::Ctr {
            name: "B".into(),
            args: vec![],
        });
        let expected = format!("reflexivity requires equal endpoints: {deep} != {right}");
        let ty = term(Term::Eql {
            left: deep,
            right,
            ty: term(Term::Qnt),
        });
        let error = engine
            .check(
                &Lhs::default(),
                &term(Term::Rfl),
                Quant::None,
                &ty,
                &Context::new(),
            )
            .unwrap_err();
        assert_eq!(error.message, expected);
        assert!(error.checker_diagnostic().is_none());
        assert_eq!(engine.depth, 0);
        assert_eq!(error, KernelError::new(expected));

        let source =
            "type Nat is Data:\n  Zero{}\n  Succ{pred: Nat}\ndef bad() -> {0n == 1n : Nat}: {==}\n";
        let error = check_book(&crate::syntax::parse(source).unwrap()).unwrap_err();
        assert!(error.checker_diagnostic().is_some());
        assert_eq!(error, KernelError::new(error.message.clone()));
        assert_eq!(error.to_string(), error.message);
        assert!(error.language_failure().is_none());
    }
}

#[cfg(test)]
#[path = "composition_work_tests.rs"]
mod composition_work_tests;

#[cfg(test)]
#[path = "law_completion_tests.rs"]
mod law_completion_tests;
