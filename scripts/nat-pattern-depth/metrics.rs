// SPDX-License-Identifier: MPL-2.0
// Read-only measurement; no parser or kernel limits are bypassed.
use teamy_bend::kernel::Declaration;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let book = match teamy_bend::syntax::load(&args[1]) {
        Ok(b) => b,
        Err(e) => {
            println!("load:error:{e}");
            return;
        }
    };
    let body = book
        .declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Def(d) if d.name == args[2] => d.body.clone(),
            _ => None,
        })
        .expect("definition");
    println!("loaded_metrics={:?}", metrics(&body));
    print!("loaded_canonical=");
    canonical(&body, &mut Vec::new());
    println!();
    let checked = match teamy_bend::kernel::check_book(&book) {
        Ok(b) => b,
        Err(e) => {
            println!("check:error:{e}");
            return;
        }
    };
    println!("check=accepted");
    for name in [&args[2], "main"] {
        match checked.normalize_definition(name) {
            Ok(v) => {
                println!("normalized_metrics:{name}={:?}", metrics(&v));
                print!("normalized_canonical:{name}=");
                canonical(&v, &mut Vec::new());
                println!();
            }
            Err(e) => println!("normalize:{name}:error:{e}"),
        }
    }
}
fn canonical(t: &teamy_bend::kernel::TermRef, binders: &mut Vec<usize>) {
    match t.as_ref() {
        Term::Var { id, .. } => print!(
            "Var:{};",
            binders
                .iter()
                .rev()
                .position(|x| x == id)
                .expect("closed binder")
        ),
        Term::Ref(k) => print!("Ref:{k};"),
        Term::Lam { id, body, .. } => {
            print!("Lam;");
            binders.push(*id);
            canonical(body, binders);
            binders.pop();
        }
        Term::App(f, x) => {
            print!("App;");
            canonical(f, binders);
            canonical(x, binders);
        }
        Term::Mat {
            constructor,
            arm,
            fallback,
        } => {
            print!("Mat:{constructor};");
            canonical(arm, binders);
            canonical(fallback, binders);
        }
        Term::Ctr { name, args } => {
            print!("Ctr:{name}:{};", args.len());
            for a in args {
                canonical(a, binders);
            }
        }
        Term::Efq => print!("Efq;"),
        Term::Ann(v, ty) => {
            print!("Ann;");
            canonical(v, binders);
            canonical(ty, binders);
        }
        _ => panic!("probe deliberately supports only observed original node tags"),
    }
}

fn metrics(root: &TermRef) -> (usize, usize) {
    let mut stack = vec![(root, 0)];
    let mut depth = 0;
    let mut nodes = 0;
    while let Some((v, d)) = stack.pop() {
        depth = depth.max(d);
        nodes += 1;
        let n = d + 1;
        match v.as_ref() {
            Term::All { domain, body, .. } => stack.extend([(domain, n), (body, n)]),
            Term::Lam { body, .. } | Term::Typ(body) => stack.push((body, n)),
            Term::Min(a, b) | Term::App(a, b) | Term::Ann(a, b) => stack.extend([(a, n), (b, n)]),
            Term::Ctr { args, .. } | Term::Adt { args, .. } => {
                stack.extend(args.iter().map(|a| (a, n)))
            }
            Term::Mat { arm, fallback, .. } => stack.extend([(arm, n), (fallback, n)]),
            Term::Eql { left, right, ty } => stack.extend([(left, n), (right, n), (ty, n)]),
            Term::Rwt {
                evidence,
                motive,
                body,
            } => stack.extend([(evidence, n), (motive, n), (body, n)]),
            Term::Let { bindings, body } => {
                stack.push((body, n));
                stack.extend(bindings.iter().map(|b| (&b.value, n)));
            }
            _ => {}
        }
    }
    (depth, nodes)
}
