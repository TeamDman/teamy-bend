// SPDX-License-Identifier: MPL-2.0
use super::DefinitionBody;
use super::ExecutableProgram;
use super::Expression;
use super::ExpressionKind;
use crate::kernel::ExecutableBook;
use crate::kernel::ExecutableEntry;
use crate::kernel::Quant;
use crate::kernel::Term;
use crate::kernel::check_executable;
use crate::syntax::executable::BuiltinForeign;
use crate::syntax::executable::NumericIntrinsic;
use crate::syntax::load_executable;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-elaboration-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, source: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, source).unwrap();
        path
    }

    fn checked(&self, source: &str) -> ExecutableBook {
        check_executable(&load_executable(self.write("main.bend", source)).unwrap()).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed = fs::remove_dir_all(&self.0);
    }
}

fn ordinary<'a>(program: &'a ExecutableProgram, name: &str) -> &'a Expression {
    let DefinitionBody::Ordinary(body) = &program.definitions[name].body else {
        panic!("{name} must have an ordinary body")
    };
    body
}

fn expressions(root: &Expression) -> Vec<&Expression> {
    let mut pending = vec![root];
    let mut result = Vec::new();
    while let Some(expression) = pending.pop() {
        result.push(expression);
        match &expression.kind {
            ExpressionKind::Lambda { body, .. } => pending.push(body),
            ExpressionKind::Apply {
                function, argument, ..
            } => pending.extend([function.as_ref(), argument.as_ref()]),
            ExpressionKind::Constructor { fields, .. } => {
                pending.extend(fields.iter().map(|field| &field.value));
            }
            ExpressionKind::Match { arm, fallback, .. } => {
                pending.extend([arm.as_ref(), fallback.as_ref()]);
            }
            ExpressionKind::Let { bindings, body } => {
                pending.extend(bindings.iter().map(|binding| &binding.value));
                pending.push(body);
            }
            ExpressionKind::Erased
            | ExpressionKind::Variable(_)
            | ExpressionKind::Definition(_)
            | ExpressionKind::Absurd { .. } => (),
        }
    }
    result
}

fn assert_scope(expression: &Expression, scope: &BTreeSet<usize>) {
    match &expression.kind {
        ExpressionKind::Variable(id) => {
            assert!(scope.contains(id), "unbound lowered variable {id}");
        }
        ExpressionKind::Lambda { parameter, body } => {
            let mut inner = scope.clone();
            assert!(inner.insert(parameter.id));
            assert_scope(body, &inner);
        }
        ExpressionKind::Apply {
            function, argument, ..
        } => {
            assert_scope(function, scope);
            assert_scope(argument, scope);
        }
        ExpressionKind::Constructor { fields, .. } => {
            for field in fields {
                assert_scope(&field.value, scope);
            }
        }
        ExpressionKind::Match { arm, fallback, .. } => {
            assert_scope(arm, scope);
            assert_scope(fallback, scope);
        }
        ExpressionKind::Let { bindings, body } => {
            let mut inner = scope.clone();
            for binding in bindings {
                assert_scope(&binding.value, scope);
                assert!(inner.insert(binding.binder.id));
            }
            assert_scope(body, &inner);
        }
        ExpressionKind::Erased | ExpressionKind::Definition(_) | ExpressionKind::Absurd { .. } => {}
    }
}

fn lowered(source: &str) -> ExecutableProgram {
    let fixture = Fixture::new();
    let checked = fixture.checked(source);
    let program = checked.lower_for_javascript().unwrap();
    for definition in program.definitions.values() {
        if let DefinitionBody::Ordinary(body) = &definition.body {
            assert_scope(body, &BTreeSet::new());
        }
    }
    program
}

#[test]
fn console_io_is_typed_and_reachability_omits_unused_foreign_imports() {
    let program = lowered(
        r#"
import Base
def unused() -> IO(Unit):
  import "not-installed.js"
def main() -> IO(Unit):
  do IO<Unit>:
    Unit <- IO.write("hello")
    IO.print(" world")
"#,
    );
    assert_eq!(program.entry, ExecutableEntry::Io);
    assert!(!program.definitions.contains_key("unused"));
    assert!(!program.definitions.contains_key("IO.print_err"));
    for (name, builtin) in [
        ("IO.write", BuiltinForeign::Write),
        ("IO.print", BuiltinForeign::Print),
    ] {
        let definition = &program.definitions[name];
        assert_eq!(definition.parameters.len(), 1);
        assert_eq!(definition.parameters[0].quant, Quant::Lone);
        let DefinitionBody::Foreign(metadata) = &definition.body else {
            panic!("expected foreign console body")
        };
        assert_eq!(metadata.builtin, Some(builtin));
    }
    let bind = &program.definitions["IO.bind"];
    assert_eq!(
        bind.parameters
            .iter()
            .map(|parameter| parameter.quant)
            .collect::<Vec<_>>(),
        [Quant::None, Quant::None, Quant::Lone, Quant::Lone]
    );
    assert!(
        expressions(ordinary(&program, "IO.bind"))
            .iter()
            .any(|expression| {
                matches!(&expression.kind, ExpressionKind::Lambda { parameter, .. }
            if parameter.name == "R" && parameter.quant == Quant::None)
            })
    );
    assert!(program.base_names.contains("IO"));
    assert!(program.datatypes.contains_key("String"));
}

#[test]
fn constructors_and_matches_retain_dependent_fields_and_erasure() {
    let program = lowered(
        r"
import Base
type Box<-A: Type> is Type:
  MkBox{-tag: Nat, val: A}
def open(-A: Type, box: Box<A>) -> A:
  match box:
    case MkBox{tag, value}: value
def main() -> Nat: open(Nat, MkBox{7n, 2n})
",
    );
    let nodes = expressions(ordinary(&program, "main"));
    let fields = nodes
        .iter()
        .find_map(|expression| match &expression.kind {
            ExpressionKind::Constructor {
                owner,
                name,
                fields,
            } if name == "MkBox" => {
                assert_eq!(owner, "Box");
                Some(fields)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(fields[0].binder.name, "tag");
    assert_eq!(fields[0].binder.quant, Quant::None);
    assert!(matches!(fields[0].value.kind, ExpressionKind::Erased));
    assert_eq!(fields[1].binder.name, "val");
    assert_eq!(fields[1].binder.ty.to_string(), "Nat");
    let nodes = expressions(ordinary(&program, "open"));
    let fields = nodes
        .iter()
        .find_map(|expression| match &expression.kind {
            ExpressionKind::Match { owner, fields, .. } => {
                assert_eq!(owner, "Box");
                Some(fields)
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(
        fields
            .iter()
            .map(|field| (field.name.as_str(), field.quant))
            .collect::<Vec<_>>(),
        [("tag", Quant::None), ("val", Quant::Lone)]
    );
    assert!(nodes.iter().any(|expression| matches!(&expression.kind,
        ExpressionKind::Absurd { owner, .. } if owner == "Box")));
}

#[test]
fn foreign_arrow_arity_live_type_and_proof_values_remain_distinct_from_erased_slots() {
    let program = lowered(
        r#"
import Base
law foreign_identity:
  @-A: Type -> @x: A -> @tag: Type -> @proof: {0n == 0n : Nat} -> IO(A)
def foreign_identity(A, x, tag, proof):
  import "identity.js"
def main() -> IO(Nat): foreign_identity(Nat, 2n, Nat, {==})
"#,
    );
    let foreign = &program.definitions["foreign_identity"];
    assert_eq!(foreign.parameters.len(), 4);
    assert_eq!(foreign.parameters[0].quant, Quant::None);
    assert_eq!(foreign.parameters[2].quant, Quant::Lone);
    assert_eq!(foreign.parameters[3].quant, Quant::Lone);
    let mut live_nulls = 0;
    let mut erased_arguments = 0;
    for expression in expressions(ordinary(&program, "main")) {
        if let ExpressionKind::Apply {
            argument, quant, ..
        } = &expression.kind
            && matches!(argument.kind, ExpressionKind::Erased)
        {
            if *quant == Quant::None {
                erased_arguments += 1;
            } else {
                live_nulls += 1;
            }
        }
    }
    assert_eq!((erased_arguments, live_nulls), (1, 2));
}

#[test]
fn imported_constructor_tags_and_foreign_symbols_keep_the_local_spelling() {
    let fixture = Fixture::new();
    fixture.write(
        "far.bend",
        r#"
import Base
type Shape is Data:
  Local.Tag{value: Nat}
def make() -> IO(Shape):
  import "far.js"
"#,
    );
    let checked = fixture.checked(
        r"
import Base
import far.bend as F
def main() -> IO(Nat):
  do IO<Nat>:
    shape : F.Shape <- F.make()
    return 0n
",
    );
    let program = checked.lower_for_javascript().unwrap();
    assert_eq!(program.constructor_tags["far.Local.Tag"], "Local.Tag");
    let DefinitionBody::Foreign(metadata) = &program.definitions["far.make"].body else {
        panic!("expected imported foreign body")
    };
    assert_eq!(metadata.local_symbol, "make");
    assert_eq!(metadata.declared_arity, 0);
}

#[test]
fn recursive_templates_callbacks_and_parallel_lets_keep_bound_identifiers() {
    let fixture = Fixture::new();
    let checked = fixture.checked(
        r"
import Base
def twice(~f: Nat -> Nat, +x: Nat) -> Nat: Nat.add(f(x), f(x))
def identity(n: Nat) -> Nat: n
def callback() -> (@-tag: Nat -> @x: Nat -> Nat): tag => x => x
def main() -> Nat:
  +a +b = identity(1n) identity(2n)
  Nat.add(twice(~(n => Nat.add(n, n)), a), callback()(0n, b))
",
    );
    let first = checked.lower_for_javascript().unwrap();
    let second = checked.lower_for_javascript().unwrap();
    assert_eq!(format!("{first:?}"), format!("{second:?}"));
    for definition in first.definitions.values() {
        if let DefinitionBody::Ordinary(body) = &definition.body {
            assert_scope(body, &BTreeSet::new());
        }
    }
    assert!(
        first
            .definitions
            .keys()
            .any(|name| name.starts_with("twice~"))
    );
    assert!(first.definitions.contains_key("Nat.add"));
    let main = ordinary(&first, "main");
    assert!(matches!(&main.kind, ExpressionKind::Let { bindings, .. }
        if bindings.len() == 2 && bindings.iter().all(|binding| binding.binder.quant == Quant::Lone)));
    assert!(
        expressions(main)
            .iter()
            .any(|expression| matches!(&expression.kind,
        ExpressionKind::Let { bindings, .. }
        if bindings.iter().any(|binding| binding.binder.quant == Quant::Many)))
    );
    let callback = ordinary(&first, "callback");
    assert!(
        matches!(&callback.kind, ExpressionKind::Lambda { parameter, .. }
        if parameter.quant == Quant::None)
    );
    assert!(matches!(
        callback.ty.as_ref(),
        Term::All {
            quant: Quant::None,
            ..
        }
    ));
}

#[test]
fn programs_without_main_do_not_lower_unused_definitions() {
    let program = lowered("import Base\ndef unused() -> Nat: 0n\n");
    assert_eq!(program.entry, ExecutableEntry::Missing);
    assert!(program.definitions.is_empty());
}

#[test]
fn equality_transport_keeps_live_body_and_omits_proof_computation() {
    let program = lowered(
        r"
import Base
def cast(-A: Type, -B: Type, proof: {A == B : Type}, x: A) -> B:
  %proof: _; x
def main() -> Nat: cast(Nat, Nat, {==}, 2n)
",
    );
    let mut body = ordinary(&program, "cast");
    let mut parameters = Vec::new();
    while let ExpressionKind::Lambda {
        parameter,
        body: inner,
    } = &body.kind
    {
        parameters.push(parameter);
        body = inner;
    }
    assert_eq!(parameters.len(), 4);
    assert!(matches!(body.kind, ExpressionKind::Variable(id) if id == parameters[3].id));
    assert!(matches!(body.ty.as_ref(), Term::Var { id, .. } if *id == parameters[1].id));
    assert!(matches!(parameters[2].ty.as_ref(), Term::Eql { .. }));
}

#[test]
fn dependent_constructor_telescope_substitutes_earlier_erased_values() {
    let program = lowered(
        r"
import Base
type Pack is Type:
  Pack{-A: Type, value: A}
def main() -> Pack: Pack{Nat, 3n}
",
    );
    let ExpressionKind::Constructor { fields, .. } = &ordinary(&program, "main").kind else {
        panic!("expected dependent package")
    };
    assert_eq!(fields[0].binder.quant, Quant::None);
    assert!(matches!(fields[0].value.kind, ExpressionKind::Erased));
    assert_eq!(fields[1].binder.ty.to_string(), "Nat");
    assert!(matches!(&fields[1].value.kind,
        ExpressionKind::Constructor { owner, .. } if owner == "Nat"));
}

#[test]
fn sealed_numeric_contracts_are_distinct_from_io_foreign_bodies() {
    let program = lowered(
        r"
import Base
def main() -> Bool: F32.is_lt(F32.add(U32.to_f32(1), 2.0), 4.0)
",
    );
    for (name, expected, arity) in [
        ("F32.is_lt", NumericIntrinsic::IsLt, 2),
        ("F32.add", NumericIntrinsic::Add, 2),
        ("U32.to_f32", NumericIntrinsic::U32ToF32, 1),
    ] {
        let definition = &program.definitions[name];
        assert!(
            matches!(definition.body, DefinitionBody::Numeric(intrinsic) if intrinsic == expected)
        );
        assert_eq!(definition.parameters.len(), arity);
        assert!(
            definition
                .parameters
                .iter()
                .all(|parameter| parameter.quant == Quant::Lone)
        );
    }
    assert!(!program.definitions.contains_key("F32.div"));
}

#[test]
fn ordinary_arrays_templates_and_u32_text_helpers_lower_transitively() {
    let program = lowered(
        r"
import Base
def render(result: Array<U32> & U32) -> IO(Unit):
  (array, value) = result
  IO.print(U32.show(value))
def inc(n: U32) -> U32: (n + 1 : U32)
def main() -> IO(Unit):
  render(Array.get(U32, Array.map(~U32, ~U32, ~inc, [0: U32 * 4n]), 2))
",
    );
    assert!(program.definitions.contains_key("Array.get"));
    assert!(program.definitions.contains_key("U32.show"));
    assert!(
        program
            .definitions
            .keys()
            .any(|name| name.starts_with("Array.map~"))
    );
    assert!(program.definitions.values().any(|definition| {
        let DefinitionBody::Ordinary(body) = &definition.body else {
            return false;
        };
        expressions(body).iter().any(|expression| {
            matches!(&expression.kind,
            ExpressionKind::Match { owner, .. } if owner == "Array")
        })
    }));
}
