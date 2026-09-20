// SPDX-License-Identifier: Apache-2.0
// Derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
use super::executable::BuiltinForeign;
use super::executable::ExecutableSource;
use super::executable::ForeignDefinition;
use super::executable::ForeignImport;
use super::executable::ForeignTarget;
use super::executable::NumericIntrinsic;
use super::surface::Body;
use super::surface::Pattern;
use super::surface::Row;
use super::surface::Variable;
use super::surface::flatten;
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
    book: &'a mut Book,
    depth: usize,
    reusable: Vec<(Variable, TermRef)>,
    reusable_markers: BTreeSet<usize>,
    templates: &'a mut Templates,
    instance_name: Option<String>,
    instance_arguments: VecDeque<TermRef>,
    compile_bindings: BTreeMap<usize, TermRef>,
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
            Err(self.error(format!("expected {s:?}, found {:?}", self.current().text)))
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
    fn fresh_variable(&mut self, name: String) -> Variable {
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
    fn variable(&self, name: &str) -> TermRef {
        self.scope
            .iter()
            .rev()
            .find(|v| v.name == name)
            .map_or_else(
                || term(Term::Ref(self.resolve(name))),
                |variable| {
                    self.compile_bindings
                        .get(&variable.id)
                        .map_or_else(|| variable.term(), Rc::clone)
                },
            )
    }
    fn is_compile_binding(&self, value: &TermRef) -> bool {
        self.compile_bindings
            .values()
            .any(|bound| Rc::ptr_eq(bound, value))
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
                (Quant::None, term(Term::Qnt))
            } else {
                return Err(self.error("expected a type after a quantified binder"));
            };
            let variable = self.open(name.clone());
            if compile_time && let Some(argument) = self.instance_arguments.pop_front() {
                self.compile_bindings.insert(variable.id, argument);
            }
            binders.push(Binder {
                quant,
                name,
                id: variable.id,
                ty,
            });
            self.take(",");
        }
        Ok(binders)
    }
    fn declarations(&mut self) -> Result<(), ParseError> {
        while !self.is("") {
            self.scope.clear();
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
            kind,
            constructors,
        });
        Ok(())
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
            let all = self.take("for");
            if !all {
                self.expect("exs")?;
                universal = false;
            }
            let quant = if all { self.quant() } else { Quant::Lone };
            let name = self.name()?;
            self.expect(":")?;
            let mut ty = self.expression(0)?;
            if self.take("where") {
                let old = self.scope.len();
                let v = self.open(name.clone());
                let predicate = self.expression(0)?;
                self.scope.truncate(old);
                ty = apply(
                    term(Term::Ref("Exists".into())),
                    [
                        ty,
                        term(Term::Lam {
                            name: name.clone(),
                            id: v.id,
                            body: predicate,
                        }),
                    ],
                );
            }
            let v = self.open(name.clone());
            let b = Binder {
                quant,
                name,
                id: v.id,
                ty,
            };
            if universal {
                parameters.push(b.clone());
            }
            clauses.push((all, b));
        }
        let body = self.body(0)?;
        let mut ty = flatten(&body, &[], self.fresh).map_err(|e| self.error(e))?;
        for (all, b) in clauses.into_iter().rev() {
            ty = if all {
                arrows(&[b], ty)
            } else {
                apply(
                    term(Term::Ref("Exists".into())),
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
            name,
            parameters,
            ty: qualify_operators(&ty, &self.resolve("Nat")),
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
        let (parameters, ty, vars) = if let Some(prior) = prior {
            if prior.body.is_some() || prior.foreign {
                return Err(self.error(format!("duplicate definition: {name}")));
            }
            let mut vars = Vec::new();
            while !self.take(")") {
                if vars.len() >= 128 {
                    return Err(
                        self.error("proof telescope exceeds the parser resource limit of 128")
                    );
                }
                let name = self.name()?;
                vars.push(self.open(name));
                self.take(",");
            }
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
            let ty = arrows(&parameters, result);
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
        // Make recursive names visible while parsing the body, then retain one event.
        let temporary = self.book.declarations.len();
        self.book.declarations.push(Declaration::Def(DefDecl {
            name: name.clone(),
            parameters: parameters.clone(),
            ty: Rc::clone(&ty),
            body: None,
            unsafe_,
            foreign: false,
        }));
        let body = self.body(0)?;
        let value = flatten(&body, &vars, self.fresh).map_err(|e| self.error(e))?;
        // Specializations minted in the body precede their caller. Removing
        // only the temporary event preserves those separately checked defs.
        self.book.declarations.remove(temporary);
        if template {
            self.retain_template_source(&name, declaration_at);
        } else {
            self.book.declarations.push(Declaration::Def(DefDecl {
                name,
                parameters,
                ty: qualify_operators(&ty, &self.resolve("Nat")),
                body: Some(qualify_operators(&value, &self.resolve("Nat"))),
                unsafe_,
                foreign: false,
            }));
        }
        Ok(())
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
            name,
            parameters,
            ty: qualify_operators(ty, &self.resolve("Nat")),
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

    fn call(&mut self, mut function: TermRef) -> Result<TermRef, ParseError> {
        let template = match function.as_ref() {
            Term::Ref(name) if self.templates.declarations.contains_key(name) => Some(name.clone()),
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
                compile_arguments.push(qualify_operators(&argument, &self.resolve("Nat")));
                self.take(",");
            }
        }
        let runtime_arguments = self.arguments(")")?;
        if let Some(name) = template
            && let Some(instance) = self.specialize(&name, &compile_arguments)?
        {
            function = term(Term::Ref(instance));
        }
        for argument in compile_arguments.into_iter().chain(runtime_arguments) {
            function = if let Term::Lam { id, body, .. } = function.as_ref()
                && self.is_compile_binding(&function)
            {
                // Upstream Sub is resolved before live checking: duplicating a
                // macro parameter duplicates caller syntax, not an affine Lam.
                crate::kernel::substitute(body, *id, &argument)
            } else {
                term(Term::App(function, argument))
            };
        }
        Ok(function)
    }

    fn specialize(
        &mut self,
        name: &str,
        arguments: &[TermRef],
    ) -> Result<Option<String>, ParseError> {
        let Some(key) = template_key(arguments).map_err(|error| self.error(error))? else {
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
            book: self.book,
            depth: 0,
            reusable: Vec::new(),
            reusable_markers: BTreeSet::new(),
            templates: self.templates,
            instance_name: Some(instance.clone()),
            instance_arguments: arguments.iter().map(Rc::clone).collect(),
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
    fn expression(&mut self, min: u8) -> Result<TermRef, ParseError> {
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
    fn expression_inner(&mut self, min: u8) -> Result<TermRef, ParseError> {
        let mut out = self.atom()?;
        let mut operations = 0;
        loop {
            operations += 1;
            if operations > 256 {
                return Err(self.error("expression spine exceeds the parser resource limit of 256"));
            }
            let same_line = self.at > 0 && self.tokens[self.at - 1].line == self.current().line;
            if self.is("!") {
                return Err(self.error(
                    "GPU offload calls (!) are not supported by this Rust implementation yet",
                ));
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
                let index = qualify_operators(&index, "U32");
                if self.take("<-") {
                    let value = self.expression(2)?;
                    out = apply(
                        term(Term::Ref("Array.set".into())),
                        [term(Term::Ref("U32".into())), out, index, value],
                    );
                } else {
                    out = apply(
                        term(Term::Ref("Array.get".into())),
                        [term(Term::Ref("U32".into())), out, index],
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
                        Term::Ref(n) | Term::Var { name: n, .. } => self.resolve(n),
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
                                (0..quantities).map(|_| term(Term::Qua(Quant::Lone))),
                            );
                        }
                    }
                    out = term(Term::Adt {
                        name,
                        args,
                        excluded: Vec::new(),
                    });
                } else {
                    out = apply(term(Term::Ref(".is_lt".into())), [out, first]);
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
                out = flatten(&body, &[v], self.fresh).map_err(|e| self.error(e))?;
                self.scope.truncate(old);
                continue;
            }
            if min == 0 && self.take("->") {
                let body = self.expression(0)?;
                let v = self.fresh_variable("_".into());
                out = term(Term::All {
                    quant: Quant::Lone,
                    name: v.name,
                    id: v.id,
                    domain: out,
                    body,
                });
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
                "<&>" => term(Term::Min(out, b)),
                "<>" => term(Term::Ctr {
                    name: "Con".into(),
                    args: vec![out, b],
                }),
                _ => apply(term(Term::Ref(method.into())), [out, b]),
            };
        }
        Ok(out)
    }
    fn arguments(&mut self, close: &str) -> Result<Vec<TermRef>, ParseError> {
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
    #[expect(
        clippy::too_many_lines,
        reason = "Surface forms are kept in upstream grammar order for auditability"
    )]
    fn atom(&mut self) -> Result<TermRef, ParseError> {
        if self.is("~") {
            return Err(self.error(
                "~ arguments require a previously declared template head and leading argument position",
            ));
        }
        if self.take("Type") {
            return Ok(term(Term::Typ(term(Term::Qua(Quant::Lone)))));
        }
        if self.take("Data") {
            return Ok(term(Term::Typ(term(Term::Qua(Quant::Many)))));
        }
        if self.take("Quant") {
            return Ok(term(Term::Qnt));
        }
        if self.take("Kind") {
            self.expect("(")?;
            let g = self.expression(0)?;
            self.expect(")")?;
            return Ok(term(Term::Typ(g)));
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
                    return Ok(term(Term::Qua(q)));
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
                apply(
                    term(Term::Ref("Exists".into())),
                    [
                        domain,
                        term(Term::Lam {
                            name,
                            id: v.id,
                            body,
                        }),
                    ],
                )
            } else {
                term(Term::All {
                    quant,
                    name,
                    id: v.id,
                    domain,
                    body,
                })
            });
        }
        if self.take("{") {
            if self.take("==") {
                self.expect("}")?;
                return Ok(term(Term::Rfl));
            }
            let left = self.expression(0)?;
            let equality = self.take("==");
            let unequal = !equality && self.take("!=");
            if equality || unequal {
                let right = self.expression(0)?;
                self.expect(":")?;
                let ty = self.expression(0)?;
                self.expect("}")?;
                let equation = term(Term::Eql { left, right, ty });
                if unequal {
                    let v = self.fresh_variable("_".into());
                    return Ok(term(Term::All {
                        quant: Quant::Lone,
                        name: v.name,
                        id: v.id,
                        domain: equation,
                        body: term(Term::Ref("Empty".into())),
                    }));
                }
                return Ok(equation);
            }
            self.expect(":")?;
            let ty = self.expression(0)?;
            self.expect("}")?;
            return Ok(term(Term::Ann(left, ty)));
        }
        if self.take("(") {
            let body = self.body(self.current().column.saturating_sub(1))?;
            let mut out = flatten(&body, &[], self.fresh).map_err(|e| self.error(e))?;
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
                    term(Term::Ctr {
                        name: "Tuple".into(),
                        args: vec![head, tail],
                    })
                });
            }
            if self.take(":") {
                let ty = self.expression(0)?;
                let name = type_head(&ty)
                    .ok_or_else(|| self.error("operators need a named type after ':'"))?;
                out = qualify_operators(&out, &name);
            }
            self.expect(")")?;
            return Ok(out);
        }
        if self.take("\\") {
            self.expect("{")?;
            let mut arms = Vec::new();
            let mut tail = term(Term::Efq);
            while !self.take("}") {
                let value = self.expression(0)?;
                if self.take(":") {
                    let name = match value.as_ref() {
                        Term::Ref(n) | Term::Var { name: n, .. } => self.resolve(n),
                        _ => return Err(self.error("a lambda-match arm needs a constructor name")),
                    };
                    let arm = self.expression(0)?;
                    arms.push((name, arm));
                    self.take(";");
                } else {
                    tail = value;
                    self.take(";");
                    self.expect("}")?;
                    break;
                }
            }
            for (constructor, arm) in arms.into_iter().rev() {
                tail = term(Term::Mat {
                    constructor,
                    arm,
                    fallback: tail,
                });
            }
            return Ok(tail);
        }
        if self.take("%") {
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
            let body = flatten(&body, &[], self.fresh).map_err(|e| self.error(e))?;
            return Ok(term(Term::Rwt {
                evidence,
                motive: term(Term::Lam {
                    name: x.name,
                    id: x.id,
                    body: term(Term::Lam {
                        name,
                        id: e.id,
                        body: motive,
                    }),
                }),
                body,
            }));
        }
        if self.take("?") {
            let name = self.name()?;
            return Ok(term(Term::Hole(name)));
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
                    nat_term(count.ilog2())
                } else {
                    size
                };
                let namespace = type_head(&ty)
                    .ok_or_else(|| self.error("array operators need a named element type"))?;
                let value = qualify_operators(&values.remove(0), &namespace);
                return Ok(apply(
                    term(Term::Ref("Array.new".into())),
                    [ty, depth, value],
                ));
            }
            self.take(",");
            values.extend(self.arguments("]")?);
            return Ok(values.into_iter().rev().fold(
                term(Term::Ctr {
                    name: "Nil".into(),
                    args: Vec::new(),
                }),
                |tail, head| {
                    term(Term::Ctr {
                        name: "Con".into(),
                        args: vec![head, tail],
                    })
                },
            ));
        }
        if self.take("+") {
            let value = self.expression(12)?;
            let name =
                type_head(&value).ok_or_else(|| self.error("+ needs a quantified datatype"))?;
            let Some(Declaration::Adt(adt)) = self
                .book
                .declarations
                .iter()
                .find(|d| matches!(d,Declaration::Adt(a) if a.name==name))
            else {
                if matches!(value.as_ref(), Term::Ref(_) | Term::Var { .. }) {
                    let marker = self.fresh_variable(name);
                    self.reusable_markers.insert(marker.id);
                    return Ok(marker.term());
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
            return Ok(term(Term::Adt {
                name,
                args,
                excluded: Vec::new(),
            }));
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
                }));
            }
            return Ok(chars.into_iter().rev().fold(
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
            ));
        }
        let name = self.name()?;
        if self.at > 0 && self.tokens[self.at - 1].end == self.current().start && self.take("{") {
            let args = self.arguments("}")?;
            Ok(term(Term::Ctr {
                name: self.resolve(&name),
                args,
            }))
        } else {
            Ok(self.variable(&name))
        }
    }
    fn do_statement(
        &mut self,
        module: &str,
        types: &[TermRef],
        result: Option<&TermRef>,
        column: usize,
    ) -> Result<TermRef, ParseError> {
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
        types: &[TermRef],
        result: Option<&TermRef>,
        column: usize,
    ) -> Result<TermRef, ParseError> {
        if self.take("return") {
            let value = self.expression(0)?;
            return Ok(self.do_call(module, "pure", types, [], result, [value]));
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
            self.variable("Unit")
        } else {
            Rc::clone(&value)
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
            Rc::clone(&value)
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
        let tail = flatten(
            &Body::Reply(tail).with_reusable(&rebindings),
            &[],
            self.fresh,
        )
        .map_err(|e| self.error(e))?;
        self.scope.truncate(old);
        if assignment {
            Ok(term(Term::Let {
                bindings: vec![crate::kernel::LetBinding {
                    quant: Quant::Lone,
                    name: variable.name,
                    id: variable.id,
                    value: term(Term::Ann(bound, ty)),
                }],
                body: tail,
            }))
        } else {
            Ok(self.do_call(
                module,
                "bind",
                types,
                [ty],
                result,
                [
                    bound,
                    term(Term::Lam {
                        name: variable.name,
                        id: variable.id,
                        body: tail,
                    }),
                ],
            ))
        }
    }
    fn do_call(
        &self,
        module: &str,
        operation: &str,
        types: &[TermRef],
        before: impl IntoIterator<Item = TermRef>,
        result: Option<&TermRef>,
        after: impl IntoIterator<Item = TermRef>,
    ) -> TermRef {
        let arguments = types
            .iter()
            .map(Rc::clone)
            .chain(before)
            .chain(result.map(Rc::clone))
            .chain(after);
        apply(
            term(Term::Ref(self.resolve(&format!("{module}.{operation}")))),
            arguments,
        )
    }
    fn number(&mut self) -> Result<TermRef, ParseError> {
        let raw = self.bump();
        if let Some(digits) = raw.strip_suffix('n') {
            let count = digits
                .parse::<usize>()
                .map_err(|error| self.error(format!("Nat literal is too large: {error}")))?;
            if count > 128 {
                return Err(self.error("Nat literals above 128 exceed the parser's resource limit"));
            }
            let glued = self.at > 0 && self.tokens[self.at - 1].end == self.current().start;
            let mut value = if glued && self.take("+") {
                self.expression(0)?
            } else {
                term(Term::Ctr {
                    name: self.resolve("Zero"),
                    args: Vec::new(),
                })
            };
            for _ in 0..count {
                value = term(Term::Ctr {
                    name: self.resolve("Succ"),
                    args: vec![value],
                });
            }
            Ok(value)
        } else if raw.contains('.') {
            let value = raw
                .parse::<f32>()
                .map_err(|error| self.error(format!("invalid F32 literal: {error}")))?;
            if !value.is_finite() {
                return Err(self.error("F32 literal must be finite"));
            }
            Ok(term(Term::Ctr {
                name: "F32".into(),
                args: vec![word_term(value.to_bits())],
            }))
        } else {
            Ok(u32_term(raw.parse::<u32>().map_err(|error| {
                self.error(format!("U32 literal must be at most 4294967295: {error}"))
            })?))
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
                Ok(Pattern::Variable(
                    self.bind_variable(name.clone(), reusable),
                ))
            }
            Term::Ctr { name, args } => {
                let constructor = self.book.declarations.iter().find_map(|d| match d {
                    Declaration::Adt(a) => a.constructors.iter().find(|c| c.name == *name),
                    Declaration::Def(_) => None,
                });
                let Some(c) = constructor else {
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
                Ok(Pattern::Constructor(name.clone(), fields))
            }
            _ => Err(self.error("a pattern must be a binder or constructor")),
        }
    }
    fn terms_before_colon(&mut self) -> Result<Vec<TermRef>, ParseError> {
        let mut terms = Vec::new();
        loop {
            terms.push(self.expression(0)?);
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
    fn body_inner(&mut self, column: usize) -> Result<Body, ParseError> {
        if self.take("match") {
            let scrutinees = self.terms_before_colon()?;
            let case_column = self.current().column;
            let mut rows = Vec::new();
            while case_column > column && self.is("case") && self.current().column >= case_column {
                let row_column = self.current().column;
                self.bump();
                let patterns = self.terms_before_colon()?;
                if patterns.len() != scrutinees.len() {
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
            patterns.push(term(Term::Ref(name)));
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
                    patterns.push(variable);
                } else {
                    return Ok(Body::Reply(patterns.remove(0)));
                }
            } else {
                self.expect("=")?;
            }
        }
        while values.len() < patterns.len() {
            values.push(self.expression(0)?);
        }
        self.take(";");
        let old = self.scope.len();
        let patterns = patterns
            .iter()
            .map(|p| self.pattern(p))
            .collect::<Result<Vec<_>, _>>()?;
        let body = self.body(column)?;
        self.scope.truncate(old);
        Ok(Body::Local {
            patterns,
            quant,
            values,
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
    (matches!(head.as_ref(), Term::Ref(name) if name == "Array.set")
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
        Term::Ref(n) | Term::Var { name: n, .. } | Term::Adt { name: n, .. } => Some(n.clone()),
        Term::App(f, _) => type_head(f),
        _ => None,
    }
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
        Term::Ref(name) if name.starts_with('.') => term(Term::Ref(format!("{namespace}{name}"))),
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
        Term::Ctr { name, args } => term(Term::Ctr {
            name: name.clone(),
            args: args.iter().map(go).collect(),
        }),
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
    Parser {
        tokens: lex(input.source, input.label)?,
        at: 0,
        label: input.label.into(),
        namespace: input.namespace.into(),
        aliases: input.aliases,
        scope: Vec::new(),
        fresh,
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
            } else {
                if !text.is_empty() {
                    header = false;
                }
                cleaned.push_str(line);
                cleaned.push('\n');
            }
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
    let mut loader = Loader::default();
    loader.file(path.as_ref(), "")?;
    Ok(loader.book)
}

/// Read a local program with execution-only Base and retained foreign contracts.
///
/// The result is not a checked proof book. Foreign files are recorded but never
/// executed by loading; an execution checker must validate every signature.
///
/// # Errors
/// Returns a located error on Bend file access, import resolution or parsing failure.
pub fn load_executable(path: impl AsRef<Path>) -> Result<ExecutableSource, ParseError> {
    let mut loader = Loader {
        executable: true,
        ..Loader::default()
    };
    loader.file(path.as_ref(), "")?;
    Ok(ExecutableSource {
        book: loader.book,
        foreign: loader.foreign,
        numeric: loader.numeric,
        constructor_tags: loader.constructor_tags,
        base_names: loader.base_names,
    })
}
