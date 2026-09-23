// SPDX-License-Identifier: Apache-2.0
// Native layouts derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
//! Conversion between a boxed ABI Term and finite inline field layouts.

use super::Body;
use super::BodyDependencies;
use super::CompileError;
use super::DEVICE_ARRAY_NEW_STATE_WORDS;
use super::DEVICE_PAYLOAD_STATE_WORDS;
use super::ExecutableProgram;
use super::FunctionResult;
use super::Generator;
use super::Kind;
use super::Layout;
use super::Quant;
use super::Term;
use super::TermRef;
use super::c_string;
use super::substitute;
use std::fmt::Write;
use std::rc::Rc;

fn packed(layout: &Layout) -> bool {
    layout.words.is_empty() || layout.words == [Kind::W32]
}

impl Generator<'_> {
    pub(super) fn construct(
        &mut self,
        name: &str,
        values: &[String],
        output: &mut Body,
    ) -> Result<String, CompileError> {
        let constructor = self.table.get(name)?.clone();
        let fields = &constructor
            .layout
            .arms
            .as_ref()
            .ok_or_else(|| CompileError::new("missing C node layout"))?[0]
            .fields;
        if fields.len() != values.len() {
            return Err(CompileError::new(format!(
                "C constructor {name} field count differs from its layout"
            )));
        }
        let array = self.array(
            output,
            &vec!["0".to_owned(); constructor.layout.words.len()],
        )?;
        for (field, value) in fields.iter().zip(values) {
            let conversion = self.conversion_use(&field.layout, output)?;
            writeln!(
                output,
                "  tb_unbox_{conversion}(e, {value}, {array} + {});",
                field.offset
            )
            .unwrap();
        }
        let mask = self.ownership_mask(&constructor.layout, &array, output)?;
        let synchronous = format!(
            "tb_c_construct(e, {}, {}, {array}, {}, {mask})",
            constructor.cid,
            constructor.layout.words.len(),
            packed(&constructor.layout)
        );
        if !output.can_suspend || packed(&constructor.layout) {
            return self.hold(output, &synchronous);
        }

        let result = self.hold(output, "0")?;
        let state = self.array(output, &vec!["0".to_owned(); DEVICE_PAYLOAD_STATE_WORDS])?;
        output.resumes += 1;
        let pc = output.resumes;
        let yielded = if output.words {
            "tb_segment_yield()"
        } else {
            "0"
        };
        writeln!(
            output,
            "tb_resume_{pc}: ;\n#ifdef __CUDA_ARCH__\n  if (!tb_device_construct_raw(e, {}, {}, {array}, {mask}, {state}, &{result})) {{\n    tb_frame->pc = {pc};\n    tb_frame->yielded = true;\n    return {yielded};\n  }}\n#else\n  {result} = {synchronous};\n#endif",
            constructor.cid,
            constructor.layout.words.len()
        )
        .unwrap();
        Ok(result)
    }

    pub(super) fn fields(
        &mut self,
        name: &str,
        value: &str,
        output: &mut Body,
    ) -> Result<Vec<String>, CompileError> {
        self.node_fields(name, value, false, output)
    }

    /// Borrow the shell, but return owned field values for temporary boxing.
    /// The runtime retains only cells marked as references and updates their
    /// stored wrappers before sharing them with the printer.
    fn node_fields(
        &mut self,
        name: &str,
        value: &str,
        borrowed: bool,
        output: &mut Body,
    ) -> Result<Vec<String>, CompileError> {
        let constructor = self.table.get(name)?.clone();
        let array = self.array(
            output,
            &vec!["0".to_owned(); constructor.layout.words.len()],
        )?;
        writeln!(
            output,
            "  {}(e, {value}, {}, {}, {}, {array});",
            if borrowed {
                "tb_c_borrow_fields"
            } else {
                "tb_c_fields"
            },
            constructor.cid,
            constructor.layout.words.len(),
            packed(&constructor.layout)
        )
        .unwrap();
        constructor
            .layout
            .arms
            .as_ref()
            .ok_or_else(|| CompileError::new("missing C node fields"))?[0]
            .fields
            .iter()
            .map(|field| {
                let conversion = self.conversion_use(&field.layout, output)?;
                self.hold(
                    output,
                    &format!("tb_box_{conversion}(e, {array} + {})", field.offset),
                )
            })
            .collect()
    }

    /// A merged finite layout can put a raw W64 and a reference in the same
    /// slot in different arms. Record the selected arm's exact ownership,
    /// rather than interpreting arbitrary raw words as native Terms.
    pub(super) fn ownership_mask(
        &mut self,
        layout: &Layout,
        input: &str,
        output: &mut Body,
    ) -> Result<String, CompileError> {
        let mask = self.array(output, &vec!["0".to_owned(); layout.words.len()])?;
        fill_ownership_mask(layout, input, &mask, 0, output);
        Ok(mask)
    }

    pub(super) fn conversion_use(
        &mut self,
        layout: &Layout,
        output: &mut Body,
    ) -> Result<usize, CompileError> {
        let index = self.conversion(layout)?;
        output.dependencies.conversions.insert(index);
        Ok(index)
    }

    fn conversion(&mut self, layout: &Layout) -> Result<usize, CompileError> {
        let key = format!("{layout:?}");
        if let Some(index) = self.conversion_ids.get(&key) {
            return Ok(*index);
        }
        if self.conversions.len() >= 4096 {
            return Err(CompileError::new(
                "executable C layout conversion budget exhausted",
            ));
        }
        let index = self.conversions.len();
        self.conversions.push(String::new());
        self.conversion_dependencies
            .push(BodyDependencies::default());
        self.conversion_ids.insert(key, index);
        let mut boxing = Body::new("  (void)e; (void)input;\n");
        let mut unboxing = Body::new("  (void)e; (void)value; (void)output;\n");
        if let Some(arms) = &layout.arms {
            if arms.len() > 1 {
                boxing.push_str("  switch (input[0]) {\n");
            }
            unboxing.push_str("  switch (term_aux(value)) {\n");
            for (arm_index, arm) in arms.iter().enumerate() {
                if arms.len() > 1 {
                    writeln!(boxing, "  case {arm_index}: {{").unwrap();
                }
                let mut values = Vec::new();
                for field in &arm.fields {
                    let conversion = self.conversion_use(&field.layout, &mut boxing)?;
                    values.push(self.hold(
                        &mut boxing,
                        &format!("tb_box_{conversion}(e, input + {})", field.offset),
                    )?);
                }
                let value = self.construct(&arm.name, &values, &mut boxing)?;
                writeln!(boxing, "  return {value};").unwrap();
                if arms.len() > 1 {
                    boxing.push_str("  }\n");
                }
                let cid = self.table.get(&arm.name)?.cid;
                writeln!(unboxing, "  case {cid}: {{").unwrap();
                for word in 0..layout.words.len() {
                    writeln!(unboxing, "  output[{word}] = 0;").unwrap();
                }
                if arms.len() > 1 {
                    writeln!(unboxing, "  output[0] = {arm_index};").unwrap();
                }
                let fields = self.fields(&arm.name, "value", &mut unboxing)?;
                for (field, value) in arm.fields.iter().zip(fields) {
                    let conversion = self.conversion_use(&field.layout, &mut unboxing)?;
                    writeln!(
                        unboxing,
                        "  tb_unbox_{conversion}(e, {value}, output + {});",
                        field.offset
                    )
                    .unwrap();
                }
                unboxing.push_str("  return;\n  }\n");
            }
            if arms.len() > 1 {
                boxing.push_str(
                    "  default: err_fail(\"invalid finite C layout discriminator\");\n  }\n",
                );
            }
            if arms.is_empty() {
                boxing.push_str("  err_fail(\"empty C layout\");\n");
            }
            unboxing.push_str(
                "  default: err_fail(\"constructor does not match its C layout\");\n  }\n",
            );
        } else {
            let cast = if layout.words == [Kind::W32] {
                "(u32)"
            } else {
                ""
            };
            writeln!(boxing, "  return {cast}input[0];").unwrap();
            writeln!(unboxing, "  output[0] = {cast}value;").unwrap();
        }
        let mut dependencies = boxing.dependencies.clone();
        dependencies.extend(&unboxing.dependencies);
        self.conversion_dependencies[index] = dependencies;
        let mut source = boxing.function(
            &format!("tb_box_{index}"),
            "const Env *e, const Term *input",
            "e, input",
            FunctionResult::Term,
        );
        source.push_str(&unboxing.function(
            &format!("tb_unbox_{index}"),
            "const Env *e, Term value, Term *output",
            "e, value, output",
            FunctionResult::Void,
        ));
        self.conversions[index] = source;
        Ok(index)
    }

    pub(super) fn array_layout(
        &mut self,
        ty: &TermRef,
    ) -> Result<(bool, usize, Layout), CompileError> {
        let ty = self
            .program
            .expose_type(ty)
            .map_err(|error| CompileError::new(error.to_string()))?;
        let Term::Adt { name, args, .. } = ty.as_ref() else {
            return Err(CompileError::new("C Array needs a known element type"));
        };
        if name != "Array" || !self.program.base_names.contains(name) || args.len() != 1 {
            return Err(CompileError::new("C Array needs its sealed native type"));
        }
        let element = self
            .program
            .expose_type(&args[0])
            .map_err(|error| CompileError::new(error.to_string()))?;
        // Preserve upstream arr_open: the outer element must be a datatype.
        // Function fields inside that datatype can still be stably boxed.
        if !matches!(element.as_ref(), Term::Adt { .. }) {
            return Err(CompileError::new(
                "C Array has an unspecialized element type",
            ));
        }
        let layout = self.layouts.stable_layout(&element)?;
        let words = layout.words.len().max(1).next_power_of_two();
        let lgs = words.trailing_zeros() as usize;
        let arr = layout.words.iter().any(|word| *word != Kind::W32);
        Ok((arr, lgs, layout))
    }

    pub(super) fn array_constructor(
        &mut self,
        name: &str,
        ty: &TermRef,
        values: &[String],
        output: &mut Body,
    ) -> Result<String, CompileError> {
        if name == "ANode" {
            let left = self.hold(output, &format!("tb_c_blk_unique(e, {})", values[0]))?;
            let right = self.hold(output, &format!("tb_c_blk_unique(e, {})", values[1]))?;
            return self.hold(output, &format!("tb_c_blk_node(e, {left}, {right})"));
        }
        let (arr, lgs, layout) = self.array_layout(ty)?;
        let conversion = self.conversion_use(&layout, output)?;
        let array = self.array(output, &vec!["0".to_owned(); layout.words.len()])?;
        writeln!(
            output,
            "  tb_unbox_{conversion}(e, {}, {array});",
            values[0]
        )
        .unwrap();
        let mask = self.ownership_mask(&layout, &array, output)?;
        self.hold(
            output,
            &format!(
                "tb_c_blk_new(e, {arr}, 0, {lgs}, {}, {array}, {mask})",
                layout.words.len()
            ),
        )
    }

    pub(super) fn array_fields(
        &mut self,
        name: &str,
        ty: &TermRef,
        value: &str,
        output: &mut Body,
    ) -> Result<Vec<String>, CompileError> {
        let value = self.hold(output, &format!("tb_c_blk_unique(e, {value})"))?;
        if name == "ANode" {
            return Ok(vec![
                self.hold(output, &format!("tb_c_blk_half(e, {value}, 0)"))?,
                self.hold(output, &format!("tb_c_blk_half(e, {value}, 1)"))?,
            ]);
        }
        let (arr, _, layout) = self.array_layout(ty)?;
        let values = (0..layout.words.len())
            .map(|index| format!("blk_read(e->mem, {arr}, term_loc({value}), {index})"))
            .collect::<Vec<_>>();
        let array = self.array(output, &values)?;
        writeln!(output, "  tb_c_blk_free(e, {value});").unwrap();
        let conversion = self.conversion_use(&layout, output)?;
        Ok(vec![self.hold(
            output,
            &format!("tb_box_{conversion}(e, {array})"),
        )?])
    }

    pub(super) fn array_operation(
        &mut self,
        name: &str,
        ty: &TermRef,
        arguments: &[String],
        output: &mut Body,
    ) -> Result<String, CompileError> {
        let array_ty = find_array_type(self.program, ty, 0)?
            .ok_or_else(|| CompileError::new(format!("C {name} needs a specialized Array type")))?;
        if name == "Array.new" {
            return self.array_new(&array_ty, arguments, output);
        }
        let (arr, lgs, layout) = self.array_layout(&array_ty)?;
        let a = self.hold(output, &format!("tb_c_blk_unique(e, {})", arguments[0]))?;
        if name == "Array.clone" {
            let clone = self.hold(output, &format!("tb_c_blk_copy(e, {a})"))?;
            return self.construct("Tuple", &[a.clone(), clone], output);
        }
        if name == "Array.size" {
            return self.construct(
                "Tuple",
                &[
                    a.clone(),
                    format!("(UINT64_C(1) << (blk_cls({a}) - {lgs}))"),
                ],
                output,
            );
        }
        let conversion = self.conversion_use(&layout, output)?;
        let offset = self.hold(output, &format!("blk_at({a}, {}, {lgs})", arguments[1]))?;
        let previous = if name == "Array.get" || name == "Array.swap" {
            let cells = (0..layout.words.len())
                .map(|index| {
                    if name == "Array.get" && layout.words[index] == Kind::Box {
                        format!("tb_c_blk_keep(e, term_loc({a}) + (u32){offset} + {index})")
                    } else {
                        format!("blk_read(e->mem, {arr}, term_loc({a}), (u32){offset} + {index})")
                    }
                })
                .collect::<Vec<_>>();
            let array = self.array(output, &cells)?;
            Some(self.hold(output, &format!("tb_box_{conversion}(e, {array})"))?)
        } else {
            None
        };
        if name != "Array.get" {
            let array = self.array(output, &vec!["0".to_owned(); layout.words.len()])?;
            writeln!(
                output,
                "  tb_unbox_{conversion}(e, {}, {array});",
                arguments[2]
            )
            .unwrap();
            let mask = self.ownership_mask(&layout, &array, output)?;
            for index in 0..layout.words.len() {
                writeln!(output, "  tb_c_blk_write(e, {arr}, term_loc({a}), (u32){offset} + {index}, {array}[{index}], {mask}[{index}] != 0, {});", name == "Array.set").unwrap();
            }
        }
        if let Some(previous) = previous {
            self.construct("Tuple", &[a.clone(), previous], output)
        } else {
            Ok(a.clone())
        }
    }

    fn array_new(
        &mut self,
        array_ty: &TermRef,
        arguments: &[String],
        output: &mut Body,
    ) -> Result<String, CompileError> {
        let (arr, lgs, layout) = self.array_layout(array_ty)?;
        let depth = self.hold(output, &arguments[0])?;
        let conversion = self.conversion_use(&layout, output)?;
        let values = self.array(output, &vec!["0".to_owned(); layout.words.len()])?;
        writeln!(
            output,
            "  tb_unbox_{conversion}(e, {}, {values});",
            arguments[1]
        )
        .unwrap();
        let mask = self.ownership_mask(&layout, &values, output)?;
        let operands = format!("e, {arr}, {depth}, {lgs}, {}, {values}", layout.words.len());
        let synchronous = format!("tb_c_blk_new({operands}, {mask})");
        if !output.can_suspend || layout.words.contains(&Kind::Box) {
            return self.hold(output, &synchronous);
        }

        // The resume label follows every operand evaluation and ownership
        // transfer. Only raw fields may be reused without nested duplication.
        let result = self.hold(output, "0")?;
        let state = self.array(output, &vec!["0".to_owned(); DEVICE_ARRAY_NEW_STATE_WORDS])?;
        output.resumes += 1;
        let pc = output.resumes;
        let yielded = if output.words {
            "tb_segment_yield()"
        } else {
            "0"
        };
        writeln!(
            output,
            "tb_resume_{pc}: ;\n#ifdef __CUDA_ARCH__\n  if (!tb_device_array_new_raw({operands}, {state}, &{result})) {{\n    tb_frame->pc = {pc};\n    tb_frame->yielded = true;\n    return {yielded};\n  }}\n#else\n  {result} = {synchronous};\n#endif"
        )
        .unwrap();
        Ok(result)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "The complete typed printer traversal keeps every native and constructor shape explicit."
    )]
    pub(super) fn printer(&mut self, ty: &TermRef, depth: usize) -> Result<usize, CompileError> {
        if depth > 96 || self.printers.len() >= 256 {
            return Err(CompileError::new("C pure printer type budget exhausted"));
        }
        let ty = self
            .program
            .expose_type(ty)
            .map_err(|error| CompileError::new(error.to_string()))?;
        let key = format!("{ty:?}");
        if let Some(index) = self.printer_ids.get(&key) {
            return Ok(*index);
        }
        let index = self.printers.len();
        self.printers.push(String::new());
        self.printer_ids.insert(key, index);
        let mut output =
            Body::new("  (void)e; (void)value; (void)chain;\n  tb_show_step(depth);\n");
        match ty.as_ref() {
            Term::Eql { .. } => output.push_str("  tb_show_text(\"{==}\");\n"),
            Term::Adt { name, args, .. }
                if self.program.base_names.contains(name)
                    && matches!(name.as_str(), "Nat" | "U32" | "F32" | "Char" | "String") =>
            {
                writeln!(
                    output,
                    "  tb_c_show_native(e, value, {});",
                    match name.as_str() {
                        "Nat" => 0,
                        "U32" => 1,
                        "F32" => 2,
                        "Char" => 3,
                        _ => 4,
                    }
                )
                .unwrap();
            }
            Term::Adt { name, args, .. }
                if self.program.base_names.contains(name) && name == "Array" =>
            {
                let (arr, lgs, layout) = self.array_layout(&ty)?;
                let conversion = self.conversion_use(&layout, &mut output)?;
                let element = self.printer(&args[0], depth + 1)?;
                let at = self.hold(&mut output, "tb_c_peek(e, value)")?;
                output.push_str("  tb_show_text(\"[\");\n");
                writeln!(output, "  for (u32 i = 0; i < (UINT64_C(1) << (blk_cls(value) - {lgs})); ++i) {{\n  if (i) tb_show_text(\", \");").unwrap();
                let values = (0..layout.words.len())
                    .map(|word| {
                        if layout.words[word] == Kind::Box {
                            format!("tb_c_blk_keep(e, {at} + (i << {lgs}) + {word})")
                        } else {
                            format!("blk_read(e->mem, {arr}, {at}, (i << {lgs}) + {word})")
                        }
                    })
                    .collect::<Vec<_>>();
                let fields = self.array(&mut output, &values)?;
                let boxed = self.hold(&mut output, &format!("tb_box_{conversion}(e, {fields})"))?;
                writeln!(output, "  tb_show_{element}(e, {boxed}, depth + 1, 0);").unwrap();
                if boxed_layout(&layout) {
                    writeln!(output, "  tb_c_drop(e, {boxed});").unwrap();
                }
                output.push_str("  }\n  tb_show_text(\"]\");\n");
            }
            Term::Adt { name, args, .. } => {
                let datatype = self.program.datatypes[name].clone();
                output.push_str("  tb_reject_request(value);\n  switch (term_aux(value)) {\n");
                for constructor in datatype.constructors {
                    let cid = self.table.get(&constructor.name)?.cid;
                    let tag = self
                        .program
                        .constructor_tags
                        .get(&constructor.name)
                        .ok_or_else(|| CompileError::new("C printer missing constructor tag"))?
                        .clone();
                    let (open, close) = match tag.as_str() {
                        "Con" | "Nil" => ('[', ']'),
                        "Tuple" => ('(', ')'),
                        _ => ('{', '}'),
                    };
                    writeln!(output, "  case {cid}: {{").unwrap();
                    if open == '{' {
                        writeln!(output, "  tb_show_text({});", c_string(&(tag + "{"))).unwrap();
                    } else {
                        writeln!(output, "  if (chain != '{open}') tb_show_text(\"{open}\");")
                            .unwrap();
                    }
                    let values = self.node_fields(&constructor.name, "value", true, &mut output)?;
                    let node = self.table.get(&constructor.name)?.layout.clone();
                    let layouts = &node.arms.as_ref().expect("node has an arm")[0].fields;
                    if constructor
                        .fields
                        .iter()
                        .any(|field| field.quant == Quant::None)
                    {
                        return Err(CompileError::new(
                            "C pure main with erased constructor fields cannot be printed",
                        ));
                    }
                    for (field_index, (field, value)) in
                        constructor.fields.iter().zip(values).enumerate()
                    {
                        let mut field_ty = Rc::clone(&field.ty);
                        for (parameter, argument) in datatype.parameters.iter().zip(args) {
                            field_ty = substitute(&field_ty, parameter.id, argument);
                        }
                        let printer = self.printer(&field_ty, depth + 1)?;
                        if open == '[' && field_index == 0 {
                            output.push_str("  if (chain == '[') tb_show_text(\", \");\n");
                        } else if open != '[' && field_index > 0 {
                            output.push_str("  tb_show_text(\", \");\n");
                        }
                        let next_chain = if field_index == 1 && open != '{' {
                            format!("'{open}'")
                        } else {
                            "0".to_owned()
                        };
                        writeln!(
                            output,
                            "  tb_show_{printer}(e, {value}, depth + 1, {next_chain});"
                        )
                        .unwrap();
                        if boxed_layout(&layouts[field_index].layout) {
                            writeln!(output, "  tb_c_drop(e, {value});").unwrap();
                        }
                    }
                    if open == '{' {
                        writeln!(output, "  tb_show_text(\"{close}\");").unwrap();
                    } else {
                        writeln!(
                            output,
                            "  if (chain != '{open}') tb_show_text(\"{close}\");"
                        )
                        .unwrap();
                    }
                    output.push_str("  return;\n  }\n");
                }
                output.push_str("  default: err_fail(\"C pure output constructor differs from its type\");\n  }\n");
            }
            _ => {
                return Err(CompileError::new(format!(
                    "C pure main type {ty} cannot be printed"
                )));
            }
        }
        self.printers[index] = output.function(
            &format!("tb_show_{index}"),
            "const Env *e, Term value, unsigned depth, char chain",
            "e, value, depth, chain",
            FunctionResult::Void,
        );
        Ok(index)
    }
}

fn boxed_layout(layout: &Layout) -> bool {
    layout.arms.is_some() || layout.words == [Kind::Box]
}

fn fill_ownership_mask(layout: &Layout, input: &str, mask: &str, offset: usize, output: &mut Body) {
    if let Some(arms) = &layout.arms {
        if arms.len() > 1 {
            writeln!(output, "  switch ({input}[{offset}]) {{").unwrap();
        }
        for (index, arm) in arms.iter().enumerate() {
            if arms.len() > 1 {
                writeln!(output, "  case {index}: {{").unwrap();
            }
            for field in &arm.fields {
                fill_ownership_mask(&field.layout, input, mask, offset + field.offset, output);
            }
            if arms.len() > 1 {
                output.push_str("  break;\n  }\n");
            }
        }
        if arms.len() > 1 {
            output.push_str(
                "  default: err_fail(\"invalid ownership layout discriminator\");\n  }\n",
            );
        }
    } else {
        for (index, kind) in layout.words.iter().enumerate() {
            if *kind == Kind::Box {
                writeln!(output, "  {mask}[{}] = 1;", offset + index).unwrap();
            }
        }
    }
}

fn find_array_type(
    program: &ExecutableProgram,
    ty: &TermRef,
    depth: usize,
) -> Result<Option<TermRef>, CompileError> {
    if depth > 96 {
        return Err(CompileError::new("C Array type search budget exhausted"));
    }
    let ty = program
        .expose_type(ty)
        .map_err(|error| CompileError::new(error.to_string()))?;
    match ty.as_ref() {
        Term::Adt { name, .. } if name == "Array" && program.base_names.contains(name) => {
            Ok(Some(ty))
        }
        Term::All { domain, body, .. } => {
            if let Some(found) = find_array_type(program, domain, depth + 1)? {
                return Ok(Some(found));
            }
            find_array_type(program, body, depth + 1)
        }
        Term::Adt { args, .. } => {
            for argument in args {
                if let Some(found) = find_array_type(program, argument, depth + 1)? {
                    return Ok(Some(found));
                }
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}
