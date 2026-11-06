// SPDX-License-Identifier: MPL-2.0
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::rc::Rc;

enum Node {
    Leaf,
    One(Rc<Node>),
    Pair(Rc<Node>, Rc<Node>),
}
fn take_edges(node: &mut Node, empty: &Rc<Node>, work: &mut Vec<Rc<Node>>) {
    match node {
        Node::Leaf => (),
        Node::One(a) => work.push(std::mem::replace(a, Rc::clone(empty))),
        Node::Pair(a, b) => {
            work.push(std::mem::replace(a, Rc::clone(empty)));
            work.push(std::mem::replace(b, Rc::clone(empty)));
        }
    }
}
impl Drop for Node {
    fn drop(&mut self) {
        if matches!(self, Node::Leaf) {
            return;
        }
        let empty = Rc::new(Node::Leaf);
        let mut work = Vec::new();
        take_edges(self, &empty, &mut work);
        while let Some(item) = work.pop() {
            if let Ok(mut child) = Rc::try_unwrap(item) {
                take_edges(&mut child, &empty, &mut work);
            }
        }
    }
}
fn chain(n: usize) -> Rc<Node> {
    (0..n).fold(Rc::new(Node::Leaf), |x, _| Rc::new(Node::One(x)))
}
fn error_case(weak: &mut std::rc::Weak<Node>) -> Result<(), ()> {
    let x = chain(100_000);
    *weak = Rc::downgrade(&x);
    Err(())
}
fn main() {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(|| {
            let a = chain(100_000);
            let weak = Rc::downgrade(&a);
            drop(a);
            assert!(weak.upgrade().is_none());
            let a = chain(100_000);
            let weak = Rc::downgrade(&a);
            let b = Rc::clone(&a);
            drop(a);
            assert!(weak.upgrade().is_some());
            drop(b);
            assert!(weak.upgrade().is_none());
            let a = chain(100_000);
            let weak = Rc::downgrade(&a);
            let tree = Rc::new(Node::Pair(Rc::clone(&a), a));
            drop(tree);
            assert!(weak.upgrade().is_none());
            let mut error_weak = std::rc::Weak::new();
            let r = error_case(&mut error_weak);
            assert!(r.is_err());
            assert!(error_weak.upgrade().is_none());
            let a = chain(100_000);
            let weak = Rc::downgrade(&a);
            let r = catch_unwind(AssertUnwindSafe(|| {
                let _owned = a;
                panic!("unwind probe");
            }));
            assert!(r.is_err());
            assert!(weak.upgrade().is_none());
        })
        .unwrap()
        .join()
        .unwrap();
    println!("unique/shared/diamond/error/unwind: pass; depth 100000; requested stack 65536");
}
