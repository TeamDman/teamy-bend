// SPDX-License-Identifier: Apache-2.0
// Layout rules derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
//! Native words, finite flattened layouts and the foreign constructor table.

use super::CompileError;
use super::executable_foreign::reference_order;
use crate::kernel::AdtDecl;
use crate::kernel::Quant;
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::kernel::elaborate::DefinitionBody;
use crate::kernel::elaborate::ExecutableProgram;
use crate::kernel::elaborate::Expression;
use crate::kernel::elaborate::ExpressionKind;
use crate::kernel::substitute;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::fmt::Write;
use std::rc::Rc;

const RUNTIME_ADTS: [&str; 9] = [
    "Sigma", "String", "Word.Con", "IO.OP", "Result", "Maybe", "Bool", "Unit",
    // The interpreter's native Word conversions construct the empty tail too.
    "Word.Nil",
];
const MAX_DEPTH: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Kind {
    W32,
    W64,
    Box,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Layout {
    pub(super) words: Vec<Kind>,
    pub(super) arms: Option<Vec<Arm>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Arm {
    pub(super) name: String,
    pub(super) fields: Vec<Field>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Field {
    pub(super) offset: usize,
    pub(super) layout: Layout,
}

impl Layout {
    fn word(kind: Kind) -> Self {
        Self {
            words: vec![kind],
            arms: None,
        }
    }

    /// A finite sum has an arm index, not a constructor ID, in its first word.
    fn pack(mut arms: Vec<Arm>) -> Self {
        let tag = usize::from(arms.len() > 1);
        let mut words = vec![Kind::W32; tag];
        for arm in &mut arms {
            for field in &mut arm.fields {
                field.offset += tag;
                for (index, kind) in field.layout.words.iter().enumerate() {
                    let index = field.offset + index;
                    words.resize(words.len().max(index + 1), Kind::W32);
                    words[index] = match (words[index], kind) {
                        (Kind::Box, _) | (_, Kind::Box) => Kind::Box,
                        (Kind::W64, _) | (_, Kind::W64) => Kind::W64,
                        _ => Kind::W32,
                    };
                }
            }
        }
        Self {
            words,
            arms: Some(arms),
        }
    }
}

pub(super) struct Layouts<'a> {
    program: &'a ExecutableProgram,
    cycles: BTreeMap<String, bool>,
    nodes: BTreeMap<String, Layout>,
}

impl<'a> Layouts<'a> {
    pub(super) fn new(program: &'a ExecutableProgram) -> Self {
        Self {
            program,
            cycles: BTreeMap::new(),
            nodes: BTreeMap::new(),
        }
    }

    pub(super) fn layout(&mut self, ty: &TermRef) -> Result<Layout, CompileError> {
        self.layout_at(ty, 0)
    }

    fn layout_at(&mut self, ty: &TermRef, depth: usize) -> Result<Layout, CompileError> {
        check_depth(depth)?;
        let ty = self
            .program
            .expose_type(ty)
            .map_err(|error| CompileError::new(error.to_string()))?;
        let Term::Adt { name, args, .. } = ty.as_ref() else {
            return Ok(Layout::word(Kind::Box));
        };
        if let Some(word) = native_word(self.program, name) {
            return Ok(Layout::word(word));
        }
        if (self.program.base_names.contains(name) && (name == "Array" || name == "IO.OP"))
            || self.cyclic(name)?
        {
            return Ok(Layout::word(Kind::Box));
        }
        let Some(datatype) = self.program.datatypes.get(name) else {
            return Ok(Layout::word(Kind::Box));
        };
        if args.len() != datatype.parameters.len() {
            return Err(CompileError::new(format!(
                "C layout: datatype parameter count differs for {name}"
            )));
        }
        let mut arms = Vec::new();
        for constructor in &datatype.constructors {
            let types = constructor
                .fields
                .iter()
                .filter(|field| field.quant != Quant::None)
                .map(|field| {
                    let mut ty = Rc::clone(&field.ty);
                    for (parameter, argument) in datatype.parameters.iter().zip(args) {
                        ty = substitute(&ty, parameter.id, argument);
                    }
                    ty
                })
                .collect::<Vec<_>>();
            arms.push(Arm {
                name: constructor.name.clone(),
                fields: self.fields(&types, depth + 1)?,
            });
        }
        Ok(Layout::pack(arms))
    }

    fn fields(&mut self, types: &[TermRef], depth: usize) -> Result<Vec<Field>, CompileError> {
        let mut offset = 0;
        let mut fields = Vec::new();
        for ty in types {
            let layout = self.layout_at(ty, depth)?;
            fields.push(Field { offset, layout });
            offset += fields.last().expect("just inserted").layout.words.len();
            // Heap node arities and foreign parameter counts are u8 upstream.
            if offset > 255 {
                return Err(CompileError::new("C layout: an arity over 255"));
            }
        }
        Ok(fields)
    }

    /// Heap nodes use the declaration with open generic parameters. For example,
    /// Tuple stores two boxes even when a particular Pair has two word fields.
    pub(super) fn node(&mut self, name: &str) -> Result<Layout, CompileError> {
        if let Some(layout) = self.nodes.get(name) {
            return Ok(layout.clone());
        }
        let constructor = self
            .program
            .datatypes
            .values()
            .flat_map(|datatype| &datatype.constructors)
            .find(|constructor| constructor.name == name)
            .ok_or_else(|| CompileError::new(format!("C layout: unknown constructor {name}")))?;
        let types = constructor
            .fields
            .iter()
            .filter(|field| field.quant != Quant::None)
            .map(|field| Rc::clone(&field.ty))
            .collect::<Vec<_>>();
        let layout = Layout::pack(vec![Arm {
            name: name.to_owned(),
            fields: self.fields(&types, 0)?,
        }]);
        self.nodes.insert(name.to_owned(), layout.clone());
        Ok(layout)
    }

    fn cyclic(&mut self, name: &str) -> Result<bool, CompileError> {
        if let Some(cyclic) = self.cycles.get(name) {
            return Ok(*cyclic);
        }
        let cyclic = self.walk_cycle(name, name, &mut BTreeSet::new(), 0)?;
        self.cycles.insert(name.to_owned(), cyclic);
        Ok(cyclic)
    }

    fn walk_cycle(
        &self,
        root: &str,
        name: &str,
        seen: &mut BTreeSet<String>,
        depth: usize,
    ) -> Result<bool, CompileError> {
        check_depth(depth)?;
        if native_word(self.program, name).is_some()
            || (self.program.base_names.contains(name) && name == "Array")
        {
            return Ok(false);
        }
        let Some(datatype) = self.program.datatypes.get(name) else {
            return Ok(false);
        };
        for field in datatype
            .constructors
            .iter()
            .flat_map(|constructor| &constructor.fields)
            .filter(|field| field.quant != Quant::None)
        {
            if self.hits_cycle(root, &field.ty, seen, depth + 1)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn hits_cycle(
        &self,
        root: &str,
        ty: &TermRef,
        seen: &mut BTreeSet<String>,
        depth: usize,
    ) -> Result<bool, CompileError> {
        check_depth(depth)?;
        let ty = self
            .program
            .expose_type(ty)
            .map_err(|error| CompileError::new(error.to_string()))?;
        let Term::Adt { name, args, .. } = ty.as_ref() else {
            return Ok(false);
        };
        if name == root {
            return Ok(true);
        }
        for argument in args {
            if self.hits_cycle(root, argument, seen, depth + 1)? {
                return Ok(true);
            }
        }
        Ok(seen.insert(name.clone()) && self.walk_cycle(root, name, seen, depth + 1)?)
    }
}

fn native_word(program: &ExecutableProgram, name: &str) -> Option<Kind> {
    if !program.base_names.contains(name) {
        return None;
    }
    match name {
        "U32" | "F32" => Some(Kind::W32),
        "Nat" => Some(Kind::W64),
        _ => None,
    }
}

fn check_depth(depth: usize) -> Result<(), CompileError> {
    if depth > MAX_DEPTH {
        Err(CompileError::new("C layout depth budget exhausted"))
    } else {
        Ok(())
    }
}

pub(super) fn cid_macro(name: &str) -> String {
    // Upstream's JavaScript regexp has no Unicode flag: supplementary scalar
    // values contain two UTF-16 code units and therefore become two underscores.
    let clean = name
        .encode_utf16()
        .map(|unit| {
            let c = char::from_u32(u32::from(unit)).unwrap_or('_');
            if c.is_ascii_alphanumeric() || c == '_' {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!("CID_{clean}")
}

#[derive(Clone, Debug)]
pub(super) struct Constructor {
    pub(super) cid: u16,
    pub(super) macro_name: String,
    pub(super) arity: u8,
    pub(super) layout: Layout,
}

pub(super) struct ConstructorTable {
    pub(super) entries: Vec<Constructor>,
    indices: BTreeMap<String, usize>,
}

impl ConstructorTable {
    pub(super) fn build(program: &ExecutableProgram) -> Result<Self, CompileError> {
        let mut table = Self {
            entries: Vec::new(),
            indices: BTreeMap::new(),
        };
        let mut layouts = Layouts::new(program);
        let mut queue = VecDeque::from(RUNTIME_ADTS.map(str::to_owned));
        // Discover datatypes in live definition and expression types. Runtime
        // ADTs are roots even if a source program never constructs them itself.
        for name in reference_order(program) {
            let definition = &program.definitions[name];
            collect_types(program, &definition.ty, &mut queue, 0)?;
            if let DefinitionBody::Ordinary(body) = &definition.body {
                expression_types(program, body, &mut queue)?;
            }
        }
        let mut seen = BTreeSet::new();
        while let Some(name) = queue.pop_front() {
            if !seen.insert(name.clone()) {
                continue;
            }
            let Some(datatype) = program.datatypes.get(&name) else {
                continue;
            };
            for constructor in &datatype.constructors {
                table.push(
                    &constructor.name,
                    cid_macro(&constructor.name),
                    layouts.node(&constructor.name)?,
                )?;
            }
            datatype_types(program, datatype, &mut queue)?;
        }
        for name in reference_order(program) {
            let definition = &program.definitions[name];
            if let DefinitionBody::Foreign(foreign) = &definition.body {
                let count = definition
                    .parameters
                    .iter()
                    .filter(|p| p.quant != Quant::None)
                    .count()
                    + 1;
                let fields = (0..count)
                    .map(|offset| Field {
                        offset,
                        layout: Layout::word(Kind::Box),
                    })
                    .collect();
                let layout = Layout::pack(vec![Arm {
                    name: name.to_owned(),
                    fields,
                }]);
                table.push(name, cid_macro(&foreign.local_symbol), layout)?;
            }
        }
        Ok(table)
    }

    fn push(&mut self, name: &str, macro_name: String, layout: Layout) -> Result<(), CompileError> {
        if self
            .entries
            .iter()
            .any(|entry| entry.macro_name == macro_name)
            || ["CID_ARITY_T", "FID_ARITY_T", "FID_FLAG_T", "FID_RESW_T"]
                .contains(&macro_name.as_str())
        {
            return Err(CompileError::new(format!(
                "C layout: two names mangle to {macro_name}"
            )));
        }
        let cid = u16::try_from(self.entries.len())
            .map_err(|_error| CompileError::new("C layout: an id over 65535"))?;
        let arity = u8::try_from(layout.words.len())
            .map_err(|_error| CompileError::new("C layout: an arity over 255"))?;
        self.indices.insert(name.to_owned(), self.entries.len());
        self.entries.push(Constructor {
            cid,
            macro_name,
            arity,
            layout,
        });
        Ok(())
    }

    pub(super) fn get(&self, name: &str) -> Result<&Constructor, CompileError> {
        self.indices
            .get(name)
            .map(|index| &self.entries[*index])
            .ok_or_else(|| CompileError::new(format!("C layout: unregistered constructor {name}")))
    }

    pub(super) fn declarations(&self) -> String {
        let mut source = String::new();
        for entry in &self.entries {
            writeln!(source, "#define {} {}", entry.macro_name, entry.cid).unwrap();
        }
        writeln!(source, "#define BEND_CID_COUNT {}", self.entries.len()).unwrap();
        let arities = self
            .entries
            .iter()
            .map(|entry| entry.arity.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(
            source,
            "static const uint8_t CID_ARITY_T[] = {{ {} }};",
            if arities.is_empty() { "0" } else { &arities }
        )
        .unwrap();
        source
    }
}

fn datatype_types(
    program: &ExecutableProgram,
    datatype: &AdtDecl,
    queue: &mut VecDeque<String>,
) -> Result<(), CompileError> {
    for field in datatype
        .constructors
        .iter()
        .flat_map(|constructor| &constructor.fields)
    {
        collect_types(program, &field.ty, queue, 0)?;
    }
    Ok(())
}

fn collect_types(
    program: &ExecutableProgram,
    ty: &TermRef,
    queue: &mut VecDeque<String>,
    depth: usize,
) -> Result<(), CompileError> {
    check_depth(depth)?;
    let ty = program
        .expose_type(ty)
        .map_err(|error| CompileError::new(error.to_string()))?;
    match ty.as_ref() {
        Term::Adt { name, args, .. } => {
            // Native Array needs no ordinary constructor row, but its element
            // type can contain constructors required by conversions/printers.
            if native_word(program, name).is_none()
                && !(program.base_names.contains(name) && name == "Array")
            {
                queue.push_back(name.clone());
            }
            for argument in args {
                collect_types(program, argument, queue, depth + 1)?;
            }
        }
        Term::All { domain, body, .. } => {
            collect_types(program, domain, queue, depth + 1)?;
            collect_types(program, body, queue, depth + 1)?;
        }
        Term::Lam { body, .. } => collect_types(program, body, queue, depth + 1)?,
        _ => {}
    }
    Ok(())
}

fn expression_types(
    program: &ExecutableProgram,
    expression: &Expression,
    queue: &mut VecDeque<String>,
) -> Result<(), CompileError> {
    let mut pending = vec![expression];
    while let Some(expression) = pending.pop() {
        collect_types(program, &expression.ty, queue, 0)?;
        match &expression.kind {
            ExpressionKind::Lambda { body, .. } => pending.push(body),
            ExpressionKind::Apply {
                function, argument, ..
            } => {
                pending.push(argument);
                pending.push(function);
            }
            ExpressionKind::Constructor { owner, fields, .. } => {
                if native_word(program, owner).is_none()
                    && !(program.base_names.contains(owner) && owner == "Array")
                {
                    queue.push_back(owner.clone());
                }
                pending.extend(fields.iter().rev().map(|field| &field.value));
            }
            ExpressionKind::Match {
                owner,
                arm,
                fallback,
                ..
            } => {
                if native_word(program, owner).is_none()
                    && !(program.base_names.contains(owner) && owner == "Array")
                {
                    queue.push_back(owner.clone());
                }
                pending.push(fallback);
                pending.push(arm);
            }
            ExpressionKind::Let { bindings, body } => {
                pending.push(body);
                pending.extend(bindings.iter().rev().map(|field| &field.value));
            }
            ExpressionKind::Erased => {
                // A concrete type can occur only as a generic call's erased
                // argument. Its expression type is Data/Type, so inspecting
                // that type alone misses constructors needed by the instance.
                let ty = program
                    .expose_type(&expression.ty)
                    .map_err(|error| CompileError::new(error.to_string()))?;
                if matches!(ty.as_ref(), Term::Typ(_)) {
                    collect_types(program, &expression.source, queue, 0)?;
                }
            }
            ExpressionKind::Variable(_)
            | ExpressionKind::Definition(_)
            | ExpressionKind::Absurd { .. } => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::check_executable;
    use crate::syntax::load_executable;
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn program(source: &str) -> ExecutableProgram {
        let path = std::env::temp_dir().join(format!(
            "teamy-bend-c-layout-{}-{}.bend",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, source).unwrap();
        let loaded = load_executable(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        check_executable(&loaded)
            .unwrap()
            .lower_for_compilation()
            .unwrap()
    }

    #[test]
    fn native_node_layouts_match_open_generic_upstream_fields() {
        let program = program("import Base\ndef main() -> U32 & Nat:\n  (3, 4n)\n");
        let mut layouts = Layouts::new(&program);
        assert_eq!(layouts.node("SCon").unwrap().words, [Kind::W32, Kind::Box]);
        assert_eq!(layouts.node("Tuple").unwrap().words, [Kind::Box, Kind::Box]);
        assert_eq!(layouts.node("WCon").unwrap().words, [Kind::W32, Kind::Box]);
        assert_eq!(layouts.node("Chr").unwrap().words, [Kind::W32]);
        let concrete = layouts.layout(&program.definitions["main"].ty).unwrap();
        assert_eq!(concrete.words, [Kind::W32, Kind::W64]);
        let arm = &concrete.arms.unwrap()[0];
        assert_eq!(arm.fields[1].offset, 1);
        let table = ConstructorTable::build(&program).unwrap();
        assert_eq!(table.get("SCon").unwrap().arity, 2);
        assert_eq!(table.get("Unit").unwrap().arity, 0);
        assert!(table.declarations().contains("CID_ARITY_T"));
    }

    #[test]
    fn finite_sum_merges_widths_and_keeps_arm_field_offsets() {
        let program = program(
            "import Base\ntype Choice is Data:\n  Narrow{x: U32}\n  Wide{x: Nat, y: String}\ndef main() -> Choice:\n  Narrow{1}\n",
        );
        let layout = Layouts::new(&program)
            .layout(&program.definitions["main"].ty)
            .unwrap();
        assert_eq!(layout.words, [Kind::W32, Kind::W64, Kind::Box]);
        let arms = layout.arms.unwrap();
        assert_eq!(arms[0].fields[0].offset, 1);
        assert_eq!(arms[1].fields[1].offset, 2);
    }

    #[test]
    fn native_spelling_without_base_origin_is_an_ordinary_datatype() {
        let program = program("type U32 is Data:\n  A{}\n  B{}\ndef main() -> U32:\n  A{}\n");
        let layout = Layouts::new(&program)
            .layout(&program.definitions["main"].ty)
            .unwrap();
        assert_eq!(layout.arms.unwrap().len(), 2);
        ConstructorTable::build(&program).unwrap().get("A").unwrap();
    }

    #[test]
    fn constructor_macros_and_arities_refuse_ambiguous_or_truncated_tables() {
        assert_eq!(cid_macro("A𝑨"), "CID_A__");
        let program =
            program("type Choice is Data:\n  X.y{}\n  X_y{}\ndef main() -> Choice:\n  X.y{}\n");
        assert!(
            ConstructorTable::build(&program)
                .err()
                .unwrap()
                .to_string()
                .contains("mangle")
        );
        let mut table = ConstructorTable {
            entries: Vec::new(),
            indices: BTreeMap::new(),
        };
        assert!(
            table
                .push(
                    "Huge",
                    "CID_HUGE".into(),
                    Layout {
                        words: vec![Kind::Box; 256],
                        arms: None
                    }
                )
                .is_err()
        );
    }
}
