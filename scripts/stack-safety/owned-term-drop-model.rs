// SPDX-License-Identifier: MPL-2.0
use std::rc::Rc;
enum Term {
    Typ(Rc<Term>),
    Rfl,
}
impl Drop for Term {
    fn drop(&mut self) {}
}
fn owned_child(value: Rc<Term>) -> Rc<Term> {
    match Rc::try_unwrap(value) {
        Ok(Term::Typ(child)) => child,
        _ => Rc::new(Term::Rfl),
    }
}
fn main() {
    drop(owned_child(Rc::new(Term::Typ(Rc::new(Term::Rfl)))));
}
