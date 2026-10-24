// SPDX-License-Identifier: MPL-2.0
//! Optional source provenance, external to terms and their identity/semantics.
use super::Term;
use super::TermRef;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::rc::Rc;
use std::rc::Weak;

#[derive(Clone, Debug, Default)]
pub(crate) struct TermOrigins(Rc<RefCell<Index>>);

#[derive(Debug, Default)]
struct Index {
    nodes: BTreeMap<usize, Origin>,
    // The parser keeps unique semantic IDs for upstream's unbound + markers.
    // This display-only index records their spelling without changing terms.
    reusable_markers: BTreeSet<usize>,
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
}

impl TermOrigins {
    pub(crate) fn reusable_marker(&self, id: usize) {
        self.0.borrow_mut().reusable_markers.insert(id);
    }

    pub(crate) fn is_reusable_marker(&self, id: usize) -> bool {
        self.0.borrow().reusable_markers.contains(&id)
    }

    pub(crate) fn track(&self, value: &TermRef) {
        let key = Rc::as_ptr(value) as usize;
        let mut index = self.0.borrow_mut();
        let children = match value.as_ref() {
            Term::Eql { left, right, ty } => Some([Some(left), Some(right), Some(ty)]),
            Term::Rwt {
                evidence,
                motive,
                body,
            } => Some([Some(evidence), Some(motive), Some(body)]),
            Term::Lam { body, .. } => Some([Some(body), None, None]),
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

    pub(crate) fn source(&self, value: &TermRef) -> usize {
        let key = Rc::as_ptr(value) as usize;
        self.0
            .borrow()
            .nodes
            .get(&key)
            .map_or(key, |origin| origin.source)
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
            .and_then(|children| children[operand])
    }

    /// Propagate along an actual substitution/qualification result. These
    /// transformations preserve every non-variable node's structure. A replaced
    /// variable keeps the replacement's own provenance; never search for a
    /// structurally similar term or infer a source line from names or text.
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
                let source = origin.source;
                let child_origins = origin.child_origins;
                origins.nodes.insert(
                    Rc::as_ptr(after) as usize,
                    Origin {
                        node: Rc::downgrade(after),
                        source,
                        child_origins,
                    },
                );
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
