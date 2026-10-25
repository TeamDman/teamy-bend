// SPDX-License-Identifier: MPL-2.0
// Research-only manual Book transformation; not a production lowering pass.
use std::rc::Rc;
use teamy_bend::kernel::Book;
use teamy_bend::kernel::Declaration;
use teamy_bend::kernel::DefDecl;
use teamy_bend::kernel::LetBinding;
use teamy_bend::kernel::Quant;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
use teamy_bend::kernel::term;
fn definition(book: &Book, name: &str) -> DefDecl {
    book.declarations
        .iter()
        .find_map(|d| match d {
            Declaration::Def(d) if d.name == name && d.body.is_some() => Some(d.clone()),
            _ => None,
        })
        .unwrap()
}
fn cut(t: &TermRef, k: usize, n: &mut usize) -> TermRef {
    if let Term::Lam { id, body, .. } = t.as_ref() {
        if let Term::App(f, x) = body.as_ref() {
            if matches!(f.as_ref(),Term::Var{id,..} if *id==k)
                && matches!(x.as_ref(),Term::Var{id:x,..} if x==id)
            {
                *n += 1;
                return Rc::clone(f);
            }
        }
    }
    match t.as_ref() {
        Term::Mat {
            constructor,
            arm,
            fallback,
        } => term(Term::Mat {
            constructor: constructor.clone(),
            arm: cut(arm, k, n),
            fallback: Rc::clone(fallback),
        }),
        Term::Lam { name, id, body } => term(Term::Lam {
            name: name.clone(),
            id: *id,
            body: cut(body, k, n),
        }),
        _ => Rc::clone(t),
    }
}
fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let mut book = teamy_bend::syntax::load(&args[1]).unwrap();
    let prefix = definition(&book, "prefix");
    let suffix = definition(&book, "suffix");
    let Term::Lam { name, id, body } = prefix.body.as_ref().unwrap().as_ref() else {
        panic!("prefix binder")
    };
    let mut n = 0;
    let body = if args.get(2).is_some_and(|a| a == "--call") {
        Rc::clone(body)
    } else {
        cut(body, *id, &mut n)
    };
    println!("cut_count={n}");
    let compact = term(Term::Let {
        bindings: vec![LetBinding {
            quant: Quant::Lone,
            name: name.clone(),
            id: *id,
            value: term(Term::Ann(suffix.body.clone().unwrap(), suffix.ty.clone())),
        }],
        body,
    });
    println!("compact_metrics={:?}", metrics(&compact));
    let Term::Let { bindings, body } = compact.as_ref() else {
        unreachable!()
    };
    let Term::Ann(rhs, _) = bindings[0].value.as_ref() else {
        unreachable!()
    };
    let expanded = teamy_bend::kernel::substitute(body, bindings[0].id, rhs);
    println!("expanded_metrics={:?}", metrics(&expanded));
    print!("expanded_canonical=");
    canonical(&expanded, &mut Vec::new());
    println!();

    for d in &mut book.declarations {
        if let Declaration::Def(d) = d {
            if d.name == "is_hi" && d.body.is_some() {
                d.body = Some(compact.clone());
            }
        }
    }
    book.declarations.retain(
        |d| !matches!(d,Declaration::Def(d) if ["prefix","suffix"].contains(&d.name.as_str())),
    );
    let checked = match teamy_bend::kernel::check_book(&book) {
        Ok(c) => {
            println!("check=accepted");
            c
        }
        Err(e) => {
            println!("check=refused:{e}");
            return;
        }
    };
    for name in ["main", "is_hi", "open"] {
        match checked.normalize_definition(name) {
            Ok(v) => {
                println!("normalize:{name}:ok:{v}");
                print!("canonical:{name}:");
                canonical(&v, &mut Vec::new());
                println!();
            }
            Err(e) => println!("normalize:{name}:error:{e}"),
        }
    }
    for input in ["\"hi\"", "\"ho\"", "\"h\"", "\"\"", "\"xi\"", "\"hii\""] {
        let x = teamy_bend::syntax::parse_term(input).unwrap();
        match checked.evaluate("is_hi", &[x.clone()]) {
            Ok(v) => println!("evaluate:{input}:ok:{v}"),
            Err(e) => println!("evaluate:{input}:error:{e}"),
        }
        match checked.evaluate_data("is_hi", &[x]) {
            Ok(v) => println!("data:{input}:ok:{v}"),
            Err(e) => println!("data:{input}:error:{e}"),
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
        Term::Ann(v, _) => canonical(v, binders),
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
