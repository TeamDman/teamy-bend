// SPDX-License-Identifier: Apache-2.0
// Derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
use super::executable::BuiltinForeign;
use super::executable::ExecutableSource;
use super::executable::ForeignDefinition;
use super::executable::ForeignImport;
use super::executable::ForeignTarget;
use super::executable::NumericIntrinsic;
use super::executable::OpaqueType;
use super::surface::Body;
use super::surface::Occurrence;
use super::surface::Pattern;
use super::surface::Row;
use super::surface::SurfaceTerm;
use super::surface::Variable;
use super::surface::flatten_occurrence;
use crate::kernel::AdtDecl;
use crate::kernel::Binder;
use crate::kernel::Book;
use crate::kernel::ConstructorDecl;
use crate::kernel::Declaration;
use crate::kernel::DefDecl;
use crate::kernel::Quant;
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::kernel::arrows;
use crate::kernel::term;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::fmt;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

/// A source location and a precise parse or module loading failure.
#[derive(Clone, Debug)]
pub struct ParseError {
    pub source: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: {}",
            self.source, self.line, self.column, self.message
        )
    }
}
impl std::error::Error for ParseError {}

#[derive(Clone, Debug)]
struct Token {
    text: String,
    line: usize,
    column: usize,
    start: usize,
    end: usize,
}

#[expect(
    clippy::too_many_lines,
    reason = "Token alternatives mirror the upstream grammar in one scan"
)]
fn lex(source: &str, label: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens = Vec::new();
    let mut pos = 0;
    let mut line = 1;
    let mut column = 1;
    while pos < source.len() {
        let rest = &source[pos..];
        let ch = rest.chars().next().expect("nonempty remainder");
        if ch.is_whitespace() {
            pos += ch.len_utf8();
            if ch == '\n' {
                line += 1;
                column = 1;
            } else {
                column += 1;
            }
            continue;
        }
        if ch == '#' {
            while pos < source.len() && source.as_bytes()[pos] != b'\n' {
                pos += 1;
                column += 1;
            }
            continue;
        }
        let start = pos;
        let start_column = column;
        if ch == '"' || ch == '\'' {
            pos += 1;
            let mut escaped = false;
            let mut closed = false;
            while pos < source.len() {
                let c = source[pos..].chars().next().expect("nonempty remainder");
                if c == '\n' {
                    return Err(ParseError {
                        source: label.into(),
                        line,
                        column: start_column,
                        message: "a string or character literal cannot cross a line".into(),
                    });
                }
                pos += c.len_utf8();
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == ch {
                    closed = true;
                    break;
                }
            }
            if !closed {
                return Err(ParseError {
                    source: label.into(),
                    line,
                    column: start_column,
                    message: "unterminated literal".into(),
                });
            }
        } else if ch.is_ascii_alphabetic() || ch == '_' {
            pos += 1;
            while pos < source.len()
                && (source.as_bytes()[pos].is_ascii_alphanumeric()
                    || b"_.".contains(&source.as_bytes()[pos]))
            {
                pos += 1;
            }
            if source.as_bytes()[pos - 1] == b'.' {
                return Err(ParseError {
                    source: label.into(),
                    line,
                    column: start_column,
                    message: "a name cannot end with a dot".into(),
                });
            }
        } else if ch.is_ascii_digit() {
            while pos < source.len() && source.as_bytes()[pos].is_ascii_digit() {
                pos += 1;
            }
            if source[pos..].starts_with('n') {
                pos += 1;
            } else if source[pos..].starts_with('.') {
                pos += 1;
                while pos < source.len() && source.as_bytes()[pos].is_ascii_digit() {
                    pos += 1;
                }
                if source[pos..].starts_with(['e', 'E']) {
                    pos += 1;
                    if source[pos..].starts_with(['+', '-']) {
                        pos += 1;
                    }
                    while pos < source.len() && source.as_bytes()[pos].is_ascii_digit() {
                        pos += 1;
                    }
                }
            }
            if source[pos..].starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') {
                return Err(ParseError {
                    source: label.into(),
                    line,
                    column: start_column,
                    message: "invalid numeric literal suffix".into(),
                });
            }
        } else {
            let punctuation = [
                "<&>", ".|.", ".^.", ".&.", "=>", "->", "==", "!=", "<=", ">=", "<>", "++", "<<",
                ">>", "&&", "||", "<-",
            ];
            if let Some(op) = punctuation.iter().find(|op| rest.starts_with(**op)) {
                pos += op.len();
            } else if "@&+\\%{}()[]?:,;=<>-*/|~!^".contains(ch) {
                pos += ch.len_utf8();
            } else {
                return Err(ParseError {
                    source: label.into(),
                    line,
                    column: start_column,
                    message: format!("unexpected character {ch:?}"),
                });
            }
        }
        column += source[start..pos].chars().count();
        tokens.push(Token {
            text: source[start..pos].into(),
            line,
            column: start_column,
            start,
            end: pos,
        });
    }
    tokens.push(Token {
        text: String::new(),
        line,
        column,
        start: source.len(),
        end: source.len(),
    });
    Ok(tokens)
}

const KEYWORDS: &[&str] = &[
    "def", "type", "law", "match", "case", "do", "return", "for", "exs", "where", "is", "import",
    "Type", "Data", "Kind", "Quant",
];

const MAX_SPECIALIZATIONS: usize = 256;
const MAX_SPECIALIZATION_DEPTH: usize = 16;
const MAX_TEMPLATE_KEY_BYTES: usize = 2048;

#[derive(Clone)]
struct Template {
    tokens: Rc<[Token]>,
    at: usize,
    label: String,
    namespace: String,
    aliases: BTreeMap<String, String>,
    origin: SourceOrigin,
    unsafe_: bool,
    instances: BTreeMap<String, String>,
}

#[derive(Default)]
struct Templates {
    declarations: BTreeMap<String, Template>,
    instances: usize,
    active: usize,
    diagnostics: Option<super::diagnostics::DiagnosticOrigins>,
}

#[derive(Clone, Default)]
enum SourceOrigin {
    #[default]
    Standalone,
    Local(PathBuf),
    Bundled,
}

struct ParseInput<'a> {
    source: &'a str,
    label: &'a str,
    namespace: &'a str,
    aliases: BTreeMap<String, String>,
    origin: SourceOrigin,
}

struct Parser<'a> {
    tokens: Vec<Token>,
    at: usize,
    label: String,
    namespace: String,
    aliases: BTreeMap<String, String>,
    scope: Vec<Variable>,
    fresh: &'a mut usize,
    // Upstream resets display numbering per declaration. Semantic IDs remain
    // unique across the whole book to preserve capture-free substitution.
    display_fresh: usize,
    book: &'a mut Book,
    depth: usize,
    reusable: Vec<(Variable, TermRef)>,
    reusable_markers: BTreeSet<usize>,
    templates: &'a mut Templates,
    instance_name: Option<String>,
    instance_arguments: VecDeque<Occurrence>,
    compile_bindings: BTreeMap<usize, Occurrence>,
    foreign: Option<&'a mut BTreeMap<String, ForeignDefinition>>,
    constructor_tags: Option<&'a mut BTreeMap<String, String>>,
    origin: SourceOrigin,
}

impl Parser<'_> {
    fn current(&self) -> &Token {
        &self.tokens[self.at]
    }
    fn is(&self, s: &str) -> bool {
        self.current().text == s
    }
    fn take(&mut self, s: &str) -> bool {
        // Lexical maximal munch must not hide stacked datatype closers.
        if s == ">" && self.is(">>") {
            let mut remainder = self.current().clone();
            remainder.text = ">".into();
            remainder.start += 1;
            remainder.column += 1;
            self.tokens[self.at].text = ">".into();
            self.tokens[self.at].end -= 1;
            self.tokens.insert(self.at + 1, remainder);
            self.at += 1;
            return true;
        }
        if self.is(s) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn bump(&mut self) -> String {
        let s = self.current().text.clone();
        if !s.is_empty() {
            self.at += 1;
        }
        s
    }
    /// Consume a grammar prefix without losing maximal-munch tokens on replay.
    /// Callers select prefix positions; infix operators keep their whole token.
    fn after_prefix<T>(
        &mut self,
        prefix: char,
        parse: impl FnOnce(&mut Self) -> Result<T, ParseError>,
    ) -> Result<T, ParseError> {
        debug_assert!(self.current().text.starts_with(prefix));
        if self.current().text.len() == prefix.len_utf8() {
            self.at += 1;
            return parse(self);
        }
        let at = self.at;
        let original = self.current().clone();
        let remainder = &mut self.tokens[at];
        remainder.text.remove(0);
        remainder.start += prefix.len_utf8();
        remainder.column += 1;
        let result = parse(self);
        // Nested cursor work is forward of this token. Retained template bodies
        // must see its original spelling, including when the nested parse fails.
        self.tokens[at] = original;
        result
    }
    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            source: self.label.clone(),
            line: self.current().line,
            column: self.current().column,
            message: message.into(),
        }
    }
    fn expect(&mut self, s: &str) -> Result<(), ParseError> {
        if self.take(s) {
            Ok(())
        } else {
            let token = self.current().clone();
            if let Some(origins) = &mut self.templates.diagnostics {
                origins.expected_token(&self.label, token.line, s, &token.text);
            }
            Err(self.error(format!("expected {s:?}, found {:?}", token.text)))
        }
    }
    fn name(&mut self) -> Result<String, ParseError> {
        let s = &self.current().text;
        if !s.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
            || KEYWORDS.contains(&s.as_str())
        {
            return Err(self.error(format!("expected a name, found {:?}", self.current().text)));
        }
        Ok(self.bump())
    }
    fn flatten_body(&mut self, body: &Body, vars: &[Variable]) -> Result<TermRef, ParseError> {
        self.flatten_body_occurrence(body, vars)
            .map(|value| value.value)
    }
    fn flatten_body_occurrence(
        &mut self,
        body: &Body,
        vars: &[Variable],
    ) -> Result<Occurrence, ParseError> {
        flatten_occurrence(
            body,
            vars,
            self.fresh,
            &mut self.display_fresh,
            self.templates.diagnostics.as_ref().map(|o| &o.tracker),
        )
        .map_err(|error| {
            if let Some(origins) = &mut self.templates.diagnostics {
                match &error {
                    super::surface::FlattenError::LocalBinding { binder } => {
                        origins.rejected_local_binding(*binder);
                    }
                    super::surface::FlattenError::ComputedScrutinee { origin } => {
                        origins.rejected_computed_scrutinee(*origin);
                    }
                    super::surface::FlattenError::VariableScrutinee { origin } => {
                        origins.rejected_variable_scrutinee(*origin);
                    }
                    super::surface::FlattenError::ConstructorScrutinee { origin } => {
                        origins.rejected_constructor_scrutinee(*origin);
                    }
                    super::surface::FlattenError::Other(_) => {}
                }
            }
            self.error(error.message())
        })
    }
    fn qualify_operators(&self, value: &TermRef, namespace: &str) -> TermRef {
        let result = qualify_operators(value, namespace);
        if let Some(origins) = &self.templates.diagnostics {
            origins.tracker.transformed(value, &result);
        }
        result
    }
    fn fresh_variable(&mut self, name: String) -> Variable {
        self.display_fresh += 1;
        let id = *self.fresh;
        *self.fresh += 1;
        Variable { name, id }
    }
    fn open(&mut self, name: String) -> Variable {
        let v = self.fresh_variable(name);
        if v.name != "_" {
            self.scope.push(v.clone());
        }
        v
    }
    fn qualify(&self, name: &str) -> String {
        if self.namespace.is_empty() {
            name.into()
        } else {
            format!("{}.{name}", self.namespace)
        }
    }
    fn bind_variable(&mut self, name: String, reusable: bool) -> Variable {
        let original = self.open(name.clone());
        if reusable {
            let binding = self.open(name);
            self.reusable.push((binding, original.term()));
        }
        original
    }
    fn is_reusable(&self, value: &TermRef) -> bool {
        matches!(value.as_ref(), Term::Var { id, .. } if self.reusable_markers.contains(id))
    }
    fn known(&self, name: &str) -> bool {
        self.templates.declarations.contains_key(name)
            || self.book.declarations.iter().any(|d| match d {
                Declaration::Def(d) => d.name == name,
                Declaration::Adt(d) => {
                    d.name == name || d.constructors.iter().any(|c| c.name == name)
                }
            })
    }
    fn resolve(&self, name: &str) -> String {
        if let Some((head, tail)) = name.split_once('.')
            && let Some(namespace) = self.aliases.get(head)
        {
            return format!("{namespace}.{tail}");
        }
        let q = self.qualify(name);
        if self.known(&q) { q } else { name.into() }
    }
    fn variable(&mut self, name: &str, line: usize) -> Occurrence {
        if let Some(variable) = self.scope.iter().rev().find(|v| v.name == name) {
            if let Some(bound) = self.compile_bindings.get(&variable.id) {
                let mut result = bound.clone();
                if result.bare_name {
                    result.source = self
                        .templates
                        .diagnostics
                        .as_mut()
                        .map(|origins| origins.anchor(&self.label, line));
                }
                result.bare_name = false;
                return result;
            }
            return variable.term().into();
        }
        let resolved = self.resolve(name);
        Occurrence {
            bare_name: resolved == name
                && !name.contains('.')
                && !self.templates.declarations.contains_key(name),
            value: term(Term::Ref(resolved)),
            source: None,
        }
    }
    fn occurrence_node(
        &self,
        value: Term,
        sources: impl IntoIterator<Item = Option<usize>>,
    ) -> Occurrence {
        let value = term(value);
        if let Some(origins) = &self.templates.diagnostics {
            origins.tracker.occurrence_children(&value, sources);
        }
        value.into()
    }
    fn application(&mut self, f: Occurrence, x: Occurrence) -> Occurrence {
        let sources = [f.source, x.source];
        let head_source = self
            .templates
            .diagnostics
            .as_ref()
            .map(|o| f.source.unwrap_or_else(|| o.tracker.source(&f.value)));
        let result = self.occurrence_node(Term::App(f.value, x.value), sources);
        if let (Some(origins), Some(source)) = (&mut self.templates.diagnostics, head_source) {
            origins.occurrence_parent(&result, source);
        }
        result
    }
    fn applications(
        &mut self,
        head: TermRef,
        args: impl IntoIterator<Item = Occurrence>,
    ) -> Occurrence {
        args.into_iter()
            .fold(head.into(), |f, x| self.application(f, x))
    }
    fn constructor(&self, name: String, args: Vec<Occurrence>) -> Occurrence {
        let sources: Vec<_> = args.iter().map(|x| x.source).collect();
        self.occurrence_node(
            Term::Ctr {
                name,
                args: args.into_iter().map(|x| x.value).collect(),
            },
            sources,
        )
    }
    fn lambda(&self, name: String, id: usize, body: Occurrence) -> Occurrence {
        let sources = [body.source];
        self.occurrence_node(
            Term::Lam {
                name,
                id,
                body: body.value,
            },
            sources,
        )
    }
    fn annotation(&self, value: Occurrence, ty: Occurrence) -> Occurrence {
        let sources = [value.source, ty.source];
        self.occurrence_node(Term::Ann(value.value, ty.value), sources)
    }
    fn all(
        &self,
        quant: Quant,
        name: String,
        id: usize,
        domain: Occurrence,
        body: Occurrence,
    ) -> Occurrence {
        let sources = [domain.source, body.source];
        self.occurrence_node(
            Term::All {
                quant,
                name,
                id,
                domain: domain.value,
                body: body.value,
            },
            sources,
        )
    }
    fn occurrence_arrows(&self, binders: &[Binder], mut result: Occurrence) -> TermRef {
        for binder in binders.iter().rev() {
            let source = self
                .templates
                .diagnostics
                .as_ref()
                .and_then(|origins| origins.type_source(binder.id));
            let domain = Occurrence {
                value: Rc::clone(&binder.ty),
                source,
                bare_name: false,
            };
            result = self.all(binder.quant, binder.name.clone(), binder.id, domain, result);
        }
        result.value
    }
    fn qualify_occurrence(&self, mut value: Occurrence, namespace: &str) -> Occurrence {
        value.value = self.qualify_operators(&value.value, namespace);
        value
    }
    fn is_compile_binding(&self, value: &TermRef) -> bool {
        self.compile_bindings
            .values()
            .any(|bound| Rc::ptr_eq(&bound.value, value))
    }
    fn quant(&mut self) -> Quant {
        if self.take("-") {
            Quant::None
        } else if self.take("+") {
            Quant::Many
        } else {
            Quant::Lone
        }
    }
    fn telescope(&mut self, close: &str) -> Result<Vec<Binder>, ParseError> {
        let mut binders = Vec::new();
        while !self.take(close) {
            if binders.len() >= 128 {
                return Err(self.error("binder telescope exceeds the parser resource limit of 128"));
            }
            let line = self.current().line;
            let compile_time = close == ")" && self.take("~");
            let quant = if compile_time {
                Quant::None
            } else {
                self.quant()
            };
            let name = self.name()?;
            let (quant, ty) = if self.take(":") {
                (quant, self.expression(0)?)
            } else if quant == Quant::Lone {
                (Quant::None, term(Term::Qnt).into())
            } else {
                return Err(self.error("expected a type after a quantified binder"));
            };
            let variable = self.open(name.clone());
            if let Some(origins) = &mut self.templates.diagnostics {
                origins.binder(variable.id, &self.label, line);
                origins.binder_type(variable.id, ty.source);
            }
            if compile_time && let Some(argument) = self.instance_arguments.pop_front() {
                self.compile_bindings.insert(variable.id, argument);
            }
            binders.push(Binder {
                quant,
                name,
                id: variable.id,
                ty: ty.value,
            });
            self.take(",");
        }
        Ok(binders)
    }
    fn declarations(&mut self) -> Result<(), ParseError> {
        while !self.is("") {
            self.scope.clear();
            self.display_fresh = 0;
            self.compile_bindings.clear();
            if self.take("type") {
                self.datatype()?;
            } else if self.take("law") {
                self.law()?;
            } else if self.take("def") {
                self.definition(false)?;
            } else if self.take("@") {
                self.expect("unsafe")?;
                self.expect("def")?;
                self.definition(true)?;
            } else {
                return Err(self.error("expected type, law, or def"));
            }
        }
        Ok(())
    }
    fn require_fresh(&self, name: &str) -> Result<(), ParseError> {
        if self.templates.declarations.contains_key(name)
            || self.book.declarations.iter().any(|d| match d {
                Declaration::Adt(d) => d.name == name,
                Declaration::Def(d) => d.name == name,
            })
        {
            Err(self.error(format!("duplicate declaration: {name}")))
        } else {
            Ok(())
        }
    }
    fn datatype(&mut self) -> Result<(), ParseError> {
        let raw = self.name()?;
        let name = self.qualify(&raw);
        self.require_fresh(&name)?;
        let erased_first = self.is("<-");
        if erased_first {
            let token = &mut self.tokens[self.at];
            token.text = "-".into();
            token.start += 1;
            token.column += 1;
        }
        let parameters = if self.take("<>") {
            Vec::new()
        } else if erased_first || self.take("<") {
            self.telescope(">")?
        } else {
            Vec::new()
        };
        self.expect("is")?;
        let kind = self.expression(0)?;
        self.expect(":")?;
        let index = self.book.declarations.len();
        self.book.declarations.push(Declaration::Adt(AdtDecl {
            name: name.clone(),
            parameters: parameters.clone(),
            kind: Rc::clone(&kind),
            constructors: Vec::new(),
        }));
        let mut constructors = Vec::new();
        while !self.is("") && !["type", "law", "def", "@"].contains(&self.current().text.as_str()) {
            let raw = self.name()?;
            let constructor = self.qualify(&raw);
            if let Some(tags) = self.constructor_tags.as_deref_mut() {
                tags.insert(constructor.clone(), raw);
            }
            if self.book.declarations.iter().any(|d|matches!(d,Declaration::Adt(a) if a.constructors.iter().any(|c|c.name==constructor))) || constructors.iter().any(|c:&ConstructorDecl|c.name==constructor) {return Err(self.error(format!("duplicate constructor: {constructor}")));}
            self.expect("{")?;
            let old = self.scope.len();
            let fields = self.telescope("}")?;
            self.scope.truncate(old);
            constructors.push(ConstructorDecl {
                name: constructor,
                fields,
            });
        }
        self.book.declarations[index] = Declaration::Adt(AdtDecl {
            name,
            parameters,
            kind: kind.value,
            constructors,
        });
        Ok(())
    }
    // Sugar keeps its fixed or already-resolved global name. Only the
    // introducing token's diagnostic origin is attached; no core node changes.
    fn reference_at(&mut self, name: String, line: usize) -> TermRef {
        let value = term(Term::Ref(name));
        if let Some(origins) = &mut self.templates.diagnostics {
            origins.node(&value, &self.label, line);
        }
        value
    }
    fn law(&mut self) -> Result<(), ParseError> {
        let raw = self.name()?;
        let name = self.qualify(&raw);
        self.require_fresh(&name)?;
        self.expect(":")?;
        let mut clauses = Vec::new();
        let mut parameters = Vec::new();
        let mut universal = true;
        while self.is("for") || self.is("exs") {
            if clauses.len() >= 128 {
                return Err(self.error("law telescope exceeds the parser resource limit of 128"));
            }
            let line = self.current().line;
            let all = self.take("for");
            if !all {
                self.expect("exs")?;
                universal = false;
            }
            let quant = if all { self.quant() } else { Quant::Lone };
            let name = self.name()?;
            self.expect(":")?;
            let mut ty = self.expression(0)?;
            let where_line = self.current().line;
            if self.take("where") {
                let old = self.scope.len();
                let v = self.open(name.clone());
                let predicate = self.expression(0)?;
                self.scope.truncate(old);
                let head = self.reference_at("Exists".into(), where_line);
                ty = self.applications(head, [ty, self.lambda(name.clone(), v.id, predicate)]);
            }
            let v = self.open(name.clone());
            let b = Binder {
                quant,
                name,
                id: v.id,
                ty: ty.value,
            };
            if universal {
                parameters.push(b.clone());
            }
            clauses.push((all, b, line));
        }
        let body = self.body(0)?;
        let mut ty = self.flatten_body(&body, &[])?;
        for (all, b, line) in clauses.into_iter().rev() {
            ty = if all {
                arrows(&[b], ty)
            } else {
                apply(
                    self.reference_at("Exists".into(), line),
                    [
                        b.ty,
                        term(Term::Lam {
                            name: b.name,
                            id: b.id,
                            body: ty,
                        }),
                    ],
                )
            };
        }
        self.book.declarations.push(Declaration::Def(DefDecl {
            completion_parameters: None,
            name,
            parameters,
            ty: self.qualify_operators(&ty, &self.resolve("Nat")),
            body: None,
            unsafe_: false,
            foreign: false,
        }));
        Ok(())
    }
    fn definition(&mut self, unsafe_: bool) -> Result<(), ParseError> {
        let declaration_at = self.at;
        let raw = self.name()?;
        let resolved = self.resolve(&raw);
        let prior = self.book.declarations.iter().rev().find_map(|d| match d {
            Declaration::Def(d) if d.name == resolved && self.instance_name.is_none() => {
                Some(d.clone())
            }
            _ => None,
        });
        let name = if let Some(instance) = &self.instance_name {
            instance.clone()
        } else if prior.is_some() {
            resolved
        } else {
            self.qualify(&raw)
        };
        self.expect("(")?;
        let template = prior.is_none() && self.instance_name.is_none() && self.is("~");
        let completing_law = prior.is_some();
        let (parameters, ty, vars) = if let Some(prior) = prior {
            if prior.body.is_some() || prior.foreign {
                return Err(self.error(format!("duplicate definition: {name}")));
            }
            let vars = self.completion_variables()?;
            (prior.parameters, prior.ty, vars)
        } else {
            self.require_fresh(&name)?;
            if template {
                self.templates.declarations.insert(
                    name.clone(),
                    Template {
                        tokens: self.tokens.clone().into(),
                        at: declaration_at,
                        label: self.label.clone(),
                        namespace: self.namespace.clone(),
                        aliases: self.aliases.clone(),
                        origin: self.origin.clone(),
                        unsafe_,
                        instances: BTreeMap::new(),
                    },
                );
            }
            let parameters = self.telescope(")")?;
            self.expect("->")?;
            let result = self.expression(0)?;
            let ty = self.occurrence_arrows(&parameters, result);
            let vars = parameters
                .iter()
                .map(|b| Variable {
                    name: b.name.clone(),
                    id: b.id,
                })
                .collect();
            (parameters, ty, vars)
        };
        self.expect(":")?;
        if self.is("import") {
            return self.foreign_definition(
                name,
                &raw,
                &vars,
                &ty,
                unsafe_,
                template.then_some(declaration_at),
            );
        }
        let completion_parameters = completing_law.then(|| {
            vars.iter()
                .map(|v| (v.name.clone(), v.id))
                .collect::<Vec<_>>()
        });
        // Make recursive names visible while parsing the body, then retain one event.
        let temporary = self.book.declarations.len();
        self.book.declarations.push(Declaration::Def(DefDecl {
            completion_parameters: completion_parameters.clone(),
            name: name.clone(),
            parameters: parameters.clone(),
            ty: Rc::clone(&ty),
            body: None,
            unsafe_,
            foreign: false,
        }));
        let body = self.body(0)?;
        let value = self.flatten_body(&body, &vars)?;
        // Specializations minted in the body precede their caller. Removing
        // only the temporary event preserves those separately checked defs.
        self.book.declarations.remove(temporary);
        if template {
            self.retain_template_source(&name, declaration_at);
        } else {
            self.book.declarations.push(Declaration::Def(DefDecl {
                completion_parameters,
                name,
                parameters,
                ty: self.qualify_operators(&ty, &self.resolve("Nat")),
                body: Some(self.qualify_operators(&value, &self.resolve("Nat"))),
                unsafe_,
                foreign: false,
            }));
        }
        Ok(())
    }

    fn completion_variables(&mut self) -> Result<Vec<Variable>, ParseError> {
        let mut vars = Vec::new();
        while !self.take(")") {
            if vars.len() >= 128 {
                return Err(self.error("proof telescope exceeds the parser resource limit of 128"));
            }
            let line = self.current().line;
            let name = self.name()?;
            let variable = self.open(name);
            if let Some(origins) = &mut self.templates.diagnostics {
                origins.binder(variable.id, &self.label, line);
            }
            vars.push(variable);
            self.take(",");
        }
        Ok(vars)
    }

    fn retain_template_source(&mut self, name: &str, declaration_at: usize) {
        // Retain only this declaration, not another copy of its full file.
        let mut tokens = self.tokens[declaration_at..self.at].to_vec();
        let mut end = self.current().clone();
        end.text.clear();
        tokens.push(end);
        let saved = self
            .templates
            .declarations
            .get_mut(name)
            .expect("registered template");
        saved.tokens = tokens.into();
        saved.at = 0;
    }

    fn foreign_definition(
        &mut self,
        name: String,
        local_name: &str,
        vars: &[Variable],
        ty: &TermRef,
        unsafe_: bool,
        template_at: Option<usize>,
    ) -> Result<(), ParseError> {
        if self.foreign.is_none() {
            return Err(self.error(
                "foreign C/JavaScript definitions are not supported by the Rust proof checker",
            ));
        }
        let imports = self.foreign_imports()?;
        if let Some(declaration_at) = template_at {
            self.retain_template_source(&name, declaration_at);
            return Ok(());
        }
        let mut cursor = Rc::clone(ty);
        let mut parameters = Vec::with_capacity(vars.len());
        for variable in vars {
            while let Term::Ann(value, _) = cursor.as_ref() {
                cursor = Rc::clone(value);
            }
            let Term::All {
                quant,
                id,
                domain,
                body,
                ..
            } = cursor.as_ref()
            else {
                return Err(
                    self.error("foreign parameter list exceeds its declared function telescope")
                );
            };
            parameters.push(Binder {
                quant: *quant,
                name: variable.name.clone(),
                id: *id,
                ty: Rc::clone(domain),
            });
            cursor = Rc::clone(body);
        }
        let builtin = if matches!(self.origin, SourceOrigin::Bundled) {
            match local_name {
                "IO.print" => Some(BuiltinForeign::Print),
                "IO.write" => Some(BuiltinForeign::Write),
                "IO.print_err" => Some(BuiltinForeign::PrintErr),
                "IO.spawn" => Some(BuiltinForeign::Spawn),
                "IO.sleep" => Some(BuiltinForeign::Sleep),
                "IO.now" => Some(BuiltinForeign::Now),
                "Chan.new" => Some(BuiltinForeign::ChanNew),
                "Chan.send" => Some(BuiltinForeign::ChanSend),
                "Chan.recv" => Some(BuiltinForeign::ChanRecv),
                "Chan.close" => Some(BuiltinForeign::ChanClose),
                "IO.get_env" => Some(BuiltinForeign::GetEnv),
                "File.open" => Some(BuiltinForeign::FileOpen),
                "File.read" => Some(BuiltinForeign::FileRead),
                "File.read_bytes" => Some(BuiltinForeign::FileReadBytes),
                "File.write" => Some(BuiltinForeign::FileWrite),
                "File.close" => Some(BuiltinForeign::FileClose),
                "TCP.listen" => Some(BuiltinForeign::TcpListen),
                "TCP.accept" => Some(BuiltinForeign::TcpAccept),
                "TCP.connect" => Some(BuiltinForeign::TcpConnect),
                "TCP.send" => Some(BuiltinForeign::TcpSend),
                "TCP.recv" => Some(BuiltinForeign::TcpRecv),
                "UDP.bind" => Some(BuiltinForeign::UdpBind),
                "UDP.send_to" => Some(BuiltinForeign::UdpSendTo),
                "UDP.recv_from" => Some(BuiltinForeign::UdpRecvFrom),
                "UDP.poll" => Some(BuiltinForeign::UdpPoll),
                "Socket.close" => Some(BuiltinForeign::SocketClose),
                "Listener.close" => Some(BuiltinForeign::ListenerClose),
                _ => None,
            }
        } else {
            None
        };
        let descriptor = ForeignDefinition {
            imports,
            local_symbol: local_name.to_lowercase().replace(['.', '/'], "_"),
            declared_arity: vars.len(),
            parameters: vars.iter().map(|variable| variable.name.clone()).collect(),
            builtin,
        };
        self.foreign
            .as_deref_mut()
            .expect("execution parser")
            .insert(name.clone(), descriptor);
        self.book.declarations.push(Declaration::Def(DefDecl {
            completion_parameters: None,
            name,
            parameters,
            ty: self.qualify_operators(ty, &self.resolve("Nat")),
            body: None,
            unsafe_,
            foreign: true,
        }));
        Ok(())
    }

    fn foreign_imports(&mut self) -> Result<Vec<ForeignImport>, ParseError> {
        let mut imports = Vec::new();
        let mut seen = BTreeSet::new();
        let mut count = 0;
        while self.take("import") {
            if count >= 128 {
                return Err(self.error("foreign import list exceeds the resource limit of 128"));
            }
            count += 1;
            let token = self.bump();
            if !token.starts_with('"') {
                return Err(self.error("foreign imports require a quoted .c or .js path"));
            }
            // Foreign paths are raw quoted text upstream, not Bend string
            // values: a Windows backslash must not become a string escape.
            let relative = &token[1..token.len() - 1];
            let target = match Path::new(&relative)
                .extension()
                .and_then(|extension| extension.to_str())
            {
                Some("c") => ForeignTarget::C,
                Some("js") => ForeignTarget::JavaScript,
                _ => return Err(self.error("foreign imports require a .c or .js path")),
            };
            if relative.contains('\0') {
                return Err(self.error("foreign import paths cannot contain NUL"));
            }
            let path = match &self.origin {
                SourceOrigin::Local(directory) => {
                    let path = directory.join(relative);
                    match fs::canonicalize(&path) {
                        Ok(real) => real,
                        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                            lexical_path(&path)
                        }
                        Err(error) => return Err(self.error(error.to_string())),
                    }
                }
                SourceOrigin::Bundled => lexical_path(Path::new(&relative)),
                SourceOrigin::Standalone => {
                    return Err(self.error("foreign imports need a declaring source file"));
                }
            };
            if seen.insert((target, path.clone())) {
                imports.push(ForeignImport { target, path });
            }
        }
        Ok(imports)
    }

    fn call(&mut self, mut function: Occurrence) -> Result<Occurrence, ParseError> {
        let template = match function.as_ref() {
            Term::Ref(name) | Term::GpuRef(name)
                if self.templates.declarations.contains_key(name) =>
            {
                Some(name.clone())
            }
            _ => None,
        };
        let mut compile_arguments = Vec::new();
        if template.is_some() {
            while self.take("~") {
                if compile_arguments.len() >= 128 {
                    return Err(
                        self.error("template argument list exceeds the resource limit of 128")
                    );
                }
                // Operators in a compile argument are fixed at that argument's
                // boundary, before an enclosing caller annotation can name them.
                let argument = self.expression(0)?;
                compile_arguments.push(self.qualify_occurrence(argument, &self.resolve("Nat")));
                self.take(",");
            }
        }
        let runtime_arguments = self.arguments(")")?;
        if let Some(name) = template
            && let Some(instance) = self.specialize(&name, &compile_arguments)?
        {
            function.value = term(if matches!(function.as_ref(), Term::GpuRef(_)) {
                Term::GpuRef(instance)
            } else {
                Term::Ref(instance)
            });
        }
        for argument in compile_arguments.into_iter().chain(runtime_arguments) {
            function = if let Term::Lam { id, body, .. } = function.as_ref()
                && self.is_compile_binding(&function)
            {
                // Upstream Sub is resolved before live checking: duplicating a
                // macro parameter duplicates caller syntax, not an affine Lam.
                let result = crate::kernel::origins::substitute_with_origins(
                    body,
                    *id,
                    &argument,
                    self.templates.diagnostics.as_ref().map(|o| &o.tracker),
                );
                let root_replaced =
                    matches!(body.as_ref(), Term::Var { id: variable, .. } if variable == id);
                if let (Some(origins), Some(source)) =
                    (&self.templates.diagnostics, argument.source)
                {
                    origins
                        .tracker
                        .substituted_occurrence(body, &result, *id, source);
                }
                Occurrence {
                    value: result,
                    source: if root_replaced {
                        argument.source
                    } else {
                        self.templates
                            .diagnostics
                            .as_ref()
                            .and_then(|o| o.tracker.occurrence_source(&function, 0))
                    },
                    bare_name: root_replaced && argument.bare_name,
                }
            } else {
                self.application(function, argument)
            };
        }
        Ok(function)
    }

    fn specialize(
        &mut self,
        name: &str,
        arguments: &[Occurrence],
    ) -> Result<Option<String>, ParseError> {
        let values: Vec<_> = arguments
            .iter()
            .map(|argument| Rc::clone(&argument.value))
            .collect();
        let Some(key) = template_key(&values).map_err(|error| self.error(error))? else {
            // Open arguments must keep the undefined template name. In
            // particular a caller-local variable never becomes a global Ref.
            return Ok(None);
        };
        let template = self
            .templates
            .declarations
            .get(name)
            .expect("known template");
        if let Some(instance) = template.instances.get(&key) {
            return Ok(Some(instance.clone()));
        }
        if self.templates.instances >= MAX_SPECIALIZATIONS
            || self.templates.active >= MAX_SPECIALIZATION_DEPTH
        {
            return Err(self.error("template specialization count or nesting budget exhausted"));
        }
        let instance = format!("{name}~{}", template.instances.len());
        let snapshot = template.clone();
        self.templates
            .declarations
            .get_mut(name)
            .expect("known template")
            .instances
            .insert(key, instance.clone());
        self.templates.instances += 1;
        self.templates.active += 1;
        let result = Parser {
            tokens: snapshot.tokens.to_vec(),
            at: snapshot.at,
            label: snapshot.label,
            namespace: snapshot.namespace,
            aliases: snapshot.aliases,
            scope: Vec::new(),
            fresh: self.fresh,
            display_fresh: self.display_fresh,
            book: self.book,
            depth: 0,
            reusable: Vec::new(),
            reusable_markers: BTreeSet::new(),
            templates: self.templates,
            instance_name: Some(instance.clone()),
            instance_arguments: arguments.iter().cloned().collect(),
            compile_bindings: BTreeMap::new(),
            foreign: self.foreign.as_deref_mut(),
            constructor_tags: self.constructor_tags.as_deref_mut(),
            origin: snapshot.origin,
        }
        .definition(snapshot.unsafe_);
        self.templates.active -= 1;
        result?;
        Ok(Some(instance))
    }
    fn expression(&mut self, min: u8) -> Result<Occurrence, ParseError> {
        if self.depth >= 64 {
            return Err(self.error("expression nesting exceeds the parser resource limit of 64"));
        }
        self.depth += 1;
        let result = self.expression_inner(min);
        self.depth -= 1;
        if let Ok(value) = &result
            && !within_term_depth(value, 256)
        {
            return Err(self.error("term depth exceeds the parser resource limit of 256"));
        }
        result
    }
    #[expect(
        clippy::too_many_lines,
        reason = "Pratt parser alternatives share precedence and source adjacency state"
    )]
    fn expression_inner(&mut self, min: u8) -> Result<Occurrence, ParseError> {
        let line = self.current().line;
        let mut out = self.atom()?;
        let mut operations = 0;
        loop {
            operations += 1;
            if operations > 256 {
                return Err(self.error("expression spine exceeds the parser resource limit of 256"));
            }
            let same_line = self.at > 0 && self.tokens[self.at - 1].line == self.current().line;
            if self.is("!") {
                let next = self.tokens.get(self.at + 1);
                if !next.is_some_and(|token| token.text == "(" && token.start == self.current().end)
                {
                    return Err(self.error("expected '(' immediately after the GPU call mark !"));
                }
                let (Term::Ref(name) | Term::GpuRef(name)) = out.as_ref() else {
                    return Err(self.error("a named def before ! (only f!(..) offloads)"));
                };
                out.value = term(Term::GpuRef(name.clone()));
                self.bump();
                continue;
            }
            if self.is("(") && same_line {
                self.bump();
                out = self.call(out)?;
                continue;
            }
            if self.is("[") && same_line {
                self.bump();
                let index = self.expression(0)?;
                self.expect("]")?;
                let index = self.qualify_occurrence(index, "U32");
                if self.take("<-") {
                    let value = self.expression(2)?;
                    out = self.applications(
                        term(Term::Ref("Array.set".into())),
                        [term(Term::Ref("U32".into())).into(), out, index, value],
                    );
                } else {
                    out = self.applications(
                        term(Term::Ref("Array.get".into())),
                        [term(Term::Ref("U32".into())).into(), out, index],
                    );
                }
                continue;
            }
            if self.is("<")
                && (min <= 4 || self.at > 0 && self.tokens[self.at - 1].end == self.current().start)
            {
                self.bump();
                let first = self.expression(5)?;
                if self.is(">") || self.is(">>") || self.is(",") {
                    let name = match out.as_ref() {
                        Term::Ref(n) | Term::GpuRef(n) | Term::Var { name: n, .. } => {
                            self.resolve(n)
                        }
                        _ => return Err(self.error("type arguments require a datatype name")),
                    };
                    let mut args = vec![first];
                    if !self.take(">") {
                        self.expect(",")?;
                        args.extend(self.arguments(">")?);
                    }
                    if let Some(Declaration::Adt(adt)) = self
                        .book
                        .declarations
                        .iter()
                        .find(|d| matches!(d,Declaration::Adt(a) if a.name==name))
                    {
                        let quantities = adt
                            .parameters
                            .iter()
                            .take_while(|b| matches!(b.ty.as_ref(), Term::Qnt))
                            .count();
                        if args.len() + quantities == adt.parameters.len() {
                            args.splice(
                                0..0,
                                (0..quantities).map(|_| term(Term::Qua(Quant::Lone)).into()),
                            );
                        }
                    }
                    let sources: Vec<_> = args.iter().map(|x| x.source).collect();
                    let grown = self.occurrence_node(
                        Term::Adt {
                            name,
                            args: args.into_iter().map(|x| x.value).collect(),
                            excluded: Vec::new(),
                        },
                        sources,
                    );
                    if let Some(origins) = &mut self.templates.diagnostics {
                        origins.grown(&grown, &out, out.source, &self.label, line);
                    }
                    out = grown;
                } else {
                    out = self.applications(term(Term::Ref(".is_lt".into())), [out, first]);
                }
                continue;
            }
            if min == 0 && self.take("=>") {
                if self.is_compile_binding(&out) {
                    return Err(
                        self.error("a template argument cannot be rebound as a lambda binder")
                    );
                }
                let name = match out.as_ref() {
                    Term::Ref(n) | Term::Var { name: n, .. } => n.clone(),
                    _ => return Err(self.error("a lambda binder must be one name")),
                };
                let old = self.scope.len();
                let reusable = self.is_reusable(&out);
                let v = self.bind_variable(name.clone(), reusable);
                let body = self.body(self.current().column.saturating_sub(1))?;
                out = self.flatten_body_occurrence(&body, &[v])?;
                if let Some(origins) = &mut self.templates.diagnostics {
                    origins.node(&out, &self.label, line);
                }
                self.scope.truncate(old);
                continue;
            }
            if min == 0 && self.take("->") {
                let body = self.expression(0)?;
                let v = self.fresh_variable("_".into());
                let grown = self.all(Quant::Lone, v.name, v.id, out.clone(), body);
                if let Some(origins) = &mut self.templates.diagnostics {
                    origins.grown(&grown, &out, out.source, &self.label, line);
                }
                out = grown;
                continue;
            }
            let Some((prec, right, method)) = operator(&self.current().text) else {
                break;
            };
            if prec < min {
                break;
            }
            if self.current().text.starts_with('>')
                && self.at > 0
                && self.tokens[self.at - 1].end == self.current().start
            {
                break;
            }
            if ["+", "-"].contains(&self.current().text.as_str())
                && self.tokens.get(self.at + 1).is_some_and(|t| {
                    t.start == self.current().end
                        && t.text
                            .starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                })
            {
                break;
            }
            if self.is("%")
                && self
                    .tokens
                    .get(self.at + 1)
                    .is_some_and(|t| t.start == self.current().end)
            {
                break;
            }
            let op = self.bump();
            let b = self.expression(if right { prec } else { prec + 1 })?;
            out = match op.as_str() {
                "<&>" => {
                    let sources = [out.source, b.source];
                    self.occurrence_node(Term::Min(out.value, b.value), sources)
                }
                "<>" => {
                    let value = self.constructor("Con".into(), vec![out, b]);
                    if let Some(origins) = &mut self.templates.diagnostics {
                        origins.node(&value, &self.label, line);
                    }
                    value
                }
                _ => self.applications(term(Term::Ref(method.into())), [out, b]),
            };
        }
        if let Some(origins) = &mut self.templates.diagnostics {
            origins.application(&out, &self.label, line);
        }
        Ok(out)
    }
    fn arguments(&mut self, close: &str) -> Result<Vec<Occurrence>, ParseError> {
        let mut xs = Vec::new();
        while !self.take(close) {
            if xs.len() >= 128 {
                return Err(self.error("argument list exceeds the parser resource limit of 128"));
            }
            xs.push(self.expression(0)?);
            self.take(",");
        }
        Ok(xs)
    }
    fn atom(&mut self) -> Result<Occurrence, ParseError> {
        let line = self.current().line;
        let value = self.atom_inner()?;
        if matches!(
            value.as_ref(),
            Term::Typ(_)
                | Term::All { .. }
                | Term::Adt { .. }
                | Term::Ctr { .. }
                | Term::Rwt { .. }
                | Term::Eql { .. }
                | Term::Var { .. }
                | Term::Ref(_)
                | Term::Ann(..)
                | Term::Hole(_)
                | Term::Lam { .. }
                | Term::Mat { .. }
                | Term::Efq
                | Term::Rfl
        ) && let Some(origins) = &mut self.templates.diagnostics
        {
            origins.node(&value, &self.label, line);
        }
        Ok(value)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "Surface forms are kept in upstream grammar order for auditability"
    )]
    fn atom_inner(&mut self) -> Result<Occurrence, ParseError> {
        if self.is("~") {
            return Err(self.error(
                "~ arguments require a previously declared template head and leading argument position",
            ));
        }
        if self.take("Type") {
            return Ok(term(Term::Typ(term(Term::Qua(Quant::Lone)))).into());
        }
        if self.take("Data") {
            return Ok(term(Term::Typ(term(Term::Qua(Quant::Many)))).into());
        }
        if self.take("Quant") {
            return Ok(term(Term::Qnt).into());
        }
        if self.take("Kind") {
            self.expect("(")?;
            let g = self.expression(0)?;
            self.expect(")")?;
            let sources = [g.source];
            return Ok(self.occurrence_node(Term::Typ(g.value), sources));
        }
        if self.take("do") {
            let module = self.name()?;
            let mut types = if self.take("<>") {
                Vec::new()
            } else {
                self.expect("<")?;
                self.arguments(">")?
            };
            let result = types.pop();
            self.expect(":")?;
            return self.do_statement(&module, &types, result.as_ref(), self.current().column);
        }
        if self.is("@") || self.is("&") {
            let line = self.current().line;
            let exists = self.take("&");
            if !exists {
                self.expect("@")?;
            }
            if exists {
                let q = match self.current().text.as_str() {
                    "0" => Some(Quant::None),
                    "1" => Some(Quant::Lone),
                    "2" => Some(Quant::Many),
                    _ => None,
                };
                if let Some(q) = q {
                    self.bump();
                    return Ok(term(Term::Qua(q)).into());
                }
            }
            let quant = if exists { Quant::Lone } else { self.quant() };
            let name = self.name()?;
            self.expect(":")?;
            let domain = self.expression(1)?;
            self.expect("->")?;
            let old = self.scope.len();
            let v = self.open(name.clone());
            let body = self.expression(0)?;
            self.scope.truncate(old);
            return Ok(if exists {
                let head = self.reference_at("Exists".into(), line);
                self.applications(head, [domain, self.lambda(name, v.id, body)])
            } else {
                self.all(quant, name, v.id, domain, body)
            });
        }
        if self.take("{") {
            if self.take("==") {
                let line = self.tokens[self.at - 2].line;
                self.expect("}")?;
                let proof = term(Term::Rfl);
                if let Some(origins) = &mut self.templates.diagnostics {
                    origins.node(&proof, &self.label, line);
                }
                return Ok(proof.into());
            }
            let left = self.expression(0)?;
            let equality = self.take("==");
            let inequality_line = self.current().line;
            let unequal = !equality && self.take("!=");
            if equality || unequal {
                let right = self.expression(0)?;
                self.expect(":")?;
                let ty = self.expression(0)?;
                self.expect("}")?;
                let sources = [left.source, right.source, ty.source];
                let equation = self.occurrence_node(
                    Term::Eql {
                        left: left.value,
                        right: right.value,
                        ty: ty.value,
                    },
                    sources,
                );
                if unequal {
                    let v = self.fresh_variable("_".into());
                    let body = self.reference_at("Empty".into(), inequality_line).into();
                    return Ok(self.all(Quant::Lone, v.name, v.id, equation, body));
                }
                return Ok(equation);
            }
            self.expect(":")?;
            let ty = self.expression(0)?;
            self.expect("}")?;
            return Ok(self.annotation(left, ty));
        }
        if self.take("(") {
            let body = self.body(self.current().column.saturating_sub(1))?;
            let mut out = self.flatten_body_occurrence(&body, &[])?;
            if self.take(",") {
                let mut values = vec![out];
                loop {
                    values.push(self.expression(0)?);
                    if !self.take(",") {
                        break;
                    }
                }
                let last = values.pop().expect("at least two tuple values");
                out = values.into_iter().rev().fold(last, |tail, head| {
                    self.constructor("Tuple".into(), vec![head, tail])
                });
            }
            if self.take(":") {
                let ty = self.expression(0)?;
                let name = type_head(&ty)
                    .ok_or_else(|| self.error("operators need a named type after ':'"))?;
                out = self.qualify_occurrence(out, &name);
            }
            self.expect(")")?;
            return Ok(out);
        }
        if self.take("\\") {
            let match_line = self.tokens[self.at - 1].line;
            self.expect("{")?;
            let mut arms = Vec::new();
            let mut tail: Occurrence = term(Term::Efq).into();
            if let Some(origins) = &mut self.templates.diagnostics {
                origins.node(&tail, &self.label, match_line);
            }
            while !self.take("}") {
                let value = self.expression(0)?;
                if self.take(":") {
                    let name = match value.as_ref() {
                        Term::Ref(n) | Term::Var { name: n, .. } => self.resolve(n),
                        _ => return Err(self.error("a lambda-match arm needs a constructor name")),
                    };
                    let arm = self.expression(0)?;
                    let source = self.templates.diagnostics.as_ref().map_or_else(
                        || Rc::as_ptr(&value) as usize,
                        |origins| origins.tracker.source(&value),
                    );
                    arms.push((name, arm, source));
                    self.take(";");
                } else {
                    tail = value;
                    self.take(";");
                    self.expect("}")?;
                    break;
                }
            }
            for (constructor, arm, source) in arms.into_iter().rev() {
                let sources = [arm.source, tail.source];
                tail = self.occurrence_node(
                    Term::Mat {
                        constructor,
                        arm: arm.value,
                        fallback: tail.value,
                    },
                    sources,
                );
                if let Some(origins) = &mut self.templates.diagnostics {
                    origins.node(&tail, &self.label, match_line);
                    origins.tracker.match_constructor(&tail, source);
                }
            }
            return Ok(tail);
        }
        if self.take("%") {
            return self.rewrite_atom();
        }
        if self.take("?") {
            let name = self.name()?;
            return Ok(term(Term::Hole(name)).into());
        }
        if self.take("[") {
            let mut values = if self.is("]") {
                Vec::new()
            } else {
                vec![self.expression(0)?]
            };
            if !values.is_empty() && self.take(":") {
                let ty = self.expression(12)?;
                let count = self.take("*");
                if !count {
                    self.expect("^")?;
                }
                let size = self.expression(0)?;
                self.expect("]")?;
                let depth = if count {
                    let count = nat_value(&size).ok_or_else(|| {
                        self.error("array count must be a Nat literal power of two")
                    })?;
                    if !count.is_power_of_two() {
                        return Err(self.error("array count must be a power of two"));
                    }
                    nat_term(count.ilog2()).into()
                } else {
                    size
                };
                let namespace = type_head(&ty)
                    .ok_or_else(|| self.error("array operators need a named element type"))?;
                let value = self.qualify_occurrence(values.remove(0), &namespace);
                return Ok(
                    self.applications(term(Term::Ref("Array.new".into())), [ty, depth, value])
                );
            }
            self.take(",");
            values.extend(self.arguments("]")?);
            return Ok(values
                .into_iter()
                .rev()
                .fold(self.constructor("Nil".into(), Vec::new()), |tail, head| {
                    self.constructor("Con".into(), vec![head, tail])
                }));
        }
        if self.current().text.starts_with('+') {
            return self.after_prefix('+', Self::quantified_atom);
        }
        if self
            .current()
            .text
            .starts_with(|c: char| c.is_ascii_digit())
        {
            return self.number();
        }
        if self.current().text.starts_with(['"', '\'']) {
            let raw = self.bump();
            let chars = decode_literal(&raw).map_err(|e| self.error(e))?;
            if chars.len() > 128 {
                return Err(self
                    .error("string literal exceeds the parser resource limit of 128 characters"));
            }
            if raw.starts_with('\'') {
                if chars.len() != 1 {
                    return Err(self.error("a character literal needs exactly one character"));
                }
                return Ok(term(Term::Ctr {
                    name: "Chr".into(),
                    args: vec![u32_term(u32::from(chars[0]))],
                })
                .into());
            }
            return Ok(chars
                .into_iter()
                .rev()
                .fold(
                    term(Term::Ctr {
                        name: "SNil".into(),
                        args: Vec::new(),
                    }),
                    |tail, c| {
                        term(Term::Ctr {
                            name: "SCon".into(),
                            args: vec![
                                term(Term::Ctr {
                                    name: "Chr".into(),
                                    args: vec![u32_term(u32::from(c))],
                                }),
                                tail,
                            ],
                        })
                    },
                )
                .into());
        }
        let name = self.name()?;
        if self.at > 0 && self.tokens[self.at - 1].end == self.current().start && self.take("{") {
            let args = self.arguments("}")?;
            Ok(self.constructor(self.resolve(&name), args))
        } else {
            // The pinned parser allocates an identity for an unresolved simple
            // name before it becomes a binder or a global reference. Track that
            // display event without manufacturing a semantic variable here.
            if !self.scope.iter().any(|v| v.name == name)
                && self.resolve(&name) == name
                && !name.contains('.')
                && !self.templates.declarations.contains_key(&name)
            {
                self.display_fresh += 1;
            }
            Ok(self.variable(&name, self.tokens[self.at - 1].line))
        }
    }
    fn rewrite_atom(&mut self) -> Result<Occurrence, ParseError> {
        let rewrite_line = self.tokens[self.at - 1].line;
        let binder_line = self.current().line;
        let first = self.expression(0)?;
        let (name, evidence) = if self.take("@") {
            let name = match first.as_ref() {
                Term::Ref(n) | Term::Var { name: n, .. } => n.clone(),
                _ => return Err(self.error("rewrite binder must be a name")),
            };
            (name, self.expression(0)?)
        } else {
            (String::new(), first)
        };
        self.expect(":")?;
        let old = self.scope.len();
        let x = self.fresh_variable("_".into());
        self.scope.push(x.clone());
        let e = self.open(name.clone());
        let motive = self.expression(0)?;
        self.scope.truncate(old);
        self.take(";");
        let body = self.body(self.current().column.saturating_sub(1))?;
        let body = self.flatten_body_occurrence(&body, &[])?;
        let proof_motive = self.lambda(name, e.id, motive);
        if let Some(origins) = &mut self.templates.diagnostics {
            origins.node(&proof_motive, &self.label, binder_line);
        }
        let motive = self.lambda(x.name, x.id, proof_motive);
        if let Some(origins) = &mut self.templates.diagnostics {
            origins.node(&motive, &self.label, rewrite_line);
        }
        let sources = [evidence.source, motive.source, body.source];
        Ok(self.occurrence_node(
            Term::Rwt {
                evidence: evidence.value,
                motive: motive.value,
                body: body.value,
            },
            sources,
        ))
    }

    fn quantified_atom(&mut self) -> Result<Occurrence, ParseError> {
        let value = self.expression(12)?;
        let name = type_head(&value).ok_or_else(|| self.error("+ needs a quantified datatype"))?;
        let Some(Declaration::Adt(adt)) = self
            .book
            .declarations
            .iter()
            .find(|d| matches!(d,Declaration::Adt(a) if a.name==name))
        else {
            if matches!(value.as_ref(), Term::Ref(_) | Term::Var { .. }) {
                // The + marker has a sentinel identity upstream, not a
                // fresh display number. Our semantic ID still stays unique.
                let id = *self.fresh;
                *self.fresh += 1;
                let marker = Variable { name, id };
                self.reusable_markers.insert(marker.id);
                if let Some(origins) = &self.templates.diagnostics {
                    origins.tracker.reusable_marker(marker.id);
                }
                return Ok(marker.term().into());
            }
            return Err(self.error("+ needs a quantified datatype or a binder"));
        };
        let count = adt
            .parameters
            .iter()
            .take_while(|b| matches!(b.ty.as_ref(), Term::Qnt))
            .count();
        let mut args = match value.as_ref() {
            Term::Adt { args, .. } => args.clone(),
            _ if adt.parameters.len() == count => {
                (0..count).map(|_| term(Term::Qua(Quant::Many))).collect()
            }
            _ => return Err(self.error("+ needs all datatype arguments")),
        };
        if count == 0 {
            return Err(self.error("+ needs a datatype with quantity parameters"));
        }
        for arg in args.iter_mut().take(count) {
            *arg = term(Term::Qua(Quant::Many));
        }
        let sources: Vec<_> = (0..args.len())
            .map(|index| {
                if index < count {
                    None
                } else {
                    self.templates
                        .diagnostics
                        .as_ref()
                        .and_then(|o| o.tracker.occurrence_source(&value, index))
                }
            })
            .collect();
        Ok(self.occurrence_node(
            Term::Adt {
                name,
                args,
                excluded: Vec::new(),
            },
            sources,
        ))
    }

    fn do_statement(
        &mut self,
        module: &str,
        types: &[Occurrence],
        result: Option<&Occurrence>,
        column: usize,
    ) -> Result<Occurrence, ParseError> {
        if self.depth >= 64 {
            return Err(self.error("do nesting exceeds the parser resource limit of 64"));
        }
        self.depth += 1;
        let parsed = self.do_statement_inner(module, types, result, column);
        self.depth -= 1;
        parsed
    }
    fn do_statement_inner(
        &mut self,
        module: &str,
        types: &[Occurrence],
        result: Option<&Occurrence>,
        column: usize,
    ) -> Result<Occurrence, ParseError> {
        let line = self.current().line;
        if self.take("return") {
            let value = self.expression(0)?;
            return Ok(self.do_call(&format!("{module}.pure"), types, [], result, [value], line));
        }
        let value = self.expression(0)?;
        let name = match value.as_ref() {
            Term::Ref(name) | Term::Var { name, .. } if !name.contains('.') => Some(name.clone()),
            _ => None,
        };
        let annotated = name.is_some() && self.take(":");
        let step = !annotated && (self.is(";") || !self.is("") && self.current().column == column);
        let ty = if annotated {
            self.expression(1)?
        } else if step {
            self.variable("Unit", self.current().line)
        } else {
            value.clone()
        };
        let assignment = annotated && self.take("=");
        if !assignment {
            if annotated {
                self.expect("<-")?;
            } else if !step && !self.take("<-") {
                return Ok(value);
            }
        }
        let bound = if step {
            value.clone()
        } else {
            self.expression(0)?
        };
        self.take(";");
        let old = self.scope.len();
        let reusable = annotated && self.is_reusable(&value);
        let variable = self.bind_variable(
            if annotated {
                name.expect("annotated binding has a name")
            } else {
                "_".into()
            },
            reusable,
        );
        let rebindings = std::mem::take(&mut self.reusable);
        let tail = self.do_statement(module, types, result, column)?;
        let tail = self.flatten_body_occurrence(
            &Body::Reply(self.surface_term(tail)).with_reusable(&rebindings),
            &[],
        )?;
        self.scope.truncate(old);
        if assignment {
            let value = self.annotation(bound, ty);
            let sources = [tail.source, value.source];
            Ok(self.occurrence_node(
                Term::Let {
                    bindings: vec![crate::kernel::LetBinding {
                        quant: Quant::Lone,
                        name: variable.name,
                        id: variable.id,
                        value: value.value,
                    }],
                    body: tail.value,
                },
                sources,
            ))
        } else {
            Ok(self.do_call(
                &format!("{module}.bind"),
                types,
                [ty],
                result,
                [bound, self.lambda(variable.name, variable.id, tail)],
                line,
            ))
        }
    }
    fn do_call(
        &mut self,
        name: &str,
        types: &[Occurrence],
        before: impl IntoIterator<Item = Occurrence>,
        result: Option<&Occurrence>,
        after: impl IntoIterator<Item = Occurrence>,
        line: usize,
    ) -> Occurrence {
        let arguments = types
            .iter()
            .cloned()
            .chain(before)
            .chain(result.cloned())
            .chain(after);
        let head = self.reference_at(self.resolve(name), line);
        self.applications(head, arguments)
    }
    fn number(&mut self) -> Result<Occurrence, ParseError> {
        let raw = self.bump();
        if let Some(digits) = raw.strip_suffix('n') {
            let count = digits
                .parse::<usize>()
                .map_err(|error| self.error(format!("Nat literal is too large: {error}")))?;
            if count > 128 {
                return Err(self.error("Nat literals above 128 exceed the parser's resource limit"));
            }
            // Literal sugar names the fixed constructors, like U32/F32 sugar.
            // Explicit constructor names still resolve in the local module.
            let glued = self.at > 0 && self.tokens[self.at - 1].end == self.current().start;
            // Numeric sugar consumes one adjacent '+'; the tail may itself
            // start with unary markers sharing a maximal-munch token.
            let mut value = if glued && self.current().text.starts_with('+') {
                self.after_prefix('+', |parser| parser.expression(0))?
            } else {
                self.constructor("Zero".into(), Vec::new())
            };
            for _ in 0..count {
                value = self.constructor("Succ".into(), vec![value]);
            }
            Ok(value)
        } else if raw.contains('.') {
            let value = float_literal(&raw)
                .map_err(|error| self.error(format!("invalid F32 literal: {error}")))?;
            if !value.is_finite() {
                return Err(self.error("F32 literal must be finite"));
            }
            Ok(term(Term::Ctr {
                name: "F32".into(),
                args: vec![word_term(value.to_bits())],
            })
            .into())
        } else {
            Ok(u32_term(raw.parse::<u32>().map_err(|error| {
                self.error(format!("U32 literal must be at most 4294967295: {error}"))
            })?)
            .into())
        }
    }
    fn pattern(&mut self, value: &TermRef) -> Result<Pattern, ParseError> {
        if self.is_compile_binding(value) {
            return Err(self.error("a template argument cannot be rebound as a pattern"));
        }
        match value.as_ref() {
            Term::Ref(name) | Term::Var { name, .. } => {
                if self.book.declarations.iter().any(|d|matches!(d,Declaration::Adt(a) if a.constructors.iter().any(|c|c.name==self.resolve(name)))) {return Err(self.error(format!("constructor pattern {name} requires braces")));}
                if name.contains('.') || name.contains('/') {
                    return Err(self.error("a pattern binder must be one unqualified name"));
                }
                let reusable = self.is_reusable(value);
                let reusable_start = self.reusable.len();
                let variable = self.bind_variable(name.clone(), reusable);
                if let Some(origins) = &self.templates.diagnostics {
                    // Flattening closes pattern fields with generated lambdas.
                    // Keep the pattern's span on that fresh binder, without
                    // changing the parsed reference or its shared identity.
                    let source = origins.tracker.source(value);
                    origins.tracker.binder_source(variable.id, source);
                    for (binding, _) in &self.reusable[reusable_start..] {
                        origins.tracker.binder_source(binding.id, source);
                    }
                }
                Ok(Pattern::Variable(variable))
            }
            Term::Ctr { name, args } => {
                let constructor = self.book.declarations.iter().find_map(|d| match d {
                    Declaration::Adt(a) => a.constructors.iter().find(|c| c.name == *name),
                    Declaration::Def(_) => None,
                });
                let Some(c) = constructor else {
                    if let Some(origins) = &mut self.templates.diagnostics {
                        origins.unknown_pattern(name, value);
                    }
                    return Err(self.error(format!("unknown constructor pattern: {name}")));
                };
                if c.fields.len() != args.len() {
                    return Err(self.error(format!(
                        "constructor {name} expects {} fields",
                        c.fields.len()
                    )));
                }
                let fields = args
                    .iter()
                    .map(|a| self.pattern(a))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Pattern::Constructor(
                    name.clone(),
                    fields,
                    self.templates.diagnostics.as_ref().map_or_else(
                        || Rc::as_ptr(value) as usize,
                        |origins| origins.tracker.source(value),
                    ),
                ))
            }
            _ => {
                if let Some(origins) = &mut self.templates.diagnostics {
                    origins.unsupported_pattern(value);
                }
                Err(self.error("a pattern must be a binder or constructor"))
            }
        }
    }
    fn surface_term(&self, value: Occurrence) -> SurfaceTerm {
        SurfaceTerm::occurrence(
            value,
            self.templates.diagnostics.as_ref().map(|o| &o.tracker),
        )
    }
    fn scrutinees(&self, values: Vec<Occurrence>) -> Vec<SurfaceTerm> {
        let origins = self.templates.diagnostics.as_ref().map(|o| &o.tracker);
        values
            .into_iter()
            .map(|value| SurfaceTerm::occurrence(value, origins))
            .collect()
    }
    fn terms_before_colon(&mut self) -> Result<Vec<Occurrence>, ParseError> {
        let mut terms = Vec::new();
        loop {
            let line = self.current().line;
            let value = self.expression(0)?;
            if let Some(origins) = &mut self.templates.diagnostics {
                origins.node(&value, &self.label, line);
            }
            terms.push(value);
            if self.take(":") {
                return Ok(terms);
            }
            self.take(",");
        }
    }
    fn body(&mut self, column: usize) -> Result<Body, ParseError> {
        if self.depth >= 64 {
            return Err(self.error("body nesting exceeds the parser resource limit of 64"));
        }
        self.depth += 1;
        let reusable = std::mem::take(&mut self.reusable);
        let result = self
            .body_inner(column)
            .map(|body| body.with_reusable(&reusable));
        self.depth -= 1;
        result
    }
    fn match_scrutinees(&mut self, line: usize) -> Result<Vec<SurfaceTerm>, ParseError> {
        let scrutinees = self.terms_before_colon()?;
        if let Some(origins) = &mut self.templates.diagnostics {
            for value in &scrutinees {
                origins.match_location(value, &self.label, line);
            }
        }
        Ok(self
            .scrutinees(scrutinees)
            .into_iter()
            .map(SurfaceTerm::statement_scrutinee)
            .collect())
    }
    fn body_inner(&mut self, column: usize) -> Result<Body, ParseError> {
        let match_line = self.current().line;
        if self.take("match") {
            let scrutinees = self.match_scrutinees(match_line)?;
            let case_column = self.current().column;
            let mut rows = Vec::new();
            while case_column > column && self.is("case") && self.current().column >= case_column {
                let row_column = self.current().column;
                self.bump();
                let patterns = self.terms_before_colon()?;
                if patterns.len() != scrutinees.len() {
                    let colon = &self.tokens[self.at - 1];
                    if let Some(origins) = &mut self.templates.diagnostics {
                        origins.row_arity(&self.label, colon.line, colon.column, scrutinees.len());
                    }
                    return Err(self.error("a match needs one pattern per scrutinee"));
                }
                let old = self.scope.len();
                let patterns = patterns
                    .iter()
                    .map(|p| self.pattern(p))
                    .collect::<Result<Vec<_>, _>>()?;
                let body = self.body(row_column)?;
                self.scope.truncate(old);
                rows.push(Row { patterns, body });
            }
            return Ok(Body::Match { scrutinees, rows });
        }
        let start = self.at;
        let mut quant = self.quant();
        let mut patterns = Vec::new();
        let mut values = Vec::new();
        if quant != Quant::Lone && self.tokens.get(self.at + 1).is_some_and(|t| t.text == "=") {
            let name = self.name()?;
            patterns.push(term(Term::Ref(name)).into());
            self.expect("=")?;
        } else {
            self.at = start;
            quant = Quant::Lone;
            patterns.push(self.expression(0)?);
            while self.at > 0
                && self.tokens[self.at - 1].line == self.current().line
                && (self.is("+")
                    || self
                        .current()
                        .text
                        .starts_with(|c: char| c.is_ascii_alphabetic() || c == '_'))
                && !KEYWORDS.contains(&self.current().text.as_str())
            {
                patterns.push(self.expression(0)?);
            }
            if patterns.len() == 1 && !self.is("=") {
                let more = self.is(";")
                    || !self.is("") && self.current().column == self.tokens[start].column;
                if let Some(variable) = array_write_binder(&patterns[0]).filter(|_| more) {
                    values.push(patterns.remove(0));
                    patterns.push(variable.into());
                } else {
                    return Ok(Body::Reply(self.surface_term(patterns.remove(0))));
                }
            } else {
                self.expect("=")?;
            }
        }
        while values.len() < patterns.len() {
            let line = self.current().line;
            let value = self.expression(0)?;
            if let Some(origins) = &mut self.templates.diagnostics {
                origins.node(&value, &self.label, line);
            }
            values.push(value);
        }
        self.take(";");
        let old = self.scope.len();
        let patterns = patterns
            .iter()
            .map(|p| self.pattern(p))
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(origins) = &mut self.templates.diagnostics {
            for pattern in &patterns {
                if let Pattern::Variable(variable) = pattern {
                    origins.local_binder(variable.id, &self.label, self.tokens[start].line);
                }
            }
        }
        let body = self.body(column)?;
        self.scope.truncate(old);
        Ok(Body::Local {
            patterns,
            quant,
            values: self.scrutinees(values),
            body: Box::new(body),
        })
    }
}

fn array_write_binder(value: &TermRef) -> Option<TermRef> {
    let Term::App(set_with_index, _) = value.as_ref() else {
        return None;
    };
    let Term::App(set_with_array, _) = set_with_index.as_ref() else {
        return None;
    };
    let Term::App(set_with_type, array) = set_with_array.as_ref() else {
        return None;
    };
    let Term::App(head, _) = set_with_type.as_ref() else {
        return None;
    };
    (matches!(head.as_ref(), Term::Ref(name) | Term::GpuRef(name) if name == "Array.set")
        && matches!(array.as_ref(), Term::Var { .. }))
    .then(|| Rc::clone(array))
}

fn operator(op: &str) -> Option<(u8, bool, &'static str)> {
    Some(match op {
        "&" => (1, true, "Pair"),
        "|" => (1, true, "Or"),
        "||" => (2, false, "Bool.or"),
        "&&" => (3, false, "Bool.and"),
        "<=" => (4, false, ".is_le"),
        ">=" => (4, false, ".is_ge"),
        ">" => (4, false, ".is_gt"),
        "<>" | "<&>" => (5, true, ""),
        "++" => (5, true, "String.append"),
        ".|." => (6, false, ".or"),
        ".^." => (7, false, ".xor"),
        ".&." => (8, false, ".and"),
        "<<" => (9, false, ".shln"),
        ">>" => (9, false, ".shrn"),
        "+" => (10, false, ".add"),
        "-" => (10, false, ".sub"),
        "*" => (11, false, ".mul"),
        "/" => (11, false, ".div"),
        "%" => (11, false, ".mod"),
        _ => return None,
    })
}
fn apply(head: TermRef, args: impl IntoIterator<Item = TermRef>) -> TermRef {
    args.into_iter().fold(head, |f, x| term(Term::App(f, x)))
}
fn type_head(ty: &TermRef) -> Option<String> {
    match ty.as_ref() {
        Term::Ref(n) | Term::GpuRef(n) | Term::Var { name: n, .. } | Term::Adt { name: n, .. } => {
            Some(n.clone())
        }
        Term::App(f, _) => type_head(f),
        _ => None,
    }
}
#[expect(
    clippy::cast_possible_truncation,
    reason = "Pinned parse_term_num uses Math.fround(Number(token)): binary64 then binary32"
)]
fn float_literal(raw: &str) -> Result<f32, std::num::ParseFloatError> {
    raw.parse::<f64>().map(|value| value as f32)
}

fn word_term(value: u32) -> TermRef {
    let mut out = term(Term::Ctr {
        name: "WNil".into(),
        args: Vec::new(),
    });
    for bit in (0..32).rev() {
        out = term(Term::Ctr {
            name: "WCon".into(),
            args: vec![
                term(Term::Ctr {
                    name: if value & (1 << bit) == 0 {
                        "False"
                    } else {
                        "True"
                    }
                    .into(),
                    args: Vec::new(),
                }),
                out,
            ],
        });
    }
    out
}
fn u32_term(value: u32) -> TermRef {
    term(Term::Ctr {
        name: "U32".into(),
        args: vec![word_term(value)],
    })
}
fn nat_value(value: &TermRef) -> Option<u32> {
    let mut value = value;
    let mut count = 0;
    loop {
        match value.as_ref() {
            Term::Ctr { name, args } if name == "Zero" && args.is_empty() => return Some(count),
            Term::Ctr { name, args } if name == "Succ" && args.len() == 1 => {
                count += 1;
                value = &args[0];
            }
            _ => return None,
        }
    }
}
fn nat_term(count: u32) -> TermRef {
    (0..count).fold(
        term(Term::Ctr {
            name: "Zero".into(),
            args: Vec::new(),
        }),
        |tail, _| {
            term(Term::Ctr {
                name: "Succ".into(),
                args: vec![tail],
            })
        },
    )
}
fn decode_literal(raw: &str) -> Result<Vec<char>, String> {
    let mut chars = raw[1..raw.len() - 1].chars();
    let mut out = Vec::new();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        let escape = chars.next().ok_or("unfinished escape")?;
        out.push(match escape {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            '0' => '\0',
            '\\' => '\\',
            '\'' => '\'',
            '"' => '"',
            'u' => {
                if chars.next() != Some('{') {
                    return Err("Unicode escape needs braces".into());
                }
                let mut digits = String::new();
                let mut closed = false;
                for d in chars.by_ref() {
                    if d == '}' {
                        closed = true;
                        break;
                    }
                    digits.push(d);
                }
                if !closed {
                    return Err("unfinished Unicode escape".into());
                }
                char::from_u32(
                    u32::from_str_radix(&digits, 16)
                        .map_err(|error| format!("invalid Unicode escape: {error}"))?,
                )
                .ok_or("invalid Unicode code point")?
            }
            _ => return Err(format!("unknown escape: \\{escape}")),
        });
    }
    Ok(out)
}

fn within_term_depth(value: &TermRef, limit: usize) -> bool {
    let mut pending = vec![(Rc::clone(value), 0)];
    while let Some((value, depth)) = pending.pop() {
        if depth > limit {
            return false;
        }
        let mut push = |child: &TermRef| pending.push((Rc::clone(child), depth + 1));
        match value.as_ref() {
            Term::Typ(child) => push(child),
            Term::Min(left, right) | Term::App(left, right) | Term::Ann(left, right) => {
                push(left);
                push(right);
            }
            Term::All { domain, body, .. } => {
                push(domain);
                push(body);
            }
            Term::Lam { body, .. } => push(body),
            Term::Ctr { args, .. } | Term::Adt { args, .. } => {
                for arg in args {
                    push(arg);
                }
            }
            Term::Mat { arm, fallback, .. } => {
                push(arm);
                push(fallback);
            }
            Term::Eql { left, right, ty } => {
                push(left);
                push(right);
                push(ty);
            }
            Term::Rwt {
                evidence,
                motive,
                body,
            } => {
                push(evidence);
                push(motive);
                push(body);
            }
            Term::Let { bindings, body } => {
                for binding in bindings {
                    push(&binding.value);
                }
                push(body);
            }
            _ => {}
        }
    }
    true
}

fn qualify_operators(value: &TermRef, namespace: &str) -> TermRef {
    let go = |t: &TermRef| qualify_operators(t, namespace);
    match value.as_ref() {
        // Relative module identities may begin with ../ or a dot-directory.
        // Only parser-generated operator placeholders need a type namespace.
        Term::Ref(name) | Term::GpuRef(name)
            if matches!(
                name.as_str(),
                ".add"
                    | ".sub"
                    | ".mul"
                    | ".div"
                    | ".mod"
                    | ".is_lt"
                    | ".is_le"
                    | ".is_gt"
                    | ".is_ge"
                    | ".or"
                    | ".xor"
                    | ".and"
                    | ".shln"
                    | ".shrn"
            ) =>
        {
            term(if matches!(value.as_ref(), Term::GpuRef(_)) {
                Term::GpuRef(format!("{namespace}{name}"))
            } else {
                Term::Ref(format!("{namespace}{name}"))
            })
        }
        Term::Typ(t) => term(Term::Typ(go(t))),
        Term::Min(a, b) => term(Term::Min(go(a), go(b))),
        Term::All {
            quant,
            name,
            id,
            domain,
            body,
        } => term(Term::All {
            quant: *quant,
            name: name.clone(),
            id: *id,
            domain: go(domain),
            body: go(body),
        }),
        Term::Lam { name, id, body } => term(Term::Lam {
            name: name.clone(),
            id: *id,
            body: go(body),
        }),
        Term::App(a, b) => term(Term::App(go(a), go(b))),
        Term::Adt {
            name,
            args,
            excluded,
        } => term(Term::Adt {
            name: name.clone(),
            args: args.iter().map(go).collect(),
            excluded: excluded.clone(),
        }),
        Term::Ctr { name, args } => crate::kernel::map_constructor(value, name, args, go),
        Term::Mat {
            constructor,
            arm,
            fallback,
        } => term(Term::Mat {
            constructor: constructor.clone(),
            arm: go(arm),
            fallback: go(fallback),
        }),
        Term::Eql { left, right, ty } => term(Term::Eql {
            left: go(left),
            right: go(right),
            ty: go(ty),
        }),
        Term::Rwt {
            evidence,
            motive,
            body,
        } => term(Term::Rwt {
            evidence: go(evidence),
            motive: go(motive),
            body: go(body),
        }),
        Term::Ann(a, b) => term(Term::Ann(go(a), go(b))),
        Term::Let { bindings, body } => term(Term::Let {
            bindings: bindings
                .iter()
                .map(|b| crate::kernel::LetBinding {
                    quant: b.quant,
                    name: b.name.clone(),
                    id: b.id,
                    value: go(&b.value),
                })
                .collect(),
            body: go(body),
        }),
        _ => Rc::clone(value),
    }
}

// Structural keys distinguish references from variables and constructor names
// from syntax. Bound IDs are represented by lexical positions, so independently
// parsed copies of a closed lambda share an instance without capturing locals.
#[derive(Default)]
struct TemplateKey {
    text: String,
    nodes: usize,
}

impl TemplateKey {
    fn write(&mut self, text: &str) -> Result<(), &'static str> {
        if self.text.len() + text.len() > MAX_TEMPLATE_KEY_BYTES {
            return Err("template argument key exceeds the resource limit of 2048 bytes");
        }
        self.text.push_str(text);
        Ok(())
    }
    fn name(&mut self, tag: &str, name: &str) -> Result<(), &'static str> {
        self.write(tag)?;
        self.write(&name.len().to_string())?;
        self.write(":")?;
        self.write(name)
    }
    fn terms(
        &mut self,
        values: &[TermRef],
        scope: &[usize],
        depth: usize,
    ) -> Result<bool, &'static str> {
        self.write("[")?;
        for value in values {
            if !self.term(value, scope, depth + 1)? {
                return Ok(false);
            }
        }
        self.write("]")?;
        Ok(true)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "an explicit key variant for every kernel term prevents accidental cache collisions"
    )]
    fn term(
        &mut self,
        value: &TermRef,
        scope: &[usize],
        depth: usize,
    ) -> Result<bool, &'static str> {
        if depth > 128 || self.nodes >= 4096 {
            return Err("template argument structure exceeds the key resource limit");
        }
        self.nodes += 1;
        self.write("(")?;
        match value.as_ref() {
            Term::Var { id, .. } => {
                let Some(index) = scope.iter().rev().position(|binder| binder == id) else {
                    return Ok(false);
                };
                self.name("v", &index.to_string())?;
            }
            Term::Ref(name) => self.name("r", name)?,
            Term::GpuRef(name) => self.name("r!", name)?,
            Term::Typ(grade) => {
                self.write("t")?;
                if !self.term(grade, scope, depth + 1)? {
                    return Ok(false);
                }
            }
            Term::Qnt => self.write("q")?,
            Term::Qua(quant) => self.name("g", &quant.to_string())?,
            Term::Min(a, b) | Term::App(a, b) | Term::Ann(a, b) => {
                self.write(match value.as_ref() {
                    Term::Min(_, _) => "m",
                    Term::App(_, _) => "a",
                    _ => "n",
                })?;
                if !self.term(a, scope, depth + 1)? || !self.term(b, scope, depth + 1)? {
                    return Ok(false);
                }
            }
            Term::All {
                quant,
                id,
                domain,
                body,
                ..
            } => {
                self.name("f", &quant.to_string())?;
                if !self.term(domain, scope, depth + 1)? {
                    return Ok(false);
                }
                let mut nested = scope.to_vec();
                nested.push(*id);
                if !self.term(body, &nested, depth + 1)? {
                    return Ok(false);
                }
            }
            Term::Lam { id, body, .. } => {
                self.write("l")?;
                let mut nested = scope.to_vec();
                nested.push(*id);
                if !self.term(body, &nested, depth + 1)? {
                    return Ok(false);
                }
            }
            Term::Adt {
                name,
                args,
                excluded,
            } => {
                self.name("d", name)?;
                if !self.terms(args, scope, depth)? {
                    return Ok(false);
                }
                for name in excluded {
                    self.name("x", name)?;
                }
            }
            Term::Ctr { name, args } => {
                self.name("c", name)?;
                if !self.terms(args, scope, depth)? {
                    return Ok(false);
                }
            }
            Term::Mat {
                constructor,
                arm,
                fallback,
            } => {
                self.name("b", constructor)?;
                if !self.term(arm, scope, depth + 1)? || !self.term(fallback, scope, depth + 1)? {
                    return Ok(false);
                }
            }
            Term::Efq => self.write("e")?,
            Term::Eql { left, right, ty } => {
                self.write("=")?;
                if !self.term(left, scope, depth + 1)?
                    || !self.term(right, scope, depth + 1)?
                    || !self.term(ty, scope, depth + 1)?
                {
                    return Ok(false);
                }
            }
            Term::Rfl => self.write("p")?,
            Term::Rwt {
                evidence,
                motive,
                body,
            } => {
                self.write("w")?;
                if !self.term(evidence, scope, depth + 1)?
                    || !self.term(motive, scope, depth + 1)?
                    || !self.term(body, scope, depth + 1)?
                {
                    return Ok(false);
                }
            }
            Term::Hole(name) => self.name("h", name)?,
            Term::Let { bindings, body } => {
                self.write("s[")?;
                for binding in bindings {
                    self.name("g", &binding.quant.to_string())?;
                    if !self.term(&binding.value, scope, depth + 1)? {
                        return Ok(false);
                    }
                }
                self.write("]")?;
                let mut nested = scope.to_vec();
                nested.extend(bindings.iter().map(|binding| binding.id));
                if !self.term(body, &nested, depth + 1)? {
                    return Ok(false);
                }
            }
        }
        self.write(")")?;
        Ok(true)
    }
}

fn template_key(arguments: &[TermRef]) -> Result<Option<String>, &'static str> {
    let mut key = TemplateKey::default();
    if key.terms(arguments, &[], 0)? {
        Ok(Some(key.text))
    } else {
        Ok(None)
    }
}

/// Parse a self-contained Bend source book. Local imports require [`load`].
///
/// # Errors
/// Returns a located error for malformed or currently unsupported syntax.
pub fn parse(source: &str) -> Result<Book, ParseError> {
    let mut book = Book::default();
    let mut fresh = 0;
    parse_into(
        ParseInput {
            source,
            label: "<input>",
            namespace: "",
            aliases: BTreeMap::new(),
            origin: SourceOrigin::Standalone,
        },
        &mut book,
        &mut fresh,
        &mut Templates::default(),
        None,
        None,
    )?;
    Ok(book)
}

/// Parse one expression, rejecting any trailing input.
///
/// # Errors
/// Returns a located error for malformed or unsupported syntax.
pub fn parse_term(source: &str) -> Result<TermRef, ParseError> {
    let mut book = Book::default();
    let mut fresh = 0;
    let mut templates = Templates::default();
    let mut p = Parser {
        tokens: lex(source, "<expression>")?,
        at: 0,
        label: "<expression>".into(),
        namespace: String::new(),
        aliases: BTreeMap::new(),
        scope: Vec::new(),
        fresh: &mut fresh,
        display_fresh: 0,
        book: &mut book,
        depth: 0,
        reusable: Vec::new(),
        reusable_markers: BTreeSet::new(),
        templates: &mut templates,
        instance_name: None,
        instance_arguments: VecDeque::new(),
        compile_bindings: BTreeMap::new(),
        foreign: None,
        constructor_tags: None,
        origin: SourceOrigin::Standalone,
    };
    let value = p.expression(0)?;
    if !p.is("") {
        return Err(p.error("unexpected trailing input"));
    }
    Ok(qualify_operators(&value, "Nat"))
}

fn parse_into(
    input: ParseInput<'_>,
    book: &mut Book,
    fresh: &mut usize,
    templates: &mut Templates,
    foreign: Option<&mut BTreeMap<String, ForeignDefinition>>,
    constructor_tags: Option<&mut BTreeMap<String, String>>,
) -> Result<(), ParseError> {
    if let Some(origins) = &mut templates.diagnostics {
        origins.source(input.label, input.source);
    }
    Parser {
        tokens: lex(input.source, input.label)?,
        at: 0,
        label: input.label.into(),
        namespace: input.namespace.into(),
        aliases: input.aliases,
        scope: Vec::new(),
        fresh,
        display_fresh: 0,
        book,
        depth: 0,
        reusable: Vec::new(),
        reusable_markers: BTreeSet::new(),
        templates,
        instance_name: None,
        instance_arguments: VecDeque::new(),
        compile_bindings: BTreeMap::new(),
        foreign,
        constructor_tags,
        origin: input.origin,
    }
    .declarations()
}

#[derive(Default)]
struct Loader {
    book: Book,
    fresh: usize,
    seen: BTreeMap<PathBuf, String>,
    active: BTreeSet<PathBuf>,
    base: bool,
    templates: Templates,
    executable: bool,
    foreign: BTreeMap<String, ForeignDefinition>,
    numeric: BTreeMap<String, NumericIntrinsic>,
    opaque: BTreeMap<String, OpaqueType>,
    constructor_tags: BTreeMap<String, String>,
    base_names: BTreeSet<String>,
}

impl Loader {
    #[expect(
        clippy::too_many_lines,
        reason = "Import validation and graph updates are one ordered transaction"
    )]
    fn file(&mut self, path: &Path, namespace: &str) -> Result<(), ParseError> {
        let error = |message: String| ParseError {
            source: path.display().to_string(),
            line: 1,
            column: 1,
            message,
        };
        let real = fs::canonicalize(path).map_err(|e| error(e.to_string()))?;
        if self.active.len() >= 64 {
            return Err(error(
                "import nesting exceeds the loader resource limit of 64".into(),
            ));
        }
        if self.active.contains(&real) {
            return Err(error("import cycle".into()));
        }
        if let Some(prior) = self.seen.get(&real) {
            if prior != namespace {
                return Err(error(format!(
                    "one namespace per file: {prior:?} and {namespace:?}"
                )));
            }
            return Ok(());
        }
        self.active.insert(real.clone());
        let source = fs::read_to_string(&real).map_err(|e| error(e.to_string()))?;
        let mut display_lines = self
            .templates
            .diagnostics
            .as_ref()
            .map(|_| source.split('\n').collect::<Vec<_>>());
        let mut aliases = BTreeMap::new();
        let mut cleaned = String::new();
        let mut header = true;
        for (line_number, line) in source.lines().enumerate() {
            let text = line.split('#').next().unwrap_or("").trim();
            if header
                && (text == "import"
                    || text.starts_with("import") && text[6..].starts_with(char::is_whitespace))
            {
                let import_failure = |message: String| ParseError {
                    source: path.display().to_string(),
                    line: line_number + 1,
                    column: line.find("import").unwrap_or(0) + 1,
                    message,
                };
                let parts = text.split_whitespace().collect::<Vec<_>>();
                if parts == ["import", "Base"] {
                    if !self.base {
                        self.base = true;
                        self.embedded_base()?;
                    }
                } else if let ["import", relative, "as", alias] = parts.as_slice() {
                    if Path::new(relative)
                        .extension()
                        .is_none_or(|extension| extension != "bend")
                        || alias.is_empty()
                        || !alias.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                        || !alias.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    {
                        return Err(import_failure(
                            "expected a .bend path and a simple alias".into(),
                        ));
                    }
                    if relative.starts_with("0x") {
                        return Err(import_failure(
                            "hub package imports are not supported; use a local .bend module"
                                .into(),
                        ));
                    }
                    if aliases.contains_key(*alias) {
                        return Err(import_failure(format!("duplicate import alias: {alias}")));
                    }
                    let import_path = Path::new(relative);
                    let at = path
                        .parent()
                        .unwrap_or_else(|| Path::new("."))
                        .join(import_path);
                    let sub = if import_path.is_absolute() {
                        normalize_namespace(relative)
                    } else {
                        let parent = namespace.rsplit_once('/').map_or("", |(p, _)| p);
                        normalize_namespace(&format!("{parent}/{relative}"))
                    };
                    let sub = sub
                        .strip_suffix(".bend")
                        .expect("extension checked")
                        .to_owned();
                    aliases.insert((*alias).into(), sub.clone());
                    self.file(&at, &sub)?;
                } else {
                    return Err(import_failure(
                        "expected 'import Base' or 'import path.bend as Name'".into(),
                    ));
                }
                cleaned.push('\n');
                if let Some(lines) = &mut display_lines {
                    lines[line_number] = "";
                }
            } else {
                if !text.is_empty() {
                    header = false;
                }
                cleaned.push_str(line);
                cleaned.push('\n');
            }
        }
        if let Some(origins) = &mut self.templates.diagnostics {
            // Upstream removes consumed import headers from diagnostic source.
            // Preserve other original bytes, including CRLF and the final line;
            // the parser's cleaned buffer has different newline normalization.
            origins.source(
                &path.display().to_string(),
                &display_lines.expect("enabled origins").join("\n"),
            );
        }
        parse_into(
            ParseInput {
                source: &cleaned,
                label: &path.display().to_string(),
                namespace,
                aliases,
                origin: SourceOrigin::Local(
                    real.parent().unwrap_or_else(|| Path::new(".")).to_owned(),
                ),
            },
            &mut self.book,
            &mut self.fresh,
            &mut self.templates,
            self.executable.then_some(&mut self.foreign),
            self.executable.then_some(&mut self.constructor_tags),
        )?;
        self.active.remove(&real);
        self.seen.insert(real, namespace.into());
        Ok(())
    }

    fn embedded_base(&mut self) -> Result<(), ParseError> {
        let start = self.book.declarations.len();
        parse_into(
            ParseInput {
                source: include_str!("base.bend"),
                label: "<Base>",
                namespace: "",
                aliases: BTreeMap::new(),
                origin: SourceOrigin::Bundled,
            },
            &mut self.book,
            &mut self.fresh,
            &mut self.templates,
            None,
            self.executable.then_some(&mut self.constructor_tags),
        )?;
        if self.executable {
            parse_into(
                ParseInput {
                    source: include_str!("executable-base.bend"),
                    label: "<Base effects>",
                    namespace: "",
                    aliases: BTreeMap::new(),
                    origin: SourceOrigin::Bundled,
                },
                &mut self.book,
                &mut self.fresh,
                &mut self.templates,
                Some(&mut self.foreign),
                Some(&mut self.constructor_tags),
            )?;
            for declaration in &self.book.declarations[start..] {
                let name = match declaration {
                    Declaration::Def(definition) => &definition.name,
                    Declaration::Adt(datatype) => &datatype.name,
                };
                self.base_names.insert(name.clone());
                if let Some(intrinsic) = NumericIntrinsic::bundled(name) {
                    self.numeric.insert(name.clone(), intrinsic);
                }
                if let Some(opaque) = OpaqueType::bundled(name) {
                    self.opaque.insert(name.clone(), opaque);
                }
            }
        }
        Ok(())
    }
}

fn lexical_path(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir
                if matches!(result.components().next_back(), Some(Component::Normal(_))) =>
            {
                result.pop();
            }
            _ => result.push(component.as_os_str()),
        }
    }
    result
}

fn normalize_namespace(path: &str) -> String {
    let mut parts = Vec::new();
    for part in Path::new(path.trim_start_matches('/')).components() {
        match part {
            Component::ParentDir => {
                if parts.last().is_some_and(|p| *p != "..") {
                    parts.pop();
                } else {
                    parts.push("..".to_owned());
                }
            }
            Component::Normal(p) => parts.push(p.to_string_lossy().into_owned()),
            _ => {}
        }
    }
    parts.join("/")
}

/// Read a local Bend module and its complete import graph.
///
/// `import Base` loads the bundled, explicitly limited foundational library.
/// Cycles, alias collisions, missing files and unsupported hub imports fail.
///
/// # Errors
/// Returns a located error on file access, import resolution or parsing failure.
pub fn load(path: impl AsRef<Path>) -> Result<Book, ParseError> {
    load_with_sources(path).map(|(book, _)| book)
}

pub(crate) fn load_with_diagnostics(
    path: impl AsRef<Path>,
) -> Result<(Book, super::diagnostics::DiagnosticOrigins), super::diagnostics::DiagnosticLoadError>
{
    let mut loader = Loader {
        templates: Templates {
            diagnostics: Some(super::diagnostics::DiagnosticOrigins::default()),
            ..Templates::default()
        },
        ..Loader::default()
    };
    if let Err(error) = loader.file(path.as_ref(), "") {
        return Err(loader
            .templates
            .diagnostics
            .as_mut()
            .expect("enabled origins")
            .load_failure(error));
    }
    Ok((
        loader.book,
        loader.templates.diagnostics.expect("enabled origins"),
    ))
}

/// Read a local module and return the canonical files consumed by its loader.
///
/// The embedded Base library has no filesystem path in this list.
///
/// # Errors
/// Returns the same file, import and parse errors as [`load`].
pub fn load_with_sources(path: impl AsRef<Path>) -> Result<(Book, Vec<PathBuf>), ParseError> {
    let mut loader = Loader::default();
    loader.file(path.as_ref(), "")?;
    Ok((loader.book, loader.seen.into_keys().collect()))
}

/// Read a local program with execution-only Base and retained foreign contracts.
///
/// The result is not a checked proof book. Foreign files are recorded but never
/// executed by loading; an execution checker must validate every signature.
///
/// # Errors
/// Returns a located error on Bend file access, import resolution or parsing failure.
pub fn load_executable(path: impl AsRef<Path>) -> Result<ExecutableSource, ParseError> {
    load_executable_with_sources(path).map(|(source, _)| source)
}

pub(crate) fn load_executable_with_diagnostics(
    path: impl AsRef<Path>,
) -> Result<
    (ExecutableSource, super::diagnostics::DiagnosticOrigins),
    super::diagnostics::DiagnosticLoadError,
> {
    load_executable_with_diagnostics_and_sources(path).map(|(source, origins, _)| (source, origins))
}

pub(crate) fn load_executable_with_diagnostics_and_sources(
    path: impl AsRef<Path>,
) -> Result<
    (
        ExecutableSource,
        super::diagnostics::DiagnosticOrigins,
        Vec<PathBuf>,
    ),
    super::diagnostics::DiagnosticLoadError,
> {
    let mut loader = Loader {
        executable: true,
        templates: Templates {
            diagnostics: Some(super::diagnostics::DiagnosticOrigins::default()),
            ..Templates::default()
        },
        ..Loader::default()
    };
    if let Err(error) = loader.file(path.as_ref(), "") {
        return Err(loader
            .templates
            .diagnostics
            .as_mut()
            .expect("enabled origins")
            .load_failure(error));
    }
    let source = ExecutableSource {
        book: loader.book,
        foreign: loader.foreign,
        numeric: loader.numeric,
        opaque: loader.opaque,
        constructor_tags: loader.constructor_tags,
        base_names: loader.base_names,
    };
    Ok((
        source,
        loader.templates.diagnostics.expect("enabled origins"),
        loader.seen.into_keys().collect(),
    ))
}

/// Read execution contracts and return their canonical Bend source paths.
///
/// Foreign implementation paths remain available on the checked executable
/// book. This list describes Bend files actually read, including imported modules.
///
/// # Errors
/// Returns the same file, import and parse errors as [`load_executable`].
pub fn load_executable_with_sources(
    path: impl AsRef<Path>,
) -> Result<(ExecutableSource, Vec<PathBuf>), ParseError> {
    let mut loader = Loader {
        executable: true,
        ..Loader::default()
    };
    loader.file(path.as_ref(), "")?;
    let source = ExecutableSource {
        book: loader.book,
        foreign: loader.foreign,
        numeric: loader.numeric,
        opaque: loader.opaque,
        constructor_tags: loader.constructor_tags,
        base_names: loader.base_names,
    };
    Ok((source, loader.seen.into_keys().collect()))
}

#[cfg(test)]
mod occurrence_tests {
    use super::*;

    #[test]
    fn nested_prefix_errors_restore_complete_tokens_for_replay() {
        for source in ["++", "++++:", "1n++++", "1n++++:"] {
            let mut book = Book::default();
            let mut fresh = 0;
            let mut templates = Templates::default();
            let mut p = Parser {
                tokens: lex(source, "<expression>").unwrap(),
                at: 0,
                label: "<expression>".into(),
                namespace: String::new(),
                aliases: BTreeMap::new(),
                scope: Vec::new(),
                fresh: &mut fresh,
                display_fresh: 0,
                book: &mut book,
                depth: 0,
                reusable: Vec::new(),
                reusable_markers: BTreeSet::new(),
                templates: &mut templates,
                instance_name: None,
                instance_arguments: VecDeque::new(),
                compile_bindings: BTreeMap::new(),
                foreign: None,
                constructor_tags: None,
                origin: SourceOrigin::Standalone,
            };
            let before = format!("{:?}", p.tokens);
            p.expression(0).expect_err("malformed tail");
            assert_eq!(format!("{:?}", p.tokens), before);
        }
    }

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/cpu-template-occurrences")
    }

    #[test]
    fn per_use_sources_keep_shared_reference_identity_and_ownership() {
        let path = root().join("positive/shared-value.bend");
        let (book, origins) = load_with_diagnostics(&path).unwrap();
        let plain = load(&path).unwrap();
        assert_eq!(format!("{book:?}"), format!("{plain:?}"));
        let Declaration::Def(pack) = book
            .declarations
            .iter()
            .find(|d| matches!(d, Declaration::Def(d) if d.name == "pack~0"))
            .unwrap()
        else {
            panic!("pack");
        };
        let Term::Lam { body, .. } = pack.body.as_ref().unwrap().as_ref() else {
            panic!("compile binder");
        };
        let Term::Ctr { args, .. } = body.as_ref() else {
            panic!("three shared uses");
        };
        assert_eq!(args.len(), 3);
        assert!(matches!(args[0].as_ref(), Term::Ref(name) if name == "zero"));
        assert!(args.iter().all(|arg| Rc::ptr_eq(arg, &args[0])));
        let Declaration::Def(main) = book
            .declarations
            .iter()
            .find(|d| matches!(d, Declaration::Def(d) if d.name == "main"))
            .unwrap()
        else {
            panic!("main");
        };
        let Term::App(_, argument) = main.body.as_ref().unwrap().as_ref() else {
            panic!("call");
        };
        assert!(Rc::ptr_eq(argument, &args[0]));
        assert_eq!(Rc::strong_count(argument), 4);
        let sources: Vec<_> = (0..3)
            .map(|i| origins.tracker.occurrence_source(body, i).unwrap())
            .collect();
        assert_eq!(sources.iter().collect::<BTreeSet<_>>().len(), 3);
        assert!(
            sources
                .iter()
                .all(|source| *source != origins.tracker.source(argument))
        );
        drop(origins);
        assert_eq!(Rc::strong_count(argument), 4);
        crate::kernel::check_book(&book)
            .unwrap()
            .evaluate_data("main", &[])
            .unwrap();
    }

    #[test]
    fn occurrence_metadata_does_not_split_template_cache_keys() {
        let path = root().join("positive/macro-cache.bend");
        let (book, origins) = load_with_diagnostics(&path).unwrap();
        let plain = load(&path).unwrap();
        assert_eq!(format!("{book:?}"), format!("{plain:?}"));
        for prefix in ["inner~", "outer~", "macro~"] {
            let names: Vec<_> = book
                .declarations
                .iter()
                .filter_map(|declaration| match declaration {
                    Declaration::Def(d) if d.name.starts_with(prefix) => Some(d.name.as_str()),
                    _ => None,
                })
                .collect();
            assert_eq!(names, vec![format!("{prefix}0")]);
        }
        drop(origins);
        crate::kernel::check_book(&book)
            .unwrap()
            .evaluate_data("main", &[])
            .unwrap();
    }
}
