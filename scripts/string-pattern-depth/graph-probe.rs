// SPDX-License-Identifier: MPL-2.0
use teamy_bend::kernel::Declaration;
use teamy_bend::kernel::Term;
fn main() {
    let path = std::env::args_os().nth(1).unwrap();
    let book = match teamy_bend::syntax::load(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("{}", e.message);
            std::process::exit(1)
        }
    };
    for declaration in &book.declarations {
        let Declaration::Def(d) = declaration else {
            continue;
        };
        if !["is_hi", "f", "main"].contains(&d.name.as_str()) {
            continue;
        }
        let Some(body) = &d.body else { continue };
        let mut stack = vec![(body, 0)];
        let mut max = 0;
        let mut nodes = 0;
        while let Some((t, depth)) = stack.pop() {
            max = max.max(depth);
            nodes += 1;
            let n = depth + 1;
            match t.as_ref() {
                Term::All { domain, body, .. } => stack.extend([(domain, n), (body, n)]),
                Term::Lam { body, .. } | Term::Typ(body) => stack.push((body, n)),
                Term::Min(a, b) | Term::App(a, b) | Term::Ann(a, b) => {
                    stack.extend([(a, n), (b, n)])
                }
                Term::Adt { args, .. } | Term::Ctr { args, .. } => {
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
        println!("SUMMARY {} max_depth={} nodes={}", d.name, max, nodes);
        println!("GRAPH {d:?}");
        print!("CANONICAL ");
        canonical(body, &mut Vec::new());
        println!();
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
        _ => panic!("probe deliberately supports only observed original node tags"),
    }
}
