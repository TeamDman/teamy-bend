// SPDX-License-Identifier: MPL-2.0
//! Shared inputs must retain path depth, occurrence work, and error order.

use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use teamy_bend::kernel::CheckedBook;
use teamy_bend::kernel::Declaration;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
use teamy_bend::kernel::term;
use teamy_bend::kernel::{self};

fn checked_book() -> CheckedBook {
    let book = teamy_bend::syntax::load("tests/fixtures/cpu-compact-patterns/helpers.bend")
        .expect("fixture loads");
    kernel::check_book(&book).expect("fixture checks")
}

fn wrap(value: TermRef, count: usize) -> TermRef {
    (0..count).fold(value, |body, _| term(Term::Typ(body)))
}

fn pair(a: &TermRef, b: &TermRef) -> TermRef {
    term(Term::Ctr {
        name: "Pair".into(),
        args: vec![Rc::clone(a), Rc::clone(b)],
    })
}

fn input_error(checked: &CheckedBook, input: TermRef) -> String {
    checked
        .evaluate("missing", &[input])
        .expect_err("the definition is intentionally absent")
        .message
}

// A representative unsound design: skipping an Rc once it has been visited,
// even though the same subtree may be encountered at a greater depth.
fn naive_unique_depth(root: &TermRef) -> bool {
    let mut seen = HashSet::new();
    let mut stack = vec![(root, 0)];
    while let Some((value, depth)) = stack.pop() {
        if depth > 128 {
            return false;
        }
        if !seen.insert(Rc::as_ptr(value)) {
            continue;
        }
        match value.as_ref() {
            Term::Typ(inner) => stack.push((inner, depth + 1)),
            Term::Ctr { args, .. } => {
                stack.extend(args.iter().map(|arg| (arg, depth + 1)));
            }
            _ => {}
        }
    }
    true
}

// Research-only fast-accept prototype for a restricted, hole-free subset.
// Unknown, over-budget, or deeply nested inputs return None and must take the
// original ordered inspection path; this never changes production checking.
fn guarded_summary(root: &TermRef) -> Option<(usize, usize)> {
    fn visit(
        value: &TermRef,
        depth: usize,
        memo: &mut HashMap<*const Term, (usize, usize)>,
    ) -> Option<(usize, usize)> {
        if depth > 128 || memo.len() >= 2_000_000 {
            return None;
        }
        let key = Rc::as_ptr(value);
        if let Some(summary) = memo.get(&key) {
            return Some(*summary);
        }
        let mut height = 0;
        let mut occurrences = 1_usize;
        let children: &[TermRef] = match value.as_ref() {
            Term::Typ(child) => std::slice::from_ref(child),
            Term::Ctr { args, .. } => args,
            Term::Rfl => &[],
            _ => return None,
        };
        for child in children {
            let (child_height, child_occurrences) = visit(child, depth + 1, memo)?;
            height = height.max(child_height.checked_add(1)?);
            occurrences = occurrences.checked_add(child_occurrences)?;
            if height > 128 || occurrences > 2_000_000 {
                return None;
            }
        }
        memo.insert(key, (height, occurrences));
        Some((height, occurrences))
    }
    visit(root, 0, &mut HashMap::new())
}

#[test]
fn shared_subtree_still_counts_its_deepest_path() {
    let checked = checked_book();
    let shared = wrap(term(Term::Rfl), 2);
    // The shared subtree is visited at depth 1 before it is visited again
    // through the left branch, as input inspection uses a LIFO stack.
    let at_limit = pair(&wrap(Rc::clone(&shared), 125), &shared);
    let past_limit = pair(&wrap(Rc::clone(&shared), 126), &shared);
    assert_eq!(guarded_summary(&at_limit).map(|x| x.0), Some(128));
    assert_eq!(guarded_summary(&past_limit), None);
    assert_eq!(
        input_error(&checked, at_limit),
        "undefined definition missing"
    );
    assert!(naive_unique_depth(&past_limit));
    assert_eq!(
        input_error(&checked, past_limit),
        "kernel input nesting limit exhausted; proof not accepted"
    );
}

#[test]
fn shared_subtrees_are_charged_per_occurrence() {
    let checked = checked_book();
    let mut repeated = term(Term::Rfl);
    for _ in 0..19 {
        repeated = pair(&repeated, &repeated);
    }
    // 1,048,575 occurrences, represented by just 20 distinct Rc nodes.
    assert_eq!(guarded_summary(&repeated), Some((19, 1_048_575)));
    assert_eq!(
        input_error(&checked, Rc::clone(&repeated)),
        "undefined definition missing"
    );
    // 2,097,151 occurrences, represented by just 21 distinct Rc nodes.
    let over_budget = pair(&repeated, &repeated);
    assert_eq!(guarded_summary(&over_budget), None);
    assert!(naive_unique_depth(&over_budget));
    assert_eq!(
        input_error(&checked, over_budget),
        "kernel input size limit exhausted; proof not accepted"
    );
}

#[test]
fn input_errors_keep_traversal_order() {
    let checked = checked_book();
    let deep = wrap(term(Term::Rfl), 128);
    let hole = term(Term::Hole("pending".into()));
    assert_eq!(guarded_summary(&pair(&deep, &hole)), None);
    assert_eq!(guarded_summary(&pair(&hole, &deep)), None);
    assert_eq!(
        input_error(&checked, pair(&deep, &hole)),
        "unfinished proof hole ?pending"
    );
    assert_eq!(
        input_error(&checked, pair(&hole, &deep)),
        "kernel input nesting limit exhausted; proof not accepted"
    );
}

// Recreate the earlier direct continuation cut, then expand its local binding.
// This is a research construction of core terms, not source-pattern lowering.
fn expanded_original() -> TermRef {
    fn cut(value: &TermRef, k: usize) -> TermRef {
        if let Term::Lam { id, body, .. } = value.as_ref()
            && let Term::App(f, x) = body.as_ref()
            && matches!(f.as_ref(), Term::Var { id, .. } if *id == k)
            && matches!(x.as_ref(), Term::Var { id: x, .. } if x == id)
        {
            return Rc::clone(f);
        }
        match value.as_ref() {
            Term::Mat {
                constructor,
                arm,
                fallback,
            } => term(Term::Mat {
                constructor: constructor.clone(),
                arm: cut(arm, k),
                fallback: Rc::clone(fallback),
            }),
            Term::Lam { name, id, body } => term(Term::Lam {
                name: name.clone(),
                id: *id,
                body: cut(body, k),
            }),
            _ => Rc::clone(value),
        }
    }
    let book = teamy_bend::syntax::load("tests/fixtures/cpu-compact-patterns/helpers.bend")
        .expect("fixture loads");
    let body = |name: &str| {
        book.declarations
            .iter()
            .find_map(|decl| match decl {
                Declaration::Def(def) if def.name == name => def.body.clone(),
                _ => None,
            })
            .expect("helper definition")
    };
    let prefix = body("prefix");
    let Term::Lam {
        id, body: inner, ..
    } = prefix.as_ref()
    else {
        panic!("expected continuation binder");
    };
    kernel::substitute(&cut(inner, *id), *id, &body("suffix"))
}

fn expanded_metrics(root: &TermRef) -> (usize, usize) {
    let mut stack = vec![(root, 0_usize)];
    let mut height = 0;
    let mut nodes = 0;
    while let Some((value, depth)) = stack.pop() {
        height = height.max(depth);
        nodes += 1;
        match value.as_ref() {
            Term::Lam { body, .. } => stack.push((body, depth + 1)),
            Term::App(a, b) => stack.extend([(a, depth + 1), (b, depth + 1)]),
            Term::Mat { arm, fallback, .. } => {
                stack.extend([(arm, depth + 1), (fallback, depth + 1)]);
            }
            Term::Ctr { args, .. } => {
                stack.extend(args.iter().map(|arg| (arg, depth + 1)));
            }
            Term::Var { .. } | Term::Ref(_) | Term::Efq => {}
            _ => panic!("unexpected core term in original matcher"),
        }
    }
    (height, nodes)
}

#[test]
fn expanded_string_matcher_is_still_an_overdepth_input() {
    let checked = checked_book();
    let expanded = expanded_original();
    assert_eq!(expanded_metrics(&expanded), (138, 610));
    assert_eq!(guarded_summary(&expanded), None);
    assert_eq!(
        input_error(&checked, expanded),
        "kernel input nesting limit exhausted; proof not accepted"
    );
}
