// SPDX-License-Identifier: MPL-2.0
use std::rc::Rc;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::term;
fn main() {
    let kind = std::env::args().nth(1).unwrap();
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(move || {
            let root = (0..100_000).fold(term(Term::Rfl), |x, _| term(Term::Typ(x)));
            match kind.as_str() {
                "unique" => drop(root),
                "shared" => {
                    let last = Rc::clone(&root);
                    drop(root);
                    drop(last);
                }
                "error" => {
                    fn early(_root: Rc<Term>) -> Result<(), ()> {
                        Err(())
                    }
                    let _ = early(root);
                }
                "unwind" => {
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let _owned = root;
                        panic!("unwind probe");
                    }));
                }
                _ => unreachable!(),
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
