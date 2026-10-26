// SPDX-License-Identifier: MPL-2.0
//! Failure-only quote diagnostics with bounded, isolated parser state.
use super::BTreeMap;
use super::BTreeSet;
use super::Binder;
use super::Book;
use super::Declaration;
use super::LexicalPrefix;
use super::ParseInput;
use super::Parser;
use super::SourceOrigin;
use super::Templates;
use super::VecDeque;
use crate::syntax::diagnostics::LoadDiagnostic;

// Additional presentation ceilings, never changes to accepted language input.
// Count every copied header/container entry and string before any state clone.
const MAX_HEADER_ITEMS: usize = 4096;
const MAX_HEADER_BYTES: usize = 64 * 1024;

struct CopyBudget {
    items: usize,
    bytes: usize,
}

impl CopyBudget {
    fn string(&mut self, value: &str) -> Option<()> {
        self.items = self.items.checked_sub(1)?;
        self.bytes = self.bytes.checked_sub(value.len())?;
        Some(())
    }

    fn binders(&mut self, binders: &[Binder]) -> Option<()> {
        for binder in binders {
            self.string(&binder.name)?;
        }
        Some(())
    }

    fn map(&mut self, map: &BTreeMap<String, String>) -> Option<()> {
        for (key, value) in map {
            self.string(key)?;
            self.string(value)?;
        }
        Some(())
    }

    fn book(&mut self, book: &Book) -> Option<()> {
        for declaration in &book.declarations {
            match declaration {
                Declaration::Adt(adt) => {
                    self.string(&adt.name)?;
                    self.binders(&adt.parameters)?;
                    for constructor in &adt.constructors {
                        self.string(&constructor.name)?;
                        self.binders(&constructor.fields)?;
                    }
                }
                Declaration::Def(definition) => {
                    self.string(&definition.name)?;
                    self.binders(&definition.parameters)?;
                    if let Some(parameters) = &definition.completion_parameters {
                        for (name, _) in parameters {
                            self.string(name)?;
                        }
                    }
                }
            }
        }
        Some(())
    }

    fn templates(&mut self, templates: &Templates) -> Option<()> {
        for (name, template) in &templates.declarations {
            self.string(name)?;
            self.string(&template.label)?;
            self.string(&template.namespace)?;
            self.map(&template.aliases)?;
            self.map(&template.instances)?;
            if let SourceOrigin::Local(path) = &template.origin {
                self.items = self.items.checked_sub(1)?;
                self.bytes = self.bytes.checked_sub(path.as_os_str().len())?;
            }
            // tokens is one immutable Rc<[Token]>, never a copied token array.
        }
        Some(())
    }
}

pub(super) struct QuoteRecovery {
    prefix: LexicalPrefix,
    book: Book,
    templates: Templates,
    fresh: usize,
    label: String,
    namespace: String,
    aliases: BTreeMap<String, String>,
}

impl QuoteRecovery {
    pub(super) fn capture(
        mut prefix: LexicalPrefix,
        input: &ParseInput<'_>,
        book: &Book,
        fresh: usize,
        templates: &Templates,
    ) -> Option<Self> {
        if input.source.len() > 4096
            || templates.lexical_diagnostic.is_some()
            || !prefix.tokens.iter().any(|token| token.text == "'''")
        {
            return None;
        }
        let mut budget = CopyBudget {
            items: MAX_HEADER_ITEMS,
            bytes: MAX_HEADER_BYTES,
        };
        budget.string(input.label)?;
        budget.string(input.label)?; // independent source-map key
        budget.string(input.namespace)?;
        budget.map(&input.aliases)?;
        budget.book(book)?;
        budget.templates(templates)?;
        let diagnostics = templates
            .diagnostics
            .as_ref()?
            .isolated_load_context(input.label)?;
        prefix.diagnostic.initial_declarations = book.declarations.len();
        Some(Self {
            prefix,
            book: book.clone(),
            templates: Templates {
                declarations: templates.declarations.clone(),
                instances: templates.instances,
                active: templates.active,
                diagnostics: Some(diagnostics),
                lexical_diagnostic: None,
            },
            fresh,
            label: input.label.into(),
            namespace: input.namespace.into(),
            aliases: input.aliases.clone(),
        })
    }

    pub(super) fn diagnostic(mut self) -> Option<LoadDiagnostic> {
        self.templates.lexical_diagnostic = Some(self.prefix.diagnostic);
        let result = Parser {
            tokens: self.prefix.tokens,
            at: 0,
            label: self.label,
            namespace: self.namespace,
            aliases: self.aliases,
            scope: Vec::new(),
            fresh: &mut self.fresh,
            display_fresh: 0,
            book: &mut self.book,
            depth: 0,
            reusable: Vec::new(),
            reusable_markers: BTreeSet::new(),
            templates: &mut self.templates,
            instance_name: None,
            instance_arguments: VecDeque::new(),
            compile_bindings: BTreeMap::new(),
            foreign: None,
            constructor_tags: None,
            // No host paths: foreign=None preserves the proof-mode refusal
            // before foreign_imports can inspect a path.
            origin: SourceOrigin::Standalone,
        }
        .declarations();
        let error = result.err()?;
        if self.templates.lexical_diagnostic.as_ref()?.unsupported() {
            return None;
        }
        self.templates
            .diagnostics?
            .load_failure(error)
            .diagnostic
            .map(|diagnostic| *diagnostic)
    }
}

#[cfg(test)]
mod tests;
