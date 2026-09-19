//! Regression tests for proof acceptance, resource usage, and termination.
use std::rc::Rc;
use teamy_bend::kernel::AdtDecl;
use teamy_bend::kernel::Binder;
use teamy_bend::kernel::Book;
use teamy_bend::kernel::ConstructorDecl;
use teamy_bend::kernel::Declaration;
use teamy_bend::kernel::DefDecl;
use teamy_bend::kernel::Quant;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::TermRef;
use teamy_bend::kernel::arrows;
use teamy_bend::kernel::check_book;
use teamy_bend::kernel::lambdas;
use teamy_bend::kernel::term;

fn r(name: &str) -> TermRef {
    term(Term::Ref(name.into()))
}
fn v(name: &str, id: usize) -> TermRef {
    term(Term::Var {
        name: name.into(),
        id,
    })
}
fn c(name: &str, args: Vec<TermRef>) -> TermRef {
    term(Term::Ctr {
        name: name.into(),
        args,
    })
}
fn nat(n: usize) -> TermRef {
    (0..n).fold(c("Zero", vec![]), |n, _| c("Succ", vec![n]))
}
fn b(name: &str, id: usize, quant: Quant, ty: TermRef) -> Binder {
    Binder {
        name: name.into(),
        id,
        quant,
        ty,
    }
}
fn app(f: TermRef, args: Vec<TermRef>) -> TermRef {
    args.into_iter().fold(f, |f, x| term(Term::App(f, x)))
}
fn eq(left: TermRef, right: TermRef) -> TermRef {
    term(Term::Eql {
        left,
        right,
        ty: r("Nat"),
    })
}
fn def(name: &str, parameters: Vec<Binder>, result: TermRef, body: TermRef) -> Declaration {
    Declaration::Def(DefDecl {
        name: name.into(),
        ty: arrows(&parameters, result),
        parameters,
        body: Some(body),
        unsafe_: false,
        foreign: false,
    })
}
fn natural() -> Declaration {
    Declaration::Adt(AdtDecl {
        name: "Nat".into(),
        parameters: vec![],
        kind: term(Term::Typ(term(Term::Qua(Quant::Many)))),
        constructors: vec![
            ConstructorDecl {
                name: "Zero".into(),
                fields: vec![],
            },
            ConstructorDecl {
                name: "Succ".into(),
                fields: vec![b("pred", 0, Quant::Lone, r("Nat"))],
            },
        ],
    })
}
fn matcher(zero: TermRef, succ: TermRef) -> TermRef {
    term(Term::Mat {
        constructor: "Zero".into(),
        arm: zero,
        fallback: term(Term::Mat {
            constructor: "Succ".into(),
            arm: succ,
            fallback: term(Term::Efq),
        }),
    })
}
fn addition() -> Declaration {
    let n = b("n", 1, Quant::Lone, r("Nat"));
    let m = b("m", 2, Quant::Lone, r("Nat"));
    let p = b("p", 3, Quant::Lone, r("Nat"));
    let zero = lambdas(std::slice::from_ref(&m), v("m", 2));
    let succ = lambdas(
        &[p, m.clone()],
        c("Succ", vec![app(r("add"), vec![v("p", 3), v("m", 2)])]),
    );
    def("add", vec![n, m], r("Nat"), matcher(zero, succ))
}

#[test]
fn evaluates_structurally_recursive_addition() {
    let checked = check_book(&Book {
        declarations: vec![natural(), addition()],
    })
    .expect("recursive addition checks");
    let result = checked
        .evaluate("add", &[nat(3), nat(2)])
        .expect("addition evaluates");
    assert_eq!(result.to_string(), nat(5).to_string());
}

#[test]
fn accepts_induction_and_equality_rewrite() {
    let n = b("n", 4, Quant::Lone, r("Nat"));
    let p = b("p", 5, Quant::Lone, r("Nat"));
    let endpoint = b("_", 6, Quant::Lone, r("Nat"));
    let evidence = b(
        "e",
        7,
        Quant::Lone,
        eq(app(r("add"), vec![v("p", 5), nat(0)]), v("_", 6)),
    );
    let motive = lambdas(
        &[endpoint, evidence],
        eq(
            c("Succ", vec![app(r("add"), vec![v("p", 5), nat(0)])]),
            c("Succ", vec![v("_", 6)]),
        ),
    );
    let step = lambdas(
        &[p],
        term(Term::Rwt {
            evidence: app(r("add_zero"), vec![v("p", 5)]),
            motive,
            body: term(Term::Rfl),
        }),
    );
    let proof = def(
        "add_zero",
        vec![n],
        eq(app(r("add"), vec![v("n", 4), nat(0)]), v("n", 4)),
        matcher(term(Term::Rfl), step),
    );
    let checked = check_book(&Book {
        declarations: vec![natural(), addition(), proof],
    })
    .expect("induction proof checks");
    assert_eq!(
        checked
            .evaluate("add_zero", &[nat(4)])
            .expect("proof evaluates")
            .to_string(),
        "{==}"
    );
}

#[test]
fn rejects_false_reflexivity() {
    let result = check_book(&Book {
        declarations: vec![
            natural(),
            def("false", vec![], eq(nat(0), nat(1)), term(Term::Rfl)),
        ],
    });
    assert!(
        result
            .expect_err("false equality rejected")
            .to_string()
            .contains("reflexivity")
    );
}

#[test]
fn rejects_unfinished_laws_and_todo() {
    let law = Declaration::Def(DefDecl {
        name: "open".into(),
        parameters: vec![],
        ty: eq(nat(0), nat(0)),
        body: None,
        unsafe_: false,
        foreign: false,
    });
    assert!(
        check_book(&Book {
            declarations: vec![natural(), law]
        })
        .expect_err("open law rejected")
        .to_string()
        .contains("unfilled")
    );
    assert!(
        check_book(&Book {
            declarations: vec![
                natural(),
                def(
                    "hole",
                    vec![],
                    eq(nat(0), nat(0)),
                    term(Term::Hole("TODO".into()))
                )
            ]
        })
        .expect_err("hole rejected")
        .to_string()
        .contains("hole")
    );
}

#[test]
fn rejects_non_decreasing_recursion() {
    let n = b("n", 10, Quant::Lone, r("Nat"));
    let body = lambdas(std::slice::from_ref(&n), app(r("loop"), vec![v("n", 10)]));
    assert!(
        check_book(&Book {
            declarations: vec![natural(), def("loop", vec![n], r("Nat"), body)]
        })
        .expect_err("loop rejected")
        .to_string()
        .contains("decrease")
    );
}

#[test]
fn rejects_affine_contraction() {
    let n = b("n", 10, Quant::Lone, r("Nat"));
    let body = lambdas(
        std::slice::from_ref(&n),
        app(r("add"), vec![v("n", 10), v("n", 10)]),
    );
    assert!(
        check_book(&Book {
            declarations: vec![
                natural(),
                addition(),
                def("duplicate", vec![n], r("Nat"), body)
            ]
        })
        .expect_err("duplicate use rejected")
        .to_string()
        .contains("observed Many")
    );
}

#[test]
fn rejects_erased_scrutinee_and_function_contraction_kind() {
    let n = b("n", 10, Quant::None, r("Nat"));
    let body = matcher(
        nat(0),
        lambdas(&[b("p", 11, Quant::Lone, r("Nat"))], nat(0)),
    );
    assert!(
        check_book(&Book {
            declarations: vec![natural(), def("erased", vec![n], r("Nat"), body)]
        })
        .expect_err("erased evidence rejected")
        .to_string()
        .contains("erased scrutinee")
    );
    let function_type = arrows(&[b("x", 12, Quant::Lone, r("Nat"))], r("Nat"));
    let f = b("f", 13, Quant::Many, function_type);
    let body = lambdas(std::slice::from_ref(&f), nat(0));
    assert!(
        check_book(&Book {
            declarations: vec![natural(), def("copy_fn", vec![f], r("Nat"), body)]
        })
        .expect_err("functions cannot be reusable")
        .to_string()
        .contains("type mismatch")
    );
}

#[test]
fn runtime_rejects_wrong_arity_and_ill_typed_arguments() {
    let checked = check_book(&Book {
        declarations: vec![natural(), addition()],
    })
    .expect("addition checks");
    assert!(
        checked
            .evaluate("add", &[nat(1)])
            .expect_err("arity checked")
            .to_string()
            .contains("2 arguments")
    );
    checked
        .evaluate("add", &[term(Term::Rfl), nat(1)])
        .expect_err("arguments are type checked");
    checked
        .evaluate("missing", &[])
        .expect_err("unknown entry is rejected");
}

#[test]
fn law_fill_cannot_change_the_claim() {
    let claim = eq(nat(0), nat(1));
    let law = Declaration::Def(DefDecl {
        name: "claim".into(),
        parameters: vec![],
        ty: claim,
        body: None,
        unsafe_: false,
        foreign: false,
    });
    let forged = def("claim", vec![], eq(nat(0), nat(0)), term(Term::Rfl));
    assert!(
        check_book(&Book {
            declarations: vec![natural(), law, forged]
        })
        .expect_err("changed claim rejected")
        .to_string()
        .contains("preserve")
    );
}

#[test]
fn rejects_unsafe_and_foreign_assumptions() {
    for (unsafe_, foreign) in [(true, false), (false, true)] {
        let declaration = Declaration::Def(DefDecl {
            name: "escape".into(),
            parameters: vec![],
            ty: r("Nat"),
            body: Some(nat(0)),
            unsafe_,
            foreign,
        });
        assert!(
            check_book(&Book {
                declarations: vec![natural(), declaration]
            })
            .expect_err("escape rejected")
            .to_string()
            .contains("strict proof")
        );
    }
}

#[test]
fn equality_checks_both_endpoint_types() {
    let wrong = term(Term::Eql {
        left: term(Term::Typ(term(Term::Qua(Quant::Lone)))),
        right: nat(0),
        ty: r("Nat"),
    });
    check_book(&Book {
        declarations: vec![natural(), def("bad_type", vec![], wrong, term(Term::Rfl))],
    })
    .expect_err("equality endpoints must inhabit the stated type");
}

#[test]
fn simultaneous_let_accounts_for_resources_once() {
    use teamy_bend::kernel::LetBinding;
    let n = b("n", 10, Quant::Lone, r("Nat"));
    let body = lambdas(
        std::slice::from_ref(&n),
        term(Term::Let {
            bindings: vec![LetBinding {
                name: "twice".into(),
                id: 11,
                quant: Quant::Many,
                value: v("n", 10),
            }],
            body: app(r("add"), vec![v("twice", 11), v("twice", 11)]),
        }),
    );
    let checked = check_book(&Book {
        declarations: vec![
            natural(),
            addition(),
            def("double", vec![n], r("Nat"), body),
        ],
    })
    .expect("Data promotion certifies once");
    assert_eq!(
        checked
            .evaluate("double", &[nat(3)])
            .expect("evaluates")
            .to_string(),
        nat(6).to_string()
    );
}

#[test]
fn accepts_declared_law_followed_by_exact_proof() {
    let ty = eq(nat(0), nat(0));
    let law = DefDecl {
        name: "truth".into(),
        parameters: vec![],
        ty: Rc::clone(&ty),
        body: None,
        unsafe_: false,
        foreign: false,
    };
    let proof = DefDecl {
        body: Some(term(Term::Rfl)),
        ..law.clone()
    };
    check_book(&Book {
        declarations: vec![natural(), Declaration::Def(law), Declaration::Def(proof)],
    })
    .expect("filled law accepted");
}

#[test]
fn rejects_holes_even_in_beta_erased_arguments() {
    let ignored = b("ignored", 20, Quant::Lone, r("Nat"));
    let body = app(
        lambdas(&[ignored], term(Term::Ann(nat(0), r("Nat")))),
        vec![term(Term::Hole("TODO".into()))],
    );
    let error = check_book(&Book {
        declarations: vec![natural(), def("hidden", vec![], r("Nat"), body)],
    })
    .expect_err("source holes cannot disappear through beta reduction");
    assert!(error.to_string().contains("unfinished proof hole"));
}

#[test]
fn rejects_deep_input_before_recursive_traversal() {
    let checked = check_book(&Book {
        declarations: vec![natural(), addition()],
    })
    .expect("addition checks");
    let error = checked
        .evaluate("add", &[nat(300), nat(0)])
        .expect_err("deep input must fail without overflowing the stack");
    assert!(error.to_string().contains("input nesting limit"));
}

#[test]
fn normalizes_inside_returned_functions() {
    let x = b("x", 20, Quant::Lone, r("Nat"));
    let result_type = arrows(std::slice::from_ref(&x), r("Nat"));
    let body = lambdas(&[x], app(r("add"), vec![nat(1), nat(2)]));
    let checked = check_book(&Book {
        declarations: vec![
            natural(),
            addition(),
            def("constant", vec![], result_type, body),
        ],
    })
    .expect("function-valued result checks");
    let result = checked
        .evaluate("constant", &[])
        .expect("result normalizes under its lambda");
    assert_eq!(result.to_string(), format!("x => {}", nat(3)));
}

#[test]
fn reusable_datatype_cannot_hide_a_live_function_field() {
    let function = arrows(&[b("n", 20, Quant::Lone, r("Nat"))], r("Nat"));
    let hidden = Declaration::Adt(AdtDecl {
        name: "Hidden".into(),
        parameters: vec![],
        kind: term(Term::Typ(term(Term::Qua(Quant::Many)))),
        constructors: vec![ConstructorDecl {
            name: "Hide".into(),
            fields: vec![b("f", 21, Quant::Lone, function)],
        }],
    });
    let error = check_book(&Book {
        declarations: vec![natural(), hidden],
    })
    .expect_err("a Data constructor cannot own a function");
    assert!(error.to_string().contains("type mismatch"));
}

#[test]
fn rewrite_motive_must_reconstruct_the_claim() {
    let endpoint = b("_", 20, Quant::Lone, r("Nat"));
    let witness = b("e", 21, Quant::Lone, eq(nat(0), v("_", 20)));
    let motive = lambdas(&[endpoint, witness], eq(nat(0), nat(0)));
    let proof = term(Term::Rwt {
        evidence: term(Term::Ann(term(Term::Rfl), eq(nat(0), nat(0)))),
        motive,
        body: term(Term::Rfl),
    });
    let error = check_book(&Book {
        declarations: vec![
            natural(),
            def("false_rewrite", vec![], eq(nat(0), nat(1)), proof),
        ],
    })
    .expect_err("a constant true motive cannot establish an unrelated false goal");
    assert!(error.to_string().contains("type mismatch"));
}
