// SPDX-License-Identifier: MPL-2.0
//! Optional source provenance, external to terms and their identity/semantics.
use super::Term;
use super::TermRef;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::rc::Rc;
use std::rc::Weak;

// Diagnostic-only source of a generated match, never part of a core term.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MatchSite {
    Expression(usize),
    Statement(usize),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TermOrigins(Rc<RefCell<Index>>);

#[derive(Debug, Default)]
struct Index {
    nodes: BTreeMap<usize, Origin>,
    // Captured source identities can outlive their semantic allocations and
    // transformed-node sweeps. Reserve only origin roots, never every result of
    // substitution, so a stale address cannot become an unrelated source span.
    source_roots: BTreeMap<usize, Weak<Term>>,
    // The parser keeps unique semantic IDs for upstream's unbound + markers.
    // This display-only index records their spelling without changing terms.
    reusable_markers: BTreeSet<usize>,
    binder_sources: BTreeMap<usize, usize>,
    sweep_at: usize,
}

#[derive(Debug)]
struct Origin {
    // Weak references prevent address reuse without keeping terms/fields alive
    // or changing strong ownership. Tracked terms are immutable, and the
    // tracker is discarded before evaluation; no weak reference enters a term.
    node: Weak<Term>,
    source: usize,
    // Per-occurrence anchors survive binder substitution even when several
    // variable uses share one replacement allocation. No strong term edges.
    child_origins: Option<[Option<usize>; 3]>,
    // Explicit template uses can override a shared Ref's caller origin. These
    // are distinct from fallback origins and support arbitrary field indices.
    occurrence_children: Vec<Option<usize>>,
    match_site: Option<MatchSite>,
    constructor_source: Option<usize>,
}

impl Origin {
    fn with_node(&self, node: &TermRef) -> Self {
        Self {
            node: Rc::downgrade(node),
            source: self.source,
            child_origins: self.child_origins,
            occurrence_children: self.occurrence_children.clone(),
            match_site: self.match_site,
            constructor_source: self.constructor_source,
        }
    }
}

impl TermOrigins {
    pub(crate) fn binder_source(&self, id: usize, source: usize) {
        self.0.borrow_mut().binder_sources.insert(id, source);
    }

    pub(crate) fn binder_node(&self, id: usize, value: &TermRef) {
        self.track(value);
        let mut index = self.0.borrow_mut();
        if let Some(source) = index.binder_sources.get(&id).copied() {
            index
                .nodes
                .get_mut(&(Rc::as_ptr(value) as usize))
                .expect("tracked binder")
                .source = source;
        }
    }

    pub(crate) fn reusable_marker(&self, id: usize) {
        self.0.borrow_mut().reusable_markers.insert(id);
    }

    pub(crate) fn is_reusable_marker(&self, id: usize) -> bool {
        self.0.borrow().reusable_markers.contains(&id)
    }

    pub(crate) fn track(&self, value: &TermRef) {
        let key = Rc::as_ptr(value) as usize;
        let mut index = self.0.borrow_mut();
        if !index.nodes.contains_key(&key) {
            index
                .source_roots
                .entry(key)
                .or_insert_with(|| Rc::downgrade(value));
        }
        let children = match value.as_ref() {
            Term::Eql { left, right, ty } => Some([Some(left), Some(right), Some(ty)]),
            Term::Rwt {
                evidence,
                motive,
                body,
            } => Some([Some(evidence), Some(motive), Some(body)]),
            Term::Lam { body, .. } => Some([Some(body), None, None]),
            Term::All { domain, body, .. } => Some([Some(domain), Some(body), None]),
            Term::Ann(value, ty) => Some([Some(value), Some(ty), None]),
            _ => None,
        };
        let child_origins = children.map(|children| {
            children.map(|child| {
                child
                    .and_then(|child| index.nodes.get(&(Rc::as_ptr(child) as usize)))
                    .map(|origin| origin.source)
            })
        });
        index.nodes.entry(key).or_insert_with(|| Origin {
            node: Rc::downgrade(value),
            source: key,
            child_origins,
            occurrence_children: Vec::new(),
            match_site: None,
            constructor_source: None,
        });
    }

    // Generated surface parents need per-edge anchors when a reply is replaced
    // by a shared constructor. Only weak, external provenance is recorded.
    pub(crate) fn surface_children(&self, value: &TermRef, children: [Option<usize>; 3]) {
        self.track(value);
        self.0
            .borrow_mut()
            .nodes
            .get_mut(&(Rc::as_ptr(value) as usize))
            .expect("tracked surface parent")
            .child_origins = Some(children);
    }

    pub(crate) fn occurrence_children(
        &self,
        value: &TermRef,
        children: impl IntoIterator<Item = Option<usize>>,
    ) {
        let children: Vec<_> = children.into_iter().collect();
        if children.iter().all(Option::is_none) {
            return;
        }
        self.track(value);
        self.0
            .borrow_mut()
            .nodes
            .get_mut(&(Rc::as_ptr(value) as usize))
            .expect("tracked occurrence parent")
            .occurrence_children = children;
    }

    pub(crate) fn occurrence_source(&self, parent: &TermRef, index: usize) -> Option<usize> {
        self.0
            .borrow()
            .nodes
            .get(&(Rc::as_ptr(parent) as usize))?
            .occurrence_children
            .get(index)
            .copied()
            .flatten()
    }

    // After parser macro or checker beta substitution, attach the
    // replacement occurrence to actual changed parent edges, never to the
    // shared replacement node. Binder shadowing mirrors core substitution.
    pub(crate) fn substituted_occurrence(
        &self,
        before: &TermRef,
        after: &TermRef,
        target: usize,
        source: usize,
    ) {
        let mut pending = vec![(before, after)];
        while let Some((before, after)) = pending.pop() {
            if Rc::ptr_eq(before, after) {
                continue;
            }
            let old_children = substitution_children(before, target);
            let new_children = substitution_children(after, target);
            for ((index, old), (_, new)) in old_children.into_iter().zip(new_children) {
                if matches!(old.as_ref(), Term::Var { id, .. } if *id == target) {
                    self.track(after);
                    let mut origins = self.0.borrow_mut();
                    let children = &mut origins
                        .nodes
                        .get_mut(&(Rc::as_ptr(after) as usize))
                        .expect("tracked substitution parent")
                        .occurrence_children;
                    if children.len() <= index {
                        children.resize(index + 1, None);
                    }
                    children[index] = Some(source);
                } else {
                    pending.push((old, new));
                }
            }
        }
    }

    pub(crate) fn surface_match(&self, value: &TermRef, site: MatchSite) {
        self.track(value);
        self.0
            .borrow_mut()
            .nodes
            .get_mut(&(Rc::as_ptr(value) as usize))
            .expect("tracked match")
            .match_site = Some(site);
    }

    pub(crate) fn match_constructor(&self, value: &TermRef, source: usize) {
        self.track(value);
        self.0
            .borrow_mut()
            .nodes
            .get_mut(&(Rc::as_ptr(value) as usize))
            .expect("tracked constructor match")
            .constructor_source = Some(source);
    }

    pub(crate) fn constructor_source(&self, value: &TermRef) -> Option<usize> {
        self.0
            .borrow()
            .nodes
            .get(&(Rc::as_ptr(value) as usize))?
            .constructor_source
    }

    pub(crate) fn match_site(&self, value: &TermRef) -> Option<MatchSite> {
        self.0
            .borrow()
            .nodes
            .get(&(Rc::as_ptr(value) as usize))?
            .match_site
    }

    pub(crate) fn source(&self, value: &TermRef) -> usize {
        let key = Rc::as_ptr(value) as usize;
        let mut index = self.0.borrow_mut();
        if let Some(origin) = index.nodes.get(&key) {
            return origin.source;
        }
        index
            .source_roots
            .entry(key)
            .or_insert_with(|| Rc::downgrade(value));
        key
    }

    pub(crate) fn child_source(
        &self,
        parent: &TermRef,
        operand: usize,
        child: &TermRef,
    ) -> Option<usize> {
        let origins = self.0.borrow();
        // An independently sourced replacement keeps its own source location.
        if origins.nodes.contains_key(&(Rc::as_ptr(child) as usize)) {
            return None;
        }
        origins
            .nodes
            .get(&(Rc::as_ptr(parent) as usize))?
            .child_origins
            .and_then(|children| children.get(operand).copied().flatten())
    }

    /// Propagate along an actual substitution/qualification result. These
    /// transformations preserve non-variable structure, except the explicit
    /// source-preserving Ref-to-Adt reduction, which copies only the root origin.
    /// A replaced variable keeps the replacement's own provenance; never search
    /// for a structurally similar term or infer a source line from names or text.
    pub(crate) fn transformed(&self, before: &TermRef, after: &TermRef) {
        let mut origins = self.0.borrow_mut();
        // Retire dead transformed allocations between operations. Source spans
        // independently hold weak anchors for their own immutable identities.
        if origins.nodes.len() >= origins.sweep_at {
            origins
                .nodes
                .retain(|_, origin| origin.node.strong_count() != 0);
            origins.sweep_at = origins.nodes.len().saturating_add(256);
        }
        let mut pending = vec![(before, after)];
        while let Some((before, after)) = pending.pop() {
            if Rc::ptr_eq(before, after) {
                continue;
            }
            if matches!(before.as_ref(), Term::Var { .. }) {
                continue;
            }
            if let Some(origin) = origins.nodes.get(&(Rc::as_ptr(before) as usize)) {
                let next = origin.with_node(after);
                origins.nodes.insert(Rc::as_ptr(after) as usize, next);
            }
            match (before.as_ref(), after.as_ref()) {
                (Term::Typ(a), Term::Typ(b))
                | (Term::Lam { body: a, .. }, Term::Lam { body: b, .. }) => pending.push((a, b)),
                (Term::Min(a, b), Term::Min(c, d))
                | (Term::App(a, b), Term::App(c, d))
                | (Term::Ann(a, b), Term::Ann(c, d))
                | (
                    Term::All {
                        domain: a, body: b, ..
                    },
                    Term::All {
                        domain: c, body: d, ..
                    },
                )
                | (
                    Term::Mat {
                        arm: a,
                        fallback: b,
                        ..
                    },
                    Term::Mat {
                        arm: c,
                        fallback: d,
                        ..
                    },
                ) => pending.extend([(a, c), (b, d)]),
                (Term::Ctr { args: a, .. }, Term::Ctr { args: b, .. })
                | (Term::Adt { args: a, .. }, Term::Adt { args: b, .. }) => {
                    pending.extend(a.iter().zip(b));
                }
                (
                    Term::Eql {
                        left: a,
                        right: b,
                        ty: source,
                    },
                    Term::Eql {
                        left: c,
                        right: d,
                        ty: result,
                    },
                )
                | (
                    Term::Rwt {
                        evidence: a,
                        motive: b,
                        body: source,
                    },
                    Term::Rwt {
                        evidence: c,
                        motive: d,
                        body: result,
                    },
                ) => pending.extend([(a, c), (b, d), (source, result)]),
                (
                    Term::Let {
                        bindings: a,
                        body: b,
                    },
                    Term::Let {
                        bindings: c,
                        body: d,
                    },
                ) => {
                    pending.push((b, d));
                    pending.extend(a.iter().zip(c).map(|(a, b)| (&a.value, &b.value)));
                }
                _ => {}
            }
        }
    }
}

fn substitution_children(value: &TermRef, target: usize) -> Vec<(usize, &TermRef)> {
    match value.as_ref() {
        Term::Typ(g) => vec![(0, g)],
        Term::Lam { id, body, .. } if *id != target => vec![(0, body)],
        Term::All {
            id, domain, body, ..
        } => {
            let mut children = vec![(0, domain)];
            if *id != target {
                children.push((1, body));
            }
            children
        }
        Term::Min(a, b)
        | Term::App(a, b)
        | Term::Ann(a, b)
        | Term::Mat {
            arm: a,
            fallback: b,
            ..
        } => vec![(0, a), (1, b)],
        Term::Ctr { args, .. } | Term::Adt { args, .. } => args.iter().enumerate().collect(),
        Term::Eql {
            left: a,
            right: b,
            ty: c,
        }
        | Term::Rwt {
            evidence: a,
            motive: b,
            body: c,
        } => vec![(0, a), (1, b), (2, c)],
        Term::Let { bindings, body } => {
            let mut children: Vec<_> = bindings
                .iter()
                .enumerate()
                .map(|(i, b)| (i + 1, &b.value))
                .collect();
            if !bindings.iter().any(|b| b.id == target) {
                children.push((0, body));
            }
            children
        }
        _ => Vec::new(),
    }
}

pub(crate) fn substitute_with_origins(
    value: &TermRef,
    target: usize,
    replacement: &TermRef,
    origins: Option<&TermOrigins>,
) -> TermRef {
    let result = super::substitute(value, target, replacement);
    if let Some(origins) = origins {
        origins.transformed(value, &result);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::term;

    fn variable(id: usize) -> TermRef {
        term(Term::Var {
            name: "x".into(),
            id,
        })
    }

    #[test]
    fn type_children_keep_distinct_sources_without_owning_shared_replacements() {
        for annotation in [false, true] {
            let origins = TermOrigins::default();
            let first = variable(7);
            let second = variable(7);
            origins.track(&first);
            origins.track(&second);
            let original = term(if annotation {
                Term::Ann(Rc::clone(&first), Rc::clone(&second))
            } else {
                Term::All {
                    quant: super::super::Quant::None,
                    name: "y".into(),
                    id: 8,
                    domain: Rc::clone(&first),
                    body: Rc::clone(&second),
                }
            });
            origins.track(&original);
            let replacement = variable(99);
            let result = substitute_with_origins(&original, 7, &replacement, Some(&origins));
            let (a, b) = match result.as_ref() {
                Term::All { domain, body, .. } | Term::Ann(domain, body) => (domain, body),
                _ => panic!("type parent"),
            };
            assert!(Rc::ptr_eq(a, &replacement) && Rc::ptr_eq(b, &replacement));
            assert_eq!(Rc::strong_count(&replacement), 3);
            assert_eq!(
                origins.child_source(&result, 0, &replacement),
                Some(Rc::as_ptr(&first) as usize)
            );
            assert_eq!(
                origins.child_source(&result, 1, &replacement),
                Some(Rc::as_ptr(&second) as usize)
            );
            assert_ne!(Rc::as_ptr(&first), Rc::as_ptr(&second));
            // A separately sourced replacement keeps its own location.
            origins.track(&replacement);
            assert_eq!(origins.child_source(&result, 0, &replacement), None);
            assert_eq!(origins.child_source(&result, 1, &replacement), None);
            drop(origins);
            assert_eq!(Rc::strong_count(&replacement), 3);
        }
    }

    #[test]
    fn macro_occurrences_map_changed_edges_without_copying_or_crossing_binders() {
        let origins = TermOrigins::default();
        let replacement = term(Term::Ref("same".into()));
        let bound = variable(7);
        let shadow = term(Term::Lam {
            name: "x".into(),
            id: 7,
            body: Rc::clone(&bound),
        });
        let original = term(Term::Ctr {
            name: "Four".into(),
            args: vec![
                Rc::clone(&bound),
                Rc::clone(&shadow),
                Rc::clone(&bound),
                Rc::clone(&bound),
            ],
        });
        let result = substitute_with_origins(&original, 7, &replacement, Some(&origins));
        origins.substituted_occurrence(&original, &result, 7, 99);
        let Term::Ctr { args, .. } = result.as_ref() else {
            panic!("constructor");
        };
        for index in [0, 2, 3] {
            assert!(Rc::ptr_eq(&args[index], &replacement));
            assert_eq!(origins.occurrence_source(&result, index), Some(99));
        }
        assert!(Rc::ptr_eq(&args[1], &shadow));
        assert_eq!(origins.occurrence_source(&result, 1), None);
        assert_eq!(origins.occurrence_source(&shadow, 0), None);
        assert_eq!(
            origins.source(&replacement),
            Rc::as_ptr(&replacement) as usize
        );
        assert_eq!(Rc::strong_count(&replacement), 4);
        drop(origins);
        assert_eq!(Rc::strong_count(&replacement), 4);
    }

    #[test]
    fn match_sites_survive_substitution_without_retaining_terms() {
        for site in [MatchSite::Expression(17), MatchSite::Statement(19)] {
            let origins = TermOrigins::default();
            let fallback = term(Term::Efq);
            let original = term(Term::Mat {
                constructor: "Z".into(),
                arm: variable(1),
                fallback: Rc::clone(&fallback),
            });
            origins.surface_match(&original, site);
            origins.match_constructor(&original, 29);
            origins.surface_match(&fallback, site);
            origins.surface_children(&original, [Some(23), None, None]);
            assert_eq!(Rc::strong_count(&original), 1);
            let replacement = variable(2);
            let result = substitute_with_origins(&original, 1, &replacement, Some(&origins));
            assert_eq!(origins.match_site(&result), Some(site));
            assert_eq!(origins.constructor_source(&result), Some(29));
            assert_eq!(origins.child_source(&result, 0, &replacement), Some(23));
            let Term::Mat {
                arm,
                fallback: after,
                ..
            } = result.as_ref()
            else {
                panic!("match")
            };
            assert!(Rc::ptr_eq(arm, &replacement));
            assert!(Rc::ptr_eq(after, &fallback));
            assert_eq!(origins.match_site(after), Some(site));
            assert_eq!(
                format!("{result:?}"),
                format!("{:?}", super::super::substitute(&original, 1, &replacement))
            );
            let weak = Rc::downgrade(&result);
            drop(result);
            assert!(weak.upgrade().is_none());
            drop(origins);
            assert_eq!(Rc::weak_count(&original), 0);
        }
    }

    #[test]
    fn observation_preserves_identity_sharing_and_replacement_origin() {
        let origins = TermOrigins::default();
        let child = term(Term::Ctr {
            name: "Child".into(),
            args: vec![],
        });
        let original = term(Term::Ctr {
            name: "Outer".into(),
            args: vec![variable(1), Rc::clone(&child), Rc::clone(&child)],
        });
        let replacement = term(Term::Ctr {
            name: "Replacement".into(),
            args: vec![],
        });
        let counts = (
            Rc::strong_count(&original),
            Rc::strong_count(&child),
            Rc::strong_count(&replacement),
        );
        origins.track(&original);
        origins.track(&replacement);
        assert_eq!(
            counts,
            (
                Rc::strong_count(&original),
                Rc::strong_count(&child),
                Rc::strong_count(&replacement)
            )
        );
        let ordinary = super::super::substitute(&original, 1, &replacement);
        let tracked = substitute_with_origins(&original, 1, &replacement, Some(&origins));
        assert_eq!(format!("{ordinary:?}"), format!("{tracked:?}"));
        assert_eq!(ordinary.to_string(), tracked.to_string());
        assert_eq!(origins.source(&tracked), Rc::as_ptr(&original) as usize);
        let Term::Ctr { args, .. } = tracked.as_ref() else {
            panic!("constructor");
        };
        assert!(Rc::ptr_eq(&args[0], &replacement));
        assert_eq!(origins.source(&args[0]), Rc::as_ptr(&replacement) as usize);
        assert!(Rc::ptr_eq(&args[1], &args[2]) && Rc::ptr_eq(&args[1], &child));
        assert!(Rc::ptr_eq(
            &substitute_with_origins(&original, 99, &replacement, Some(&origins)),
            &original
        ));
        let weak = Rc::downgrade(&tracked);
        drop(tracked);
        assert!(
            weak.upgrade().is_none(),
            "provenance must not keep terms or their fields alive"
        );
        drop(origins);
        assert_eq!(Rc::weak_count(&original), 0);
        assert_eq!(Rc::weak_count(&replacement), 0);
    }

    #[test]
    fn equality_occurrences_keep_distinct_origins_without_copying_replacements() {
        let origins = TermOrigins::default();
        let left = variable(1);
        let right = variable(1);
        let original = term(Term::Eql {
            left: Rc::clone(&left),
            right: Rc::clone(&right),
            ty: variable(2),
        });
        for value in [&left, &right, &original] {
            origins.track(value);
        }
        let replacement = variable(3);
        let result = substitute_with_origins(&original, 1, &replacement, Some(&origins));
        let Term::Eql {
            left: a, right: b, ..
        } = result.as_ref()
        else {
            panic!("equality");
        };
        assert!(Rc::ptr_eq(a, &replacement) && Rc::ptr_eq(a, b));
        assert_eq!(
            origins.child_source(&result, 0, a),
            Some(Rc::as_ptr(&left) as usize)
        );
        assert_eq!(
            origins.child_source(&result, 1, b),
            Some(Rc::as_ptr(&right) as usize)
        );
        let next = substitute_with_origins(&result, 2, &variable(4), Some(&origins));
        assert_eq!(
            origins.child_source(&next, 1, &replacement),
            Some(Rc::as_ptr(&right) as usize)
        );
        assert_eq!(
            origins.child_source(&next, 2, &variable(4)),
            None,
            "an untracked operand never invents a source anchor"
        );
        origins.track(&replacement);
        assert_eq!(origins.child_source(&next, 1, &replacement), None);
        drop(result);
        drop(next);
        drop(origins);
        assert_eq!(Rc::strong_count(&replacement), 1);
        assert_eq!(Rc::weak_count(&replacement), 0);
    }

    #[test]
    fn rewrite_and_motive_occurrences_keep_use_sites_without_copying_values() {
        let origins = TermOrigins::default();
        let evidence = variable(1);
        let body = variable(1);
        let motive = term(Term::Lam {
            name: "proof".into(),
            id: 2,
            body: Rc::clone(&body),
        });
        let original = term(Term::Rwt {
            evidence: Rc::clone(&evidence),
            motive: Rc::clone(&motive),
            body: term(Term::Rfl),
        });
        for value in [&evidence, &body, &motive, &original] {
            let count = Rc::strong_count(value);
            origins.track(value);
            assert_eq!(Rc::strong_count(value), count);
        }
        let replacement = variable(3);
        let result = substitute_with_origins(&original, 1, &replacement, Some(&origins));
        let Term::Rwt {
            evidence: e,
            motive: m,
            ..
        } = result.as_ref()
        else {
            panic!("rewrite");
        };
        let Term::Lam { body: b, .. } = m.as_ref() else {
            panic!("motive");
        };
        assert!(Rc::ptr_eq(e, &replacement) && Rc::ptr_eq(e, b));
        assert_eq!(
            origins.child_source(&result, 0, e),
            Some(Rc::as_ptr(&evidence) as usize)
        );
        assert_eq!(
            origins.child_source(m, 0, b),
            Some(Rc::as_ptr(&body) as usize)
        );
        origins.track(&replacement);
        assert_eq!(origins.child_source(&result, 0, e), None);
        assert_eq!(origins.child_source(m, 0, b), None);
        drop(result);
        drop(origins);
        assert_eq!(Rc::strong_count(&replacement), 1);
        assert_eq!(Rc::weak_count(&replacement), 0);
    }

    #[test]
    fn captured_untracked_source_reserves_only_a_weak_identity() {
        let origins = TermOrigins::default();
        let child = term(Term::Qua(super::super::Quant::Lone));
        let original = term(Term::Typ(Rc::clone(&child)));
        let source = origins.source(&original);
        assert_eq!(Rc::strong_count(&original), 1);
        assert_eq!(origins.0.borrow().source_roots.len(), 1);
        assert!(origins.0.borrow().nodes.is_empty());
        drop(original);
        assert_eq!(Rc::strong_count(&child), 1);
        assert!(origins.0.borrow().source_roots[&source].upgrade().is_none());
        for _ in 0..1024 {
            let unrelated = variable(7);
            assert_ne!(Rc::as_ptr(&unrelated) as usize, source);
        }
    }

    #[test]
    fn tracked_root_survives_transform_and_sweep_before_first_source_query() {
        let origins = TermOrigins::default();
        let mut value = term(Term::Ctr {
            name: "Outer".into(),
            args: vec![variable(0)],
        });
        let source = Rc::as_ptr(&value) as usize;
        origins.track(&value);
        for id in 0..1024 {
            value = substitute_with_origins(&value, id, &variable(id + 1), Some(&origins));
        }
        assert_eq!(origins.source(&value), source);
        assert!(origins.0.borrow().nodes.len() < 260);
        assert_eq!(origins.0.borrow().source_roots.len(), 1);
        assert!(origins.0.borrow().source_roots[&source].upgrade().is_none());
        assert_eq!(Rc::strong_count(&value), 1);
        drop(origins);
        assert_eq!(Rc::weak_count(&value), 0);
    }

    #[test]
    fn long_transformation_chain_retires_dead_entries_without_losing_origin() {
        let origins = TermOrigins::default();
        let mut value = term(Term::Ctr {
            name: "Outer".into(),
            args: vec![variable(0)],
        });
        let source = Rc::as_ptr(&value) as usize;
        let anchor = Rc::downgrade(&value);
        origins.track(&value);
        for id in 0..1024 {
            value = substitute_with_origins(&value, id, &variable(id + 1), Some(&origins));
            assert_eq!(origins.source(&value), source);
        }
        assert!(anchor.upgrade().is_none());
        assert!(origins.0.borrow().nodes.len() < 260);
    }
}
