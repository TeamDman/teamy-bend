// SPDX-License-Identifier: MPL-2.0
use std::rc::Rc;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
use teamy_bend::kernel::term;
fn owned_child(value: TermRef) -> TermRef {
    match Rc::try_unwrap(value) {
        Ok(Term::Typ(child)) => child,
        _ => term(Term::Rfl),
    }
}
fn main() {
    assert!(matches!(
        owned_child(term(Term::Typ(term(Term::Rfl)))).as_ref(),
        Term::Rfl
    ));
}
