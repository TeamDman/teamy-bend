// SPDX-License-Identifier: MPL-2.0
//! Compact executable words with lossless, lazy constructor views. This module
//! never normalizes a source field or participates in proof checking.

use super::Machine;
use super::Thunk;
use super::ThunkId;
use super::Value;
use crate::kernel::AdtDecl;
use crate::kernel::KernelError;
use crate::kernel::Quant;
use crate::kernel::Term;
use crate::kernel::TermRef;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Wrapper {
    U32,
    F32,
}

impl Wrapper {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::U32 => "U32",
            Self::F32 => "F32",
        }
    }

    pub(super) fn from_name(name: &str) -> Option<Self> {
        match name {
            "U32" => Some(Self::U32),
            "F32" => Some(Self::F32),
            _ => None,
        }
    }
}

fn reference(value: &TermRef, expected: &str) -> bool {
    matches!(value.as_ref(), Term::Ref(name) | Term::GpuRef(name) if name == expected)
}

fn data_kind(datatype: &AdtDecl) -> bool {
    matches!(datatype.kind.as_ref(), Term::Typ(quantity)
        if matches!(quantity.as_ref(), Term::Qua(Quant::Many)))
}

fn constructors(datatype: &AdtDecl, shapes: &[(&str, usize)]) -> bool {
    datatype.constructors.len() == shapes.len()
        && datatype
            .constructors
            .iter()
            .zip(shapes)
            .all(|(ctor, (name, fields))| {
                ctor.name == *name
                    && ctor.fields.len() == *fields
                    && ctor.fields.iter().all(|field| field.quant == Quant::Lone)
            })
}

fn nat_32(mut value: &TermRef) -> bool {
    for _ in 0..32 {
        let Term::Ctr { name, args } = value.as_ref() else {
            return false;
        };
        if name != "Succ" || args.len() != 1 {
            return false;
        }
        value = &args[0];
    }
    matches!(value.as_ref(), Term::Ctr {name, args} if name == "Zero" && args.is_empty())
}

/// All names must be loader-sealed Base origins, with the exact checked native
/// constructor layouts. A same-named user type alone never enables packing.
pub(super) fn checked_layouts(
    datatypes: &BTreeMap<String, AdtDecl>,
    base_names: &BTreeSet<String>,
) -> bool {
    const NAMES: [&str; 6] = ["Bool", "Nat", "Word.Nil", "Word.Con", "U32", "F32"];
    if !base_names.contains("Word") || NAMES.iter().any(|name| !base_names.contains(*name)) {
        return false;
    }
    let layouts = NAMES.map(|name| datatypes.get(name));
    let [
        Some(boolean),
        Some(nat),
        Some(nil),
        Some(cons),
        Some(u32_),
        Some(f32_),
    ] = layouts
    else {
        return false;
    };
    if layouts.iter().zip(NAMES).any(|(layout, name)| {
        let layout = layout.expect("all layouts present");
        layout.name != name || !data_kind(layout)
    }) {
        return false;
    }
    if !boolean.parameters.is_empty()
        || !constructors(boolean, &[("False", 0), ("True", 0)])
        || !nat.parameters.is_empty()
        || !constructors(nat, &[("Zero", 0), ("Succ", 1)])
        || !reference(&nat.constructors[1].fields[0].ty, "Nat")
        || !nil.parameters.is_empty()
        || !constructors(nil, &[("WNil", 0)])
        || cons.parameters.len() != 1
        || !constructors(cons, &[("WCon", 2)])
    {
        return false;
    }
    let parameter = &cons.parameters[0];
    let fields = &cons.constructors[0].fields;
    if parameter.quant != Quant::None
        || !reference(&parameter.ty, "Nat")
        || !reference(&fields[0].ty, "Bool")
        || !matches!(fields[1].ty.as_ref(), Term::App(function, argument)
            if reference(function, "Word")
                && matches!(argument.as_ref(), Term::Var{id, ..} if *id == parameter.id))
    {
        return false;
    }
    for (layout, wrapper) in [(u32_, "U32"), (f32_, "F32")] {
        if !layout.parameters.is_empty()
            || !constructors(layout, &[(wrapper, 1)])
            || !matches!(layout.constructors[0].fields[0].ty.as_ref(), Term::App(function, argument)
                if reference(function, "Word") && nat_32(argument))
        {
            return false;
        }
    }
    true
}

/// Recognize only a complete literal tree: no references, annotations, variables,
/// calls, or field evaluation. Traversal is bounded by the word's fixed width.
pub(super) fn literal(name: &str, fields: &[TermRef]) -> Option<Value> {
    let wrapper = Wrapper::from_name(name)?;
    let [word] = fields else { return None };
    let mut word = word;
    let mut bits = 0;
    for bit in 0..32 {
        let Term::Ctr { name, args } = word.as_ref() else {
            return None;
        };
        let [head, tail] = args.as_slice() else {
            return None;
        };
        if name != "WCon" {
            return None;
        }
        let Term::Ctr { name, args } = head.as_ref() else {
            return None;
        };
        if !args.is_empty() {
            return None;
        }
        match name.as_str() {
            "True" => bits |= 1 << bit,
            "False" => {}
            _ => return None,
        }
        word = tail;
    }
    if !matches!(word.as_ref(), Term::Ctr{name, args} if name == "WNil" && args.is_empty()) {
        return None;
    }
    Some(Value::PackedWord { wrapper, bits })
}

pub(super) const fn valid_bits(bits: u32, width: u8) -> bool {
    width <= 32 && (width == 32 || bits >> width == 0)
}

impl Machine<'_> {
    /// Reveal one constructor layer. Packed tails remain compact until demanded;
    /// a fallback must retain the caller's original thunk, not this temporary view.
    pub(super) fn constructor_value(
        &mut self,
        value: Value,
    ) -> Result<(String, Vec<ThunkId>), KernelError> {
        match value {
            Value::Constructor { name, fields } => Ok((name, fields)),
            Value::PackedNat(value) => {
                self.tick()?;
                if value == 0 {
                    Ok(("Zero".into(), vec![]))
                } else {
                    let tail = self.nat(value - 1)?;
                    Ok(("Succ".into(), vec![tail]))
                }
            }
            Value::PackedWord { wrapper, bits } => {
                self.tick()?;
                let word = self.allocate(Thunk::Ready(Value::PackedBits { bits, width: 32 }))?;
                Ok((wrapper.name().into(), vec![word]))
            }
            Value::PackedBits { bits, width } => {
                self.tick()?;
                if !valid_bits(bits, width) {
                    return Err(KernelError::new("invalid packed Word width or bits"));
                }
                if width == 0 {
                    return Ok(("WNil".into(), vec![]));
                }
                let head = self.allocate(Thunk::Ready(Value::Constructor {
                    name: if bits & 1 == 0 { "False" } else { "True" }.into(),
                    fields: vec![],
                }))?;
                let tail = self.allocate(Thunk::Ready(Value::PackedBits {
                    bits: bits >> 1,
                    width: width - 1,
                }))?;
                Ok(("WCon".into(), vec![head, tail]))
            }
            Value::Request { .. } => Err(KernelError::new(
                "runtime fail-stop: a foreign effect request cannot be inspected as constructor data",
            )),
            _ => Err(KernelError::new("runtime match expected constructor data")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::Declaration;
    use crate::kernel::check_book;
    use crate::kernel::term;
    use crate::runtime::Program;
    use crate::syntax::parse;
    use crate::syntax::parse_term;
    use std::rc::Rc;

    fn program() -> Program {
        let source = parse(include_str!("../syntax/base.bend")).unwrap();
        check_book(&source).unwrap();
        let mut definitions = BTreeMap::new();
        let mut datatypes = BTreeMap::new();
        let mut origins = BTreeSet::new();
        for declaration in source.declarations {
            match declaration {
                Declaration::Def(definition) => {
                    origins.insert(definition.name.clone());
                    definitions.insert(definition.name.clone(), definition);
                }
                Declaration::Adt(datatype) => {
                    origins.insert(datatype.name.clone());
                    datatypes.insert(datatype.name.clone(), datatype);
                }
            }
        }
        Program::from_executable(
            &Rc::new(definitions),
            &Rc::new(datatypes),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &origins,
        )
    }

    fn raw(wrapper: &str, bits: u32) -> TermRef {
        let literal = parse_term(&bits.to_string()).unwrap();
        let Term::Ctr { args, .. } = literal.as_ref() else {
            panic!("U32 literal")
        };
        term(Term::Ctr {
            name: wrapper.into(),
            args: args.clone(),
        })
    }

    #[test]
    fn packing_requires_complete_sealed_exact_base_layouts() {
        let program = program();
        assert!(program.packed);
        let names = ["Bool", "Nat", "Word", "Word.Nil", "Word.Con", "U32", "F32"];
        let origins = names
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<BTreeSet<_>>();
        assert!(checked_layouts(&program.datatypes, &origins));
        assert!(!checked_layouts(&program.datatypes, &BTreeSet::new()));
        for name in names {
            let mut missing = origins.clone();
            missing.remove(name);
            assert!(!checked_layouts(&program.datatypes, &missing), "{name}");
        }
        for mutation in 0..11 {
            let mut datatypes = program.datatypes.as_ref().clone();
            match mutation {
                0 => {
                    datatypes.remove("Word.Con");
                }
                1 => datatypes.get_mut("Bool").unwrap().constructors[0].name = "Fake".into(),
                2 => {
                    datatypes.get_mut("Nat").unwrap().kind =
                        term(Term::Typ(term(Term::Qua(Quant::Lone))));
                }
                3 => {
                    datatypes.get_mut("Nat").unwrap().constructors[1].fields[0].ty =
                        term(Term::Ref("Bool".into()));
                }
                4 => datatypes.get_mut("Word.Con").unwrap().parameters[0].quant = Quant::Lone,
                5 => {
                    datatypes.get_mut("Word.Con").unwrap().constructors[0].fields[0].quant =
                        Quant::None;
                }
                6 => {
                    datatypes.get_mut("Word.Con").unwrap().constructors[0].fields[0].ty =
                        term(Term::Ref("Nat".into()));
                }
                7 => datatypes.get_mut("Word.Con").unwrap().parameters[0].id += 1,
                8 => {
                    datatypes.get_mut("U32").unwrap().constructors[0].fields[0].ty =
                        term(Term::Ref("Word.Nil".into()));
                }
                9 => {
                    datatypes.get_mut("F32").unwrap().constructors[0]
                        .fields
                        .clear();
                }
                10 => datatypes.get_mut("Word.Nil").unwrap().name = "other.Word.Nil".into(),
                _ => unreachable!(),
            }
            assert!(
                !checked_layouts(&datatypes, &origins),
                "mutation {mutation}"
            );
        }
    }

    #[test]
    fn strict_programs_keep_constructor_trees_and_executables_pack_literal_bits() {
        let executable = program();
        let strict = Program::from_checked(&executable.definitions, &executable.datatypes);
        assert!(!strict.packed);
        for bits in [0, 1, u32::MAX, 0x8000_0000, 0x7fc0_0001, 0x7f80_0001] {
            for wrapper in [Wrapper::U32, Wrapper::F32] {
                let source = raw(wrapper.name(), bits);
                let mut machine = Machine::new(&strict);
                let id = machine.expression(Rc::clone(&source), 0).unwrap();
                assert!(matches!(
                    machine.force(id).unwrap(),
                    Value::Constructor { .. }
                ));
                let mut machine = Machine::new(&executable);
                let id = machine.expression(source, 0).unwrap();
                assert!(
                    matches!(machine.force(id).unwrap(), Value::PackedWord{wrapper: actual, bits: value} if actual == wrapper && value == bits)
                );
                assert_eq!(machine.arena.len(), 1, "a literal must remain one thunk");
            }
        }
    }

    #[test]
    fn literal_recognition_never_unfolds_or_demands_nonliteral_fields() {
        let executable = program();
        for field in [
            term(Term::Ref("unevaluated_word".into())),
            term(Term::Ctr {
                name: "WCon".into(),
                args: vec![
                    term(Term::Ref("unevaluated_bit".into())),
                    term(Term::Ctr {
                        name: "WNil".into(),
                        args: vec![],
                    }),
                ],
            }),
        ] {
            assert!(literal("U32", &[Rc::clone(&field)]).is_none());
            let mut machine = Machine::new(&executable);
            let value = term(Term::Ctr {
                name: "U32".into(),
                args: vec![field],
            });
            let id = machine.expression(value, 0).unwrap();
            assert!(matches!(
                machine.force(id).unwrap(),
                Value::Constructor { .. }
            ));
            assert_eq!(machine.arena.len(), 2);
        }
        let literal_value = raw("other.U32", 7);
        let Term::Ctr { name, args } = literal_value.as_ref() else {
            unreachable!()
        };
        assert!(literal(name, args).is_none());
    }

    #[test]
    fn one_layer_views_and_materialization_preserve_the_original_tree() {
        let program = program();
        for bits in [0, 1, u32::MAX, 0x8000_0000, 0x7fc0_0042, 0x7f80_0001] {
            for wrapper in [Wrapper::U32, Wrapper::F32] {
                let mut machine = Machine::new(&program);
                let (name, fields) = machine
                    .constructor_value(Value::PackedWord { wrapper, bits })
                    .unwrap();
                assert_eq!(name, wrapper.name());
                assert_eq!(machine.arena.len(), 1);
                let word = machine.force(fields[0]).unwrap();
                assert!(matches!(word, Value::PackedBits{bits: value, width:32} if value == bits));
                let (_, fields) = machine.constructor_value(word).unwrap();
                assert_eq!(
                    machine.arena.len(),
                    3,
                    "only the first bit and tail are exposed"
                );
                assert!(
                    matches!(machine.force(fields[1]).unwrap(), Value::PackedBits{bits: value, width:31} if value == bits >> 1)
                );
                let id = machine
                    .allocate(Thunk::Ready(Value::PackedWord { wrapper, bits }))
                    .unwrap();
                let result = machine.materialize(id, 0).unwrap();
                assert_eq!(result.to_string(), raw(wrapper.name(), bits).to_string());
                assert_eq!(
                    machine.output_nodes, 66,
                    "packed output still pays every constructor node"
                );
            }
        }
    }

    #[test]
    fn match_fallback_receives_the_original_packed_argument() {
        let program = program();
        let mut machine = Machine::new(&program);
        let argument = machine
            .allocate(Thunk::Ready(Value::PackedBits { bits: 1, width: 2 }))
            .unwrap();
        let arm = machine.allocate(Thunk::Ready(Value::Impossible)).unwrap();
        let fallback = machine
            .allocate(Thunk::Ready(Value::Closure {
                binder: 0,
                body: term(Term::Var {
                    name: "rest".into(),
                    id: 0,
                }),
                environment: 0,
            }))
            .unwrap();
        let function = machine
            .allocate(Thunk::Ready(Value::Match {
                constructor: "WNil".into(),
                arm,
                fallback,
            }))
            .unwrap();
        let call = machine
            .allocate(Thunk::Application(function, argument))
            .unwrap();
        assert!(matches!(
            machine.force(call).unwrap(),
            Value::PackedBits { bits: 1, width: 2 }
        ));
        assert!(matches!(
            machine.force(argument).unwrap(),
            Value::PackedBits { bits: 1, width: 2 }
        ));
    }

    #[test]
    fn malformed_packed_tails_requests_and_output_exhaustion_fail_closed() {
        let program = program();
        for (bits, width) in [(1, 0), (2, 1), (0, 33)] {
            let mut machine = Machine::new(&program);
            machine
                .constructor_value(Value::PackedBits { bits, width })
                .expect_err("invalid packed tails must fail closed");
        }
        let mut machine = Machine::new(&program);
        let error = machine
            .constructor_value(Value::Request {
                name: "effect".into(),
                arguments: vec![],
                continuation: 0,
            })
            .unwrap_err()
            .to_string();
        assert!(error.contains("foreign effect request"));
        let id = machine
            .allocate(Thunk::Ready(Value::PackedWord {
                wrapper: Wrapper::U32,
                bits: 7,
            }))
            .unwrap();
        machine.output_nodes = super::super::OUTPUT_NODES - 1;
        assert!(
            machine
                .materialize(id, 0)
                .unwrap_err()
                .to_string()
                .contains("output depth or node budget")
        );
        let mut machine = Machine::new(&program);
        machine.cancelled = Some(&|| true);
        assert!(
            machine
                .constructor_value(Value::PackedBits { bits: 0, width: 32 })
                .unwrap_err()
                .to_string()
                .contains("cancelled")
        );
    }
}
