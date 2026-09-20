// SPDX-License-Identifier: MPL-2.0
use super::executable::BuiltinForeign;
use super::executable::ForeignTarget;
use super::executable::NumericIntrinsic;
use super::executable::OpaqueType;
use super::load;
use super::load_executable;
use super::parse;
use crate::kernel::Declaration;
use crate::kernel::Quant;
use crate::kernel::check_book;
use crate::kernel::check_executable;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static NEXT: AtomicUsize = AtomicUsize::new(0);

#[test]
fn opaque_chan_metadata_requires_exact_origin_signature_and_declaration() {
    use crate::kernel::Term;
    use crate::kernel::term;
    use std::rc::Rc;

    let fixture = Fixture::new();
    let path = fixture.write("main.bend", "import Base\n");
    let source = load_executable(&path).unwrap();
    assert_eq!(source.opaque.len(), 1);
    assert_eq!(source.opaque["Chan"], OpaqueType::Chan);
    assert!(!source.foreign.contains_key("Chan") && !source.numeric.contains_key("Chan"));
    assert!(!source.book.declarations.iter().any(|declaration| {
        matches!(declaration, Declaration::Adt(datatype) if datatype.name == "Chan")
    }));
    check_executable(&source).unwrap();
    check_book(&source.book)
        .expect_err("executable assumptions cannot produce a strict proof token");

    for mutation in [
        "order",
        "origin",
        "missing",
        "extra",
        "arity",
        "quantity",
        "parameter-quantity",
        "id",
        "domain",
        "parameter-domain",
        "result",
        "body",
        "foreign",
        "unsafe",
    ] {
        let mut source = load_executable(&path).unwrap();
        match mutation {
            "order" => {
                let index = source.book.declarations.iter().position(|declaration| {
                    matches!(declaration, Declaration::Def(definition) if definition.name == "Chan")
                }).unwrap();
                let declaration = source.book.declarations.remove(index);
                source.book.declarations.push(declaration);
            }
            "origin" => {
                source.base_names.remove("Chan");
            }
            "missing" => {
                source.opaque.remove("Chan");
            }
            "extra" => {
                source.opaque.insert("absent".into(), OpaqueType::Chan);
            }
            _ => {
                let declaration = source
                    .book
                    .declarations
                    .iter_mut()
                    .find_map(|declaration| match declaration {
                        Declaration::Def(definition) if definition.name == "Chan" => {
                            Some(definition)
                        }
                        _ => None,
                    })
                    .unwrap();
                match mutation {
                    "arity" => declaration.parameters.clear(),
                    "parameter-quantity" => declaration.parameters[0].quant = Quant::Lone,
                    "id" => declaration.parameters[0].id += 1,
                    "parameter-domain" => {
                        declaration.parameters[0].ty =
                            term(Term::Typ(term(Term::Qua(Quant::Many))));
                    }
                    "body" => declaration.body = Some(term(Term::Ref("U32".into()))),
                    "foreign" => declaration.foreign = true,
                    "unsafe" => declaration.unsafe_ = true,
                    _ => {
                        let Term::All {
                            quant,
                            domain,
                            body,
                            ..
                        } = Rc::make_mut(&mut declaration.ty)
                        else {
                            panic!("opaque telescope");
                        };
                        match mutation {
                            "quantity" => *quant = Quant::Lone,
                            "domain" => *domain = term(Term::Typ(term(Term::Qua(Quant::Many)))),
                            "result" => *body = term(Term::Typ(term(Term::Qua(Quant::Lone)))),
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }
        check_executable(&source).expect_err(mutation);
    }
}

#[test]
fn channel_contracts_keep_exact_quantities_and_erased_source_arity() {
    let fixture = Fixture::new();
    let path = fixture.write("main.bend", "import Base\n");
    let source = load_executable(path).unwrap();
    let checked = check_executable(&source).unwrap();
    for (name, builtin, arity, expected) in [
        (
            "Chan.new",
            BuiltinForeign::ChanNew,
            2,
            "@-A:Type -> @room:U32 -> IO(Chan(A))",
        ),
        (
            "Chan.send",
            BuiltinForeign::ChanSend,
            3,
            "@-A:Type -> @chan:Chan(A) -> @value:A -> IO(Bool)",
        ),
        (
            "Chan.recv",
            BuiltinForeign::ChanRecv,
            2,
            "@-A:Type -> @chan:Chan(A) -> IO(Maybe<&1, A>)",
        ),
        (
            "Chan.close",
            BuiltinForeign::ChanClose,
            2,
            "@-A:Type -> @chan:Chan(A) -> IO(Unit)",
        ),
    ] {
        assert_eq!(source.foreign[name].builtin, Some(builtin));
        assert_eq!(source.foreign[name].declared_arity, arity);
        assert_eq!(checked.definition_type(name).unwrap().to_string(), expected);
        let declaration = source
            .book
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Def(definition) if definition.name == name => Some(definition),
                _ => None,
            })
            .unwrap();
        assert_eq!(declaration.parameters[0].quant, Quant::None);
        assert!(
            declaration.parameters[1..]
                .iter()
                .all(|parameter| parameter.quant == Quant::Lone)
        );
    }
}

#[test]
fn local_opaque_names_never_acquire_bundled_origin() {
    let fixture = Fixture::new();
    fixture.write("other.bend", "law Chan:\n  for -A: Type\n  Data\n");
    let path = fixture.write("main.bend", "import other.bend as O\n");
    let source = load_executable(path).unwrap();
    assert!(source.opaque.is_empty());
    assert!(source.base_names.is_empty());
    assert!(
        check_executable(&source)
            .unwrap_err()
            .to_string()
            .contains("unfilled laws: other.Chan")
    );
    let path = fixture.write("fake.bend", "type Chan<-A: Type> is Data: Fake{}\n");
    let source = load_executable(path).unwrap();
    assert!(source.opaque.is_empty());
    check_executable(&source).unwrap();
    let path = fixture.write(
        "foreign.bend",
        "import Base\ndef chan_new(-A: Type, room: U32) -> IO(Chan(A)): import \"fake.js\"\n",
    );
    let source = load_executable(path).unwrap();
    assert_eq!(source.foreign["chan_new"].builtin, None);
}

#[test]
fn numeric_metadata_is_sealed_and_independent_from_foreign_contracts() {
    let fixture = Fixture::new();
    let path = fixture.write("main.bend", "import Base\n");
    let mut source = load_executable(&path).expect("load numeric contracts");
    assert_eq!(source.numeric.len(), 37);
    assert_eq!(source.numeric["F32.add"], NumericIntrinsic::Add);
    assert!(!source.foreign.contains_key("F32.add"));
    source
        .numeric
        .insert("F32.add".into(), NumericIntrinsic::Sub);
    assert!(
        check_executable(&source)
            .unwrap_err()
            .to_string()
            .contains("numeric contract metadata")
    );

    let mut source = load_executable(&path).unwrap();
    source.base_names.remove("F32");
    assert!(
        check_executable(&source)
            .unwrap_err()
            .to_string()
            .contains("actual Base datatypes")
    );

    let mut source = load_executable(&path).unwrap();
    source.numeric.remove("F32.add");
    let missing = check_executable(&source).unwrap_err().to_string();
    assert!(
        missing.contains("F32.add") && missing.contains("unfilled"),
        "{missing}"
    );

    let mut source = load_executable(&path).unwrap();
    let definition = source
        .book
        .declarations
        .iter_mut()
        .find_map(|declaration| match declaration {
            Declaration::Def(definition) if definition.name == "F32.add" => Some(definition),
            _ => None,
        })
        .unwrap();
    definition.parameters[0].quant = Quant::Many;
    if let crate::kernel::Term::All { quant, .. } = std::rc::Rc::make_mut(&mut definition.ty) {
        *quant = Quant::Many;
    }
    assert!(
        check_executable(&source)
            .unwrap_err()
            .to_string()
            .contains("invalid parameter")
    );
}

#[test]
fn constructor_tags_retain_original_spelling_before_qualification() {
    let fixture = Fixture::new();
    fixture.write("library.bend", "type Value is Data: Local.Tag{}\n");
    let path = fixture.write("main.bend", "import library.bend as L\n");
    let source = load_executable(path).expect("import dotted local constructor");
    assert_eq!(source.constructor_tags["library.Local.Tag"], "Local.Tag");
    assert!(source.numeric.is_empty());
    assert!(source.base_names.is_empty());
    check_executable(&source).expect("constructor metadata does not change proof rules");
}

#[test]
fn text_numeric_contracts_require_exact_quantities_and_maybe_payload() {
    use crate::kernel::Term;
    use crate::syntax::parse_term;
    use std::rc::Rc;

    let fixture = Fixture::new();
    let path = fixture.write("main.bend", "import Base\n");
    check_executable(&load_executable(&path).unwrap()).expect("unaltered numeric contracts check");
    for (name, replacement, diagnostic) in [
        ("F32.show", "show-quantity", "invalid parameter"),
        ("F32.read", "Maybe<&1, F32>", "invalid result type"),
        ("F32.read", "Maybe<&2, U32>", "invalid result type"),
        ("F32.read", "F32", "invalid result type"),
        ("F32.read", "excluded", "invalid result type"),
    ] {
        let mut source = load_executable(&path).unwrap();
        let definition = source
            .book
            .declarations
            .iter_mut()
            .find_map(|declaration| match declaration {
                Declaration::Def(definition) if definition.name == name => Some(definition),
                _ => None,
            })
            .unwrap();
        let Term::All { quant, body, .. } = Rc::make_mut(&mut definition.ty) else {
            panic!("unary numeric signature");
        };
        if replacement == "show-quantity" {
            definition.parameters[0].quant = Quant::Lone;
            *quant = Quant::Lone;
        } else if replacement == "excluded" {
            let Term::Adt { excluded, .. } = Rc::make_mut(body) else {
                panic!("actual Maybe family");
            };
            excluded.push("Some".into());
        } else {
            *body = parse_term(replacement).unwrap();
        }
        let error = check_executable(&source)
            .expect_err("altered numeric signature must fail")
            .to_string();
        assert!(error.contains(diagnostic), "{name}: {error}");
    }
    let mut source = load_executable(&path).unwrap();
    source.base_names.remove("Maybe");
    assert!(
        check_executable(&source)
            .unwrap_err()
            .to_string()
            .contains("actual Base datatypes")
    );
}

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-executable-loader-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).expect("create fixture");
        Self(path)
    }

    fn write(&self, name: &str, source: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, source).expect("write fixture");
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _result = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn executable_base_has_sealed_console_origins_and_checked_ordinary_helpers() {
    let fixture = Fixture::new();
    let path = fixture.write(
        "main.bend",
        "import Base\ndef main() -> IO(Unit): IO.pure(Unit, Unit{})\n",
    );
    let mut source = load_executable(&path).expect("execution Base loads");
    assert!(source.base_names.contains("IO"));
    assert!(source.base_names.contains("IO.OP"));
    assert!(!source.base_names.contains("main"));
    assert_eq!(source.foreign.len(), 10);
    for (name, builtin) in [
        ("IO.print", BuiltinForeign::Print),
        ("IO.write", BuiltinForeign::Write),
        ("IO.print_err", BuiltinForeign::PrintErr),
    ] {
        let definition = &source.foreign[name];
        assert_eq!(definition.builtin, Some(builtin));
        assert_eq!(definition.declared_arity, 1);
        assert_eq!(definition.parameters, ["text"]);
        assert_eq!(definition.imports.len(), 2);
        assert_eq!(definition.imports[0].target, ForeignTarget::C);
        assert_eq!(definition.imports[1].target, ForeignTarget::JavaScript);
    }
    check_book(&source.book).expect_err("foreign contracts cannot become a strict proof token");
    source.book.declarations.retain(
        |declaration| !matches!(declaration, Declaration::Def(definition) if definition.foreign || source.numeric.contains_key(&definition.name) || source.opaque.contains_key(&definition.name) || definition.name.starts_with("F32.") || definition.name.starts_with("IO.fork") || definition.name.starts_with("IO.join")),
    );
    check_book(&source.book)
        .expect("pure IO continuation helpers are checked without runtime assumptions");
    let strict = load(&path).expect("strict loading retains the pure Base");
    assert!(!strict.declarations.iter().any(
        |declaration| matches!(declaration, Declaration::Def(definition) if definition.name == "IO")
    ));
    check_book(&strict).expect_err("strict Base does not provide execution-only IO");
}

#[test]
fn foreign_paths_are_resolved_deduplicated_and_keep_source_local_symbols() {
    let fixture = Fixture::new();
    let js = fixture.write(
        "shared.js",
        "// retained foreign source; never executed by the loader\n",
    );
    let c = fixture.write("shared.c", "/* retained foreign source */\n");
    fixture.write("library.bend", "import Base\ndef Module.Echo(-A: Type, text: String) -> IO(Unit):\n  import \"./shared.js\"\n  import \"shared.js\"\n  import \"./shared.c\"\ndef Another(text: String) -> IO(Unit):\n  import \"shared.js\"\n");
    let path = fixture.write("main.bend", "import library.bend as L\n");
    let source = load_executable(path).expect("module with foreign imports");
    let definition = &source.foreign["library.Module.Echo"];
    assert_eq!(definition.local_symbol, "module_echo");
    assert_eq!(definition.declared_arity, 2);
    assert_eq!(definition.parameters, ["A", "text"]);
    assert_eq!(definition.builtin, None);
    assert_eq!(definition.imports.len(), 2);
    assert_eq!(
        definition.imports[0].path,
        fs::canonicalize(js).expect("canonical JS")
    );
    assert_eq!(
        definition.imports[1].path,
        fs::canonicalize(c).expect("canonical C")
    );
    assert_eq!(
        source.foreign["library.Another"].imports[0].path,
        definition.imports[0].path
    );
    assert!(!source.base_names.contains("library.Module.Echo"));
    let declaration = source
        .book
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            Declaration::Def(definition) if definition.name == "library.Module.Echo" => {
                Some(definition)
            }
            _ => None,
        })
        .expect("foreign definition event");
    assert!(declaration.foreign && declaration.body.is_none());
    assert_eq!(declaration.parameters[0].quant, Quant::None);
    assert_eq!(declaration.parameters[1].quant, Quant::Lone);
}

#[test]
fn a_foreign_law_fill_uses_its_actual_parameter_list() {
    let fixture = Fixture::new();
    let path = fixture.write("main.bend", "import Base\nlaw effect: @-A: Type -> @text: String -> IO(Unit)\ndef effect(A, text): import \"not-installed.js\"\n");
    let source = load_executable(path).expect("arrow signature and foreign fill");
    assert_eq!(source.foreign["effect"].declared_arity, 2);
    let event = source
        .book
        .declarations
        .iter()
        .rev()
        .find_map(|declaration| match declaration {
            Declaration::Def(definition) if definition.name == "effect" => Some(definition),
            _ => None,
        })
        .expect("foreign fill event");
    assert!(event.foreign);
    assert_eq!(event.parameters.len(), 2);
    assert_eq!(event.parameters[0].quant, Quant::None);
    assert_eq!(event.parameters[1].quant, Quant::Lone);
    assert!(source.foreign["effect"].imports[0].path.is_absolute());
}

#[test]
fn user_names_cannot_mint_base_origin_or_console_intrinsics() {
    let fixture = Fixture::new();
    let path = fixture.write(
        "main.bend",
        "type IO is Data:\n  Pretend{}\ndef IO.print() -> IO: import \"print.js\"\n",
    );
    let source =
        load_executable(path).expect("syntax alone does not validate foreign return types");
    assert!(source.base_names.is_empty());
    assert_eq!(source.foreign["IO.print"].builtin, None);
}

#[test]
fn bundled_scheduler_contracts_keep_source_order_exact_types_and_erased_arity() {
    let fixture = Fixture::new();
    let path = fixture.write("main.bend", "import Base\n");
    let source = load_executable(&path).expect("scheduler contracts load without executing them");
    let checked =
        check_executable(&source).expect("scheduler contracts use ordinary foreign checking");
    let mut previous = source.book.declarations.iter().position(|declaration| {
        matches!(declaration, Declaration::Def(definition) if definition.name == "IO.try")
    }).expect("IO.try precedes the upstream scheduler declarations");
    for (name, builtin, parameters, quantities, expected_type) in [
        (
            "IO.spawn",
            BuiltinForeign::Spawn,
            &["A", "act"][..],
            &[Quant::None, Quant::Lone][..],
            "@-A:Type -> @act:IO(A) -> IO(Unit)",
        ),
        (
            "IO.sleep",
            BuiltinForeign::Sleep,
            &["ms"][..],
            &[Quant::Lone][..],
            "@ms:U32 -> IO(Unit)",
        ),
        ("IO.now", BuiltinForeign::Now, &[][..], &[][..], "IO(Nat)"),
    ] {
        let metadata = &source.foreign[name];
        assert!(source.base_names.contains(name));
        assert_eq!(metadata.builtin, Some(builtin));
        assert_eq!(metadata.declared_arity, parameters.len());
        assert_eq!(metadata.parameters, parameters);
        assert_eq!(metadata.local_symbol, name.to_lowercase().replace('.', "_"));
        assert_eq!(metadata.imports.len(), 2);
        assert_eq!(metadata.imports[0].target, ForeignTarget::C);
        assert_eq!(metadata.imports[1].target, ForeignTarget::JavaScript);
        let position = source.book.declarations.iter().position(|declaration| {
            matches!(declaration, Declaration::Def(definition) if definition.name == name)
        }).unwrap();
        assert!(
            position > previous,
            "{name} keeps upstream declaration order"
        );
        previous = position;
        let Declaration::Def(definition) = &source.book.declarations[position] else {
            unreachable!()
        };
        assert!(definition.foreign && definition.body.is_none());
        assert_eq!(
            definition
                .parameters
                .iter()
                .map(|parameter| parameter.quant)
                .collect::<Vec<_>>(),
            quantities
        );
        assert_eq!(definition.ty.to_string(), expected_type);
        assert_eq!(
            checked.definition_type(name).unwrap().to_string(),
            expected_type
        );
    }
    check_book(&source.book).expect_err("scheduler contracts cannot mint a strict proof token");
    let strict = load(&path).expect("strict Base remains separate");
    check_book(&strict).expect("the pure bundled library remains strictly checked");
    for name in ["IO.spawn", "IO.sleep", "IO.now"] {
        assert!(!strict.declarations.iter().any(|declaration| {
            matches!(declaration, Declaration::Def(definition) if definition.name == name)
        }));
    }
}

#[test]
fn scheduler_spelling_and_user_foreign_symbols_do_not_mint_builtin_origin() {
    let fixture = Fixture::new();
    fixture.write(
        "library.bend",
        r#"import Base
def IO.spawn(-A: Type, act: IO(A)) -> IO(Unit): import "spawn.js"
def IO.sleep(ms: U32) -> IO(Unit): import "sleep.js"
def IO.now() -> IO(Nat): import "now.js"
"#,
    );
    let path = fixture.write("main.bend", "import library.bend as L\n");
    let duplicate =
        load_executable(&path).expect_err("importing Base reserves its exact scheduler names");
    assert!(
        duplicate.message.contains("duplicate definition: IO.spawn"),
        "{duplicate}"
    );
    fixture.write(
        "library.bend",
        r#"import Base
def io_spawn(-A: Type, act: IO(A)) -> IO(Unit): import "spawn.js"
def io_sleep(ms: U32) -> IO(Unit): import "sleep.js"
def io_now() -> IO(Nat): import "now.js"
"#,
    );
    let source = load_executable(path).expect("matching host symbols in an ordinary module load");
    for name in ["library.io_spawn", "library.io_sleep", "library.io_now"] {
        assert_eq!(source.foreign[name].builtin, None, "{name}");
        assert!(!source.base_names.contains(name), "{name}");
    }
    check_executable(&source).expect("user imports remain ordinary foreign assumptions");
    let path = fixture.write("forged.bend", "type IO is Data: Pretend{}\ndef IO.spawn() -> IO: import \"spawn.js\"\ndef IO.sleep() -> IO: import \"sleep.js\"\ndef IO.now() -> IO: import \"now.js\"\n");
    let forged = load_executable(path).expect("foreign gate is checked separately from parsing");
    assert!(forged.base_names.is_empty());
    assert!(
        forged
            .foreign
            .values()
            .all(|metadata| metadata.builtin.is_none())
    );
    assert!(
        check_executable(&forged)
            .unwrap_err()
            .to_string()
            .contains("actual Base IO")
    );
}

#[test]
fn native_scheduler_refusal_precedes_arity_and_console_payload_decoding() {
    for (name, body) in [
        (
            "IO.spawn",
            r#"def main() -> IO(Unit): IO.spawn(Unit, IO.print("child must not run"))"#,
        ),
        ("IO.sleep", "def main() -> IO(Unit): IO.sleep(1)"),
        ("IO.now", "def main() -> IO(Nat): IO.now()"),
    ] {
        let fixture = Fixture::new();
        let path = fixture.write("main.bend", &format!("import Base\n{body}\n"));
        let checked =
            check_executable(&load_executable(path).unwrap()).expect("scheduler action checks");
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let error = checked
            .run_main(&mut stdout, &mut stderr, &|| false)
            .expect_err("native scheduling remains explicitly unavailable")
            .to_string();
        assert!(
            error.contains(&format!("scheduler builtin {name}")),
            "{error}"
        );
        assert!(stdout.is_empty() && stderr.is_empty());
    }
}

#[test]
fn foreign_bodies_keep_strict_rejection_and_cannot_be_refilled() {
    let fixture = Fixture::new();
    let foreign = "def effect() -> Type: import \"effect.js\"\n";
    parse(foreign).expect_err("standalone proof parsing rejects foreign bodies");
    let path = fixture.write("strict.bend", foreign);
    load(path).expect_err("proof module loading rejects foreign bodies");
    for body in [
        "def effect() -> Type: import effect.js\n",
        "def effect() -> Type: import \"effect.txt\"\n",
        "def effect() -> Type: import \"effect.js\"\ndef effect() -> Type: Type\n",
        "law effect: Type\ndef effect(): import \"effect.js\"\ndef effect(): Type\n",
        "def effect() -> Type: import \"bad\0name.js\"\n",
    ] {
        let path = fixture.write("invalid.bend", body);
        load_executable(path).expect_err("malformed foreign contract or duplicate fill");
    }
}

#[test]
fn foreign_import_limits_apply_before_deduplication_and_paths_are_raw() {
    let fixture = Fixture::new();
    let imports = "import \"effect.js\"\n".repeat(128);
    let path = fixture.write("main.bend", &format!("def effect() -> Type:\n{imports}"));
    let source = load_executable(&path).expect("exact foreign import limit");
    assert_eq!(source.foreign["effect"].imports.len(), 1);
    fixture.write(
        "main.bend",
        &format!("def effect() -> Type:\n{imports}import \"effect.js\"\n"),
    );
    let error = load_executable(path).expect_err("duplicate paths still consume parsing budget");
    assert!(error.message.contains("resource limit"), "{error}");

    #[cfg(windows)]
    {
        fs::create_dir(fixture.0.join("nested")).expect("effect directory");
        let effect = fixture.write("nested/effect.js", "// raw foreign path\n");
        let path = fixture.write(
            "raw.bend",
            "def effect() -> Type: import \"nested\\effect.js\"\n",
        );
        let source = load_executable(path).expect("backslash is a path separator, not an escape");
        assert_eq!(
            source.foreign["effect"].imports[0].path,
            fs::canonicalize(effect).expect("effect path")
        );
    }
}

#[test]
fn foreign_template_instances_retain_the_declaring_module_and_actual_arity() {
    let fixture = Fixture::new();
    fixture.write(
        "library.bend",
        "import Base\ndef effect(~A: Type, x: A) -> IO(A): import \"effect.js\"\n",
    );
    let path = fixture.write(
        "main.bend",
        "import Base\nimport library.bend as L\ndef main() -> IO(Unit): L.effect(~Unit, Unit{})\n",
    );
    let source = load_executable(path).expect("closed foreign template");
    assert!(!source.foreign.contains_key("library.effect"));
    let instance = &source.foreign["library.effect~0"];
    assert_eq!(instance.declared_arity, 2);
    assert_eq!(instance.local_symbol, "effect");
    assert_eq!(instance.builtin, None);
    assert_eq!(
        instance.imports[0].path,
        fs::canonicalize(&fixture.0)
            .expect("fixture directory")
            .join("effect.js")
    );
}
