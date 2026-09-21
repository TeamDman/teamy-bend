// SPDX-License-Identifier: Apache-2.0
// Segment/value lowering derived from Bend 2.0.5 comp.ts, Copyright 2026 HigherOrderCO.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
//! Typed word vectors for direct definitions and their generated continuations.

use super::Body;
use super::CompileError;
use super::DefinitionBody;
use super::ExecutableProgram;
use super::Expression;
use super::ExpressionKind;
use super::Field;
use super::Generator;
use super::Kind;
use super::Layout;
use super::OwnedLocal;
use super::Quant;
use super::Scope;
use super::Substitutions;
use super::Term;
use super::drop_owned;
use super::fork_group;
use super::forks;
use super::live_bindings;
use super::native;
use super::runtime_uses;
use super::specialize_segment_expression;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::rc::Rc;

#[derive(Clone)]
pub(super) struct Signature {
    pub(super) id: usize,
    pub(super) arguments: Vec<Layout>,
    pub(super) result: Layout,
}

#[derive(Clone)]
pub(super) struct Value {
    pub(super) layout: Layout,
    pub(super) words: Vec<String>,
}

#[derive(Clone)]
struct Local {
    value: Value,
    remaining: usize,
    owned: bool,
}

type FlatScope = BTreeMap<usize, Local>;

fn boxed() -> Layout {
    Layout {
        words: vec![Kind::Box],
        arms: None,
    }
}

fn result_layout(layout: Layout) -> Layout {
    if layout.words.is_empty() {
        boxed()
    } else {
        layout
    }
}

impl Generator<'_> {
    /// Direct entries consume upstream's raised telescope together. Leading
    /// returned lambdas count as parameters only when every match branch agrees;
    /// a leading let still ends raising and returns its ordinary callable value.
    pub(super) fn segment(
        &mut self,
        name: &str,
        substitutions: &Substitutions,
    ) -> Result<Option<Signature>, CompileError> {
        let key = format!("{name}:{substitutions:?}");
        if let Some(signature) = self.segment_ids.get(&key) {
            return Ok(signature.clone());
        }
        let definition = self.program.definitions[name].clone();
        let DefinitionBody::Ordinary(expression) = definition.body else {
            self.segment_ids.insert(key, None);
            return Ok(None);
        };
        if self.program.base_names.contains(name) && native::optimized(name) {
            self.segment_ids.insert(key, None);
            return Ok(None);
        }
        if self.segment_ids.len() >= 4096 {
            return Err(CompileError::new(
                "executable C segment specialization budget exhausted",
            ));
        }
        let declared = definition.parameters.len();
        let total = i64::try_from(declared)
            .ok()
            .and_then(|count| forks::raised_arguments(&expression, count))
            .and_then(|extra| declared.checked_add(extra));
        let Some(total) = total else {
            self.segment_ids.insert(key, None);
            return Ok(None);
        };
        // Preserve erased binders while specializing their uses. The original
        // raised telescope includes those binders even though its payload does
        // not, and the body traversal removes them without consuming a word.
        let expression = specialize_segment_expression(&expression, substitutions);
        let mut ty = Rc::clone(&expression.ty);
        let mut arguments = Vec::new();
        // Erased parameters still belong to the raised telescope. Stopping at
        // the last live binder leaves trailing erased binders in the result
        // type, hiding finite result words behind a spurious callable layout.
        for index in 0..total {
            let exposed = self
                .program
                .expose_type(&ty)
                .map_err(|e| CompileError::new(e.to_string()))?;
            let Term::All {
                quant,
                domain,
                body,
                ..
            } = exposed.as_ref()
            else {
                // An impossible match supplies upstream's raising sentinel;
                // the actual type telescope caps that additional prefix.
                if index >= declared {
                    break;
                }
                self.segment_ids.insert(key, None);
                return Ok(None);
            };
            if *quant != Quant::None {
                arguments.push(self.layouts.layout(domain)?);
            }
            ty = Rc::clone(body);
        }
        let result = result_layout(self.layouts.layout(&ty)?);
        let arity = arguments.iter().map(|layout| layout.words.len()).sum();
        let id = self.reserve_segment(arity, result.words.len())?;
        let signature = Signature {
            id,
            arguments,
            result,
        };
        // Recursive calls must see the registered signature before the body.
        self.segment_ids.insert(key, Some(signature.clone()));
        let mut body = Body::new_segment();
        body.push_str("  tb_tick();\n");
        let mut offset = 0;
        let mut values = Vec::new();
        for layout in &signature.arguments {
            let mut words = Vec::new();
            for _ in &layout.words {
                words.push(self.hold(&mut body, &format!("captures[{offset}]"))?);
                offset += 1;
            }
            values.push(Value {
                layout: layout.clone(),
                words,
            });
        }
        self.flat_body(
            &expression,
            values,
            &mut FlatScope::new(),
            &mut body,
            &signature.result,
        )?;
        self.finish_segment(id, body);
        Ok(Some(signature))
    }

    pub(super) fn segment_arguments(
        &mut self,
        signature: &Signature,
        arguments: &[String],
        output: &mut Body,
    ) -> Result<(String, String), CompileError> {
        if arguments.len() != signature.arguments.len() {
            return Err(CompileError::new(
                "C segment logical argument count differs",
            ));
        }
        let mut values = Vec::new();
        for (argument, layout) in arguments.iter().zip(&signature.arguments) {
            values.push(self.flat_convert(
                Value {
                    layout: boxed(),
                    words: vec![argument.clone()],
                },
                layout,
                output,
            )?);
        }
        self.flat_arrays(&values, output)
    }

    pub(super) fn segment_result(
        &mut self,
        signature: &Signature,
        input: &str,
        output: &mut Body,
    ) -> Result<String, CompileError> {
        let conversion = self.conversion(&signature.result)?;
        self.hold(output, &format!("tb_box_{conversion}(e, {input})"))
    }

    pub(super) fn pending_words(
        &mut self,
        output: &mut Body,
        task: &str,
        layout: &Layout,
        tail: bool,
    ) -> Result<Value, CompileError> {
        if tail {
            return Err(CompileError::new(
                "tail word calls must return their task directly",
            ));
        }
        self.fresh()?;
        let slot = output.reserve(layout.words.len().max(1));
        output.resumes += 1;
        let pc = output.resumes;
        writeln!(output, "  tb_frame->pc = {pc}; tb_frame->destination = {slot}; tb_frame->expected = {}; tb_frame->waiting = true;", layout.words.len()).unwrap();
        if output.words {
            writeln!(output, "  return tb_segment_task({task});").unwrap();
        } else {
            writeln!(output, "  return {task};").unwrap();
        }
        writeln!(output, "tb_resume_{pc}: ;").unwrap();
        Ok(Value {
            layout: layout.clone(),
            words: (0..layout.words.len())
                .map(|i| format!("tb_values[{}]", slot + i))
                .collect(),
        })
    }

    fn flat_arrays(
        &mut self,
        values: &[Value],
        output: &mut Body,
    ) -> Result<(String, String), CompileError> {
        let mut words = Vec::new();
        let mut masks = Vec::new();
        for value in values {
            let input = self.array(output, &value.words)?;
            let mask = self.ownership_mask(&value.layout, &input, output)?;
            words.extend(value.words.iter().cloned());
            masks.extend((0..value.words.len()).map(|i| format!("{mask}[{i}]")));
        }
        Ok((self.array(output, &words)?, self.array(output, &masks)?))
    }

    fn flat_convert(
        &mut self,
        value: Value,
        layout: &Layout,
        output: &mut Body,
    ) -> Result<Value, CompileError> {
        if &value.layout == layout {
            return Ok(value);
        }
        if value.layout.arms.is_none() && layout.arms.is_none() {
            let words = if layout.words == [Kind::W32] {
                vec![self.hold(output, &format!("(u32){}", value.words[0]))?]
            } else {
                value.words
            };
            return Ok(Value {
                layout: layout.clone(),
                words,
            });
        }
        let input = self.array(output, &value.words)?;
        let from = self.conversion(&value.layout)?;
        let word = self.hold(output, &format!("tb_box_{from}(e, {input})"))?;
        let to = self.conversion(layout)?;
        let array = self.array(output, &vec!["0".into(); layout.words.len()])?;
        writeln!(output, "  tb_unbox_{to}(e, {word}, {array});").unwrap();
        let words = (0..layout.words.len())
            .map(|i| format!("{array}[{i}]"))
            .map(|word| self.hold(output, &word))
            .collect::<Result<_, _>>()?;
        Ok(Value {
            layout: layout.clone(),
            words,
        })
    }

    fn flat_box(&mut self, value: Value, output: &mut Body) -> Result<String, CompileError> {
        Ok(self.flat_convert(value, &boxed(), output)?.words[0].clone())
    }

    fn flat_drop(&mut self, value: &Value, output: &mut Body) -> Result<(), CompileError> {
        let input = self.array(output, &value.words)?;
        let mask = self.ownership_mask(&value.layout, &input, output)?;
        for (i, word) in value.words.iter().enumerate() {
            writeln!(output, "  if ({mask}[{i}]) tb_c_drop(e, {word});").unwrap();
        }
        Ok(())
    }

    fn flat_take(
        &mut self,
        id: usize,
        amount: usize,
        scope: &mut FlatScope,
        output: &mut Body,
    ) -> Result<Value, CompileError> {
        let local = scope
            .get_mut(&id)
            .ok_or_else(|| CompileError::new(format!("unbound C segment variable {id}")))?;
        if !local.owned || amount == 0 || local.remaining < amount {
            return Err(CompileError::new(format!(
                "unbalanced C segment ownership for variable {id}"
            )));
        }
        local.remaining -= amount;
        if local.remaining == 0 {
            local.owned = false;
            return Ok(local.value.clone());
        }
        let input = self.array(output, &local.value.words)?;
        let mask = self.ownership_mask(&local.value.layout, &input, output)?;
        let words = local
            .value
            .words
            .iter()
            .enumerate()
            .map(|(i, word)| {
                self.hold(
                    output,
                    &format!("{mask}[{i}] ? tb_c_duplicate(e, &{word}) : {word}"),
                )
            })
            .collect::<Result<_, _>>()?;
        Ok(Value {
            layout: local.value.layout.clone(),
            words,
        })
    }

    fn flat_drop_scope(
        &mut self,
        scope: &mut FlatScope,
        output: &mut Body,
        unused: bool,
    ) -> Result<(), CompileError> {
        for local in scope
            .values_mut()
            .filter(|local| local.owned && (!unused || local.remaining == 0))
        {
            self.flat_drop(&local.value, output)?;
            local.owned = false;
        }
        Ok(())
    }

    fn flat_legacy(
        &mut self,
        expression: &Expression,
        scope: &mut FlatScope,
        output: &mut Body,
        tail: bool,
    ) -> Result<Value, CompileError> {
        let uses = runtime_uses(self.program, expression);
        let mut inner = Scope::new();
        for (id, count) in uses {
            let value = self.flat_take(id, count, scope, output)?;
            let word = self.flat_box(value, output)?;
            inner.insert(id, OwnedLocal::new(word, count));
        }
        let word = self.expression(expression, &mut inner, output, tail)?;
        drop_owned(&mut inner, output);
        Ok(Value {
            layout: boxed(),
            words: vec![word],
        })
    }

    fn flat_expression(
        &mut self,
        expression: &Expression,
        scope: &mut FlatScope,
        output: &mut Body,
    ) -> Result<Value, CompileError> {
        self.fresh()?;
        match &expression.kind {
            ExpressionKind::Variable(id) => self.flat_take(*id, 1, scope, output),
            ExpressionKind::Erased => Ok(Value {
                layout: boxed(),
                words: vec![self.hold(output, "0")?],
            }),
            ExpressionKind::Lambda { parameter, body } if parameter.quant == Quant::None => {
                self.flat_expression(body, scope, output)
            }
            ExpressionKind::Constructor { .. } => self.flat_constructor(expression, scope, output),
            ExpressionKind::Apply { .. } | ExpressionKind::Definition(_) => {
                if let Some((signature, values)) = self.flat_call(expression, scope, output)? {
                    let (words, owned) = self.flat_arrays(&values, output)?;
                    let count = values.iter().map(|value| value.words.len()).sum::<usize>();
                    return self.pending_words(
                        output,
                        &format!(
                            "tb_c_word_task(e, {}, {count}, {words}, {owned})",
                            signature.id + 2
                        ),
                        &signature.result,
                        false,
                    );
                }
                self.flat_legacy(expression, scope, output, false)
            }
            ExpressionKind::Let { bindings, body } => {
                let uses = runtime_uses(self.program, body);
                let bindings = live_bindings(bindings, &uses);
                if fork_group(self.program, &bindings) {
                    return self.flat_fork(&bindings, body, scope, output);
                }
                let mut locals = Vec::new();
                for binding in bindings {
                    let value = self.flat_expression(&binding.value, scope, output)?;
                    locals.push((binding.binder.id, value));
                }
                for (id, value) in &locals {
                    scope.insert(
                        *id,
                        Local {
                            value: value.clone(),
                            remaining: uses.get(id).copied().unwrap_or(0),
                            owned: true,
                        },
                    );
                }
                self.flat_drop_scope(scope, output, true)?;
                let value = self.flat_expression(body, scope, output)?;
                for (id, _) in locals {
                    if let Some(local) = scope.remove(&id)
                        && local.owned
                    {
                        self.flat_drop(&local.value, output)?;
                    }
                }
                Ok(value)
            }
            ExpressionKind::Lambda { .. }
            | ExpressionKind::Match { .. }
            | ExpressionKind::Absurd { .. } => self.flat_legacy(expression, scope, output, false),
        }
    }

    /// Construct a finite arm directly in its word layout, leaving heap and
    /// native constructor representations at the existing boxed boundary.
    fn flat_constructor(
        &mut self,
        expression: &Expression,
        scope: &mut FlatScope,
        output: &mut Body,
    ) -> Result<Value, CompileError> {
        let ExpressionKind::Constructor { name, fields, .. } = &expression.kind else {
            unreachable!()
        };
        let layout = self.layouts.layout(&expression.ty)?;
        let Some(arms) = &layout.arms else {
            return self.flat_legacy(expression, scope, output, false);
        };
        let arm_index = arms
            .iter()
            .position(|arm| arm.name == *name)
            .ok_or_else(|| CompileError::new("C segment constructor outside its layout"))?;
        let arm = &arms[arm_index];
        if fields
            .iter()
            .filter(|field| field.binder.quant != Quant::None)
            .count()
            != arm.fields.len()
        {
            return Err(CompileError::new(
                "C segment constructor field count differs",
            ));
        }
        let mut words = vec!["0".to_owned(); layout.words.len()];
        if arms.len() > 1 {
            words[0] = arm_index.to_string();
        }
        for (field, target) in fields
            .iter()
            .filter(|field| field.binder.quant != Quant::None)
            .zip(&arm.fields)
        {
            let value = self.flat_expression(&field.value, scope, output)?;
            let value = self.flat_convert(value, &target.layout, output)?;
            words[target.offset..target.offset + value.words.len()].clone_from_slice(&value.words);
        }
        let words = words
            .iter()
            .map(|word| self.hold(output, word))
            .collect::<Result<_, _>>()?;
        Ok(Value { layout, words })
    }

    /// Classification is pure until a usable signature is known: a rejected
    /// direct optimization must not evaluate or consume any source operand.
    fn flat_call(
        &mut self,
        expression: &Expression,
        scope: &mut FlatScope,
        output: &mut Body,
    ) -> Result<Option<(Signature, Vec<Value>)>, CompileError> {
        let mut head = expression;
        let mut arguments = Vec::new();
        while let ExpressionKind::Apply {
            function,
            argument,
            quant,
        } = &head.kind
        {
            arguments.push((argument.as_ref(), *quant));
            head = function;
        }
        arguments.reverse();
        let ExpressionKind::Definition(name) = &head.kind else {
            return Ok(None);
        };
        let supplied = arguments.iter().filter(|(_, q)| *q != Quant::None).count();
        if forks::required_arguments(self.program, name) != Some(supplied) {
            return Ok(None);
        }
        let substitutions = self.instantiation(name, &arguments);
        let Some(signature) = self.segment(name, &substitutions)? else {
            return Ok(None);
        };
        if supplied != signature.arguments.len() {
            return Ok(None);
        }
        let mut values = Vec::new();
        for ((argument, _), layout) in arguments
            .iter()
            .filter(|(_, q)| *q != Quant::None)
            .zip(&signature.arguments)
        {
            let value = self.flat_expression(argument, scope, output)?;
            values.push(self.flat_convert(value, layout, output)?);
        }
        Ok(Some((signature, values)))
    }

    fn flat_body(
        &mut self,
        expression: &Expression,
        mut arguments: Vec<Value>,
        scope: &mut FlatScope,
        output: &mut Body,
        result: &Layout,
    ) -> Result<(), CompileError> {
        let uses = invocation_uses(self.program, expression, arguments.len());
        for (id, local) in scope.iter_mut().filter(|(_, local)| local.owned) {
            local.remaining = uses.get(id).copied().unwrap_or(0);
        }
        self.flat_drop_scope(scope, output, true)?;
        match &expression.kind {
            ExpressionKind::Lambda { parameter, body } if parameter.quant == Quant::None => {
                self.flat_body(body, arguments, scope, output, result)
            }
            ExpressionKind::Lambda { parameter, body } if !arguments.is_empty() => {
                let value = arguments.remove(0);
                scope.insert(
                    parameter.id,
                    Local {
                        value,
                        remaining: 0,
                        owned: true,
                    },
                );
                self.flat_body(body, arguments, scope, output, result)
            }
            ExpressionKind::Match { .. } if !arguments.is_empty() => {
                self.flat_match(expression, arguments, scope, output, result)
            }
            ExpressionKind::Absurd { .. } if !arguments.is_empty() => {
                output.push_str("  err_fail(\"impossible C segment branch\");\n");
                Ok(())
            }
            ExpressionKind::Let { bindings, body } if arguments.is_empty() => {
                let uses = runtime_uses(self.program, body);
                let bindings = live_bindings(bindings, &uses);
                if fork_group(self.program, &bindings) {
                    let value = self.flat_fork(&bindings, body, scope, output)?;
                    return self.flat_return(value, scope, output, result);
                }
                let mut locals = Vec::new();
                for binding in bindings {
                    let value = self.flat_expression(&binding.value, scope, output)?;
                    locals.push((binding.binder.id, value));
                }
                for (id, value) in locals {
                    scope.insert(
                        id,
                        Local {
                            value,
                            remaining: 0,
                            owned: true,
                        },
                    );
                }
                self.flat_body(body, arguments, scope, output, result)
            }
            _ => {
                if arguments.is_empty()
                    && let Some((signature, values)) = self.flat_call(expression, scope, output)?
                {
                    let (words, owned) = self.flat_arrays(&values, output)?;
                    let count = values.iter().map(|value| value.words.len()).sum::<usize>();
                    let task = format!(
                        "tb_c_word_task(e, {}, {count}, {words}, {owned})",
                        signature.id + 2
                    );
                    if signature.result == *result {
                        self.flat_drop_scope(scope, output, false)?;
                        writeln!(output, "  return tb_segment_task({task});").unwrap();
                        return Ok(());
                    }
                    let value = self.pending_words(output, &task, &signature.result, false)?;
                    return self.flat_return(value, scope, output, result);
                }
                if arguments.is_empty()
                    && let Some(mode) = legacy_tail_mode(result)
                    && legacy_call(expression)
                {
                    // Preserve the final unary call as task control. Scalar
                    // casts and ownership travel with its result boundary,
                    // rather than retaining a frame at each indirect tail.
                    let task = self.flat_legacy(expression, scope, output, true)?;
                    return self.flat_legacy_return(&task.words[0], mode, scope, output);
                }
                let mut value = self.flat_expression(expression, scope, output)?;
                let count = arguments.len();
                for (index, argument) in arguments.into_iter().enumerate() {
                    let function = self.flat_box(value, output)?;
                    let argument = self.flat_box(argument, output)?;
                    let mode = if index + 1 == count {
                        legacy_tail_mode(result)
                    } else {
                        None
                    };
                    let word = self.apply(output, &function, &argument, mode.is_some())?;
                    if let Some(mode) = mode {
                        return self.flat_legacy_return(&word, mode, scope, output);
                    }
                    value = Value {
                        layout: boxed(),
                        words: vec![word],
                    };
                }
                self.flat_return(value, scope, output, result)
            }
        }
    }

    fn flat_legacy_return(
        &mut self,
        task: &str,
        mode: &str,
        scope: &mut FlatScope,
        output: &mut Body,
    ) -> Result<(), CompileError> {
        self.flat_drop_scope(scope, output, false)?;
        if mode == "TB_RESULT_NONE" {
            writeln!(output, "  return tb_segment_task({task});").unwrap();
        } else {
            writeln!(
                output,
                "  return tb_segment_task(tb_tail_result(tb_frame, {task}, {mode}));"
            )
            .unwrap();
        }
        Ok(())
    }

    fn flat_return(
        &mut self,
        value: Value,
        scope: &mut FlatScope,
        output: &mut Body,
        result: &Layout,
    ) -> Result<(), CompileError> {
        let value = self.flat_convert(value, result, output)?;
        self.flat_drop_scope(scope, output, false)?;
        let count = value.words.len();
        let (words, owned) = self.flat_arrays(&[value], output)?;
        writeln!(
            output,
            "  return tb_segment_words({words}, {owned}, {count});"
        )
        .unwrap();
        Ok(())
    }

    fn flat_match(
        &mut self,
        expression: &Expression,
        mut arguments: Vec<Value>,
        scope: &mut FlatScope,
        output: &mut Body,
        result: &Layout,
    ) -> Result<(), CompileError> {
        let ExpressionKind::Match {
            parameter,
            fields,
            arm,
            fallback,
            ..
        } = &expression.kind
        else {
            unreachable!()
        };
        if parameter.quant == Quant::None {
            return Err(CompileError::new(
                "executable C cannot match an erased scrutinee",
            ));
        }
        let value = arguments.remove(0);
        let layout = self.layouts.layout(&parameter.ty)?;
        let value = self.flat_convert(value, &layout, output)?;
        scope.insert(
            parameter.id,
            Local {
                value: value.clone(),
                remaining: 1,
                owned: true,
            },
        );
        let condition = self.flat_match_condition(expression, &value, output)?;
        if condition != "1" {
            writeln!(output, "  if ({condition}) {{").unwrap();
        }
        let mut inner = scope.clone();
        let live_fields = fields
            .iter()
            .filter(|field| field.quant != Quant::None)
            .count();
        let uses = invocation_uses(self.program, arm, live_fields + arguments.len());
        for (id, local) in &mut inner {
            local.remaining = uses.get(id).copied().unwrap_or(0) + usize::from(*id == parameter.id);
        }
        self.flat_drop_scope(&mut inner, output, true)?;
        let scrutinee = self.flat_take(parameter.id, 1, &mut inner, output)?;
        let mut values = self.flat_match_fields(expression, scrutinee, output)?;
        if values.len() != live_fields {
            return Err(CompileError::new("C segment match field count differs"));
        }
        values.extend(arguments.iter().cloned());
        self.flat_body(arm, values, &mut inner, output, result)?;
        if condition != "1" {
            output.push_str("  } else {\n");
            let mut inner = scope.clone();
            let uses = invocation_uses(self.program, fallback, arguments.len() + 1);
            for (id, local) in &mut inner {
                local.remaining =
                    uses.get(id).copied().unwrap_or(0) + usize::from(*id == parameter.id);
            }
            self.flat_drop_scope(&mut inner, output, true)?;
            let scrutinee = self.flat_take(parameter.id, 1, &mut inner, output)?;
            arguments.insert(0, scrutinee);
            self.flat_body(fallback, arguments, &mut inner, output, result)?;
            output.push_str("  }\n");
        }
        for local in scope.values_mut() {
            local.owned = false;
        }
        Ok(())
    }

    fn flat_match_condition(
        &mut self,
        expression: &Expression,
        value: &Value,
        output: &mut Body,
    ) -> Result<String, CompileError> {
        let ExpressionKind::Match {
            parameter,
            owner,
            constructor,
            ..
        } = &expression.kind
        else {
            unreachable!()
        };
        let layout = &value.layout;
        let native = self.program.base_names.contains(owner);
        let condition = if let Some(arms) = &layout.arms {
            if arms.len() == 1 {
                "1".into()
            } else {
                let tag = arms
                    .iter()
                    .position(|arm| arm.name == *constructor)
                    .ok_or_else(|| CompileError::new("C flat match outside layout"))?;
                format!("{} == {tag}", value.words[0])
            }
        } else if native && owner == "Nat" {
            format!(
                "{} {} 0",
                value.words[0],
                if constructor == "Zero" { "==" } else { "!=" }
            )
        } else if native && matches!(owner.as_str(), "U32" | "F32") {
            "1".into()
        } else if native && owner == "Array" {
            let (_, lgs, _) = self.array_layout(&parameter.ty)?;
            format!(
                "blk_cls({}) {} {lgs}",
                value.words[0],
                if constructor == "ALeaf" { "==" } else { "!=" }
            )
        } else {
            format!(
                "term_aux({}) == {}",
                value.words[0],
                self.table.get(constructor)?.cid
            )
        };
        if layout.arms.is_none() && !(native && matches!(owner.as_str(), "Nat" | "U32" | "F32")) {
            writeln!(output, "  tb_reject_request({});", value.words[0]).unwrap();
        }
        Ok(condition)
    }

    /// Extract the selected arm's owned fields. Finite words transfer directly;
    /// native and heap representations use their existing consuming helpers.
    fn flat_match_fields(
        &mut self,
        expression: &Expression,
        scrutinee: Value,
        output: &mut Body,
    ) -> Result<Vec<Value>, CompileError> {
        let ExpressionKind::Match {
            parameter,
            owner,
            constructor,
            ..
        } = &expression.kind
        else {
            unreachable!()
        };
        let layout = &scrutinee.layout;
        let native = self.program.base_names.contains(owner);
        let values = if let Some(arms) = &layout.arms {
            let selected = arms
                .iter()
                .find(|arm| arm.name == *constructor)
                .ok_or_else(|| CompileError::new("C flat constructor fields missing"))?;
            selected
                .fields
                .iter()
                .map(|field| Value {
                    layout: field.layout.clone(),
                    words: scrutinee.words[field.offset..field.offset + field.layout.words.len()]
                        .to_vec(),
                })
                .collect::<Vec<_>>()
        } else if native && owner == "Nat" {
            if constructor == "Zero" {
                Vec::new()
            } else {
                vec![Value {
                    layout: layout.clone(),
                    words: vec![self.hold(output, &format!("{} - 1", scrutinee.words[0]))?],
                }]
            }
        } else {
            let word = self.flat_box(scrutinee, output)?;
            let words = if native && matches!(owner.as_str(), "U32" | "F32") {
                vec![self.hold(output, &format!("tb_c_word(e, (u32){word})"))?]
            } else if native && owner == "Array" {
                self.array_fields(constructor, &parameter.ty, &word, output)?
            } else {
                self.fields(constructor, &word, output)?
            };
            words
                .into_iter()
                .map(|word| Value {
                    layout: boxed(),
                    words: vec![word],
                })
                .collect()
        };
        Ok(values)
    }

    fn flat_fork(
        &mut self,
        bindings: &[&Field],
        expression: &Expression,
        scope: &mut FlatScope,
        output: &mut Body,
    ) -> Result<Value, CompileError> {
        let mut tasks = Vec::new();
        let mut results = Vec::new();
        for binding in bindings {
            if let Some((signature, values)) = self.flat_call(&binding.value, scope, output)? {
                let count = values.iter().map(|value| value.words.len()).sum::<usize>();
                let (words, owned) = self.flat_arrays(&values, output)?;
                tasks.push(self.hold(
                    output,
                    &format!(
                        "tb_c_word_task(e, {}, {count}, {words}, {owned})",
                        signature.id + 2
                    ),
                )?);
                results.push(signature.result);
            } else {
                let uses = runtime_uses(self.program, &binding.value);
                let mut inner = Scope::new();
                for (id, count) in uses {
                    let value = self.flat_take(id, count, scope, output)?;
                    let word = self.flat_box(value, output)?;
                    inner.insert(id, OwnedLocal::new(word, count));
                }
                let (function, argument) =
                    self.fork_application(&binding.value, &mut inner, output)?;
                drop_owned(&mut inner, output);
                tasks.push(self.hold(
                    output,
                    &format!("tb_c_tail_apply(e, {function}, {argument})"),
                )?);
                results.push(boxed());
            }
        }
        let uses = runtime_uses(self.program, expression);
        let ids = scope
            .keys()
            .filter(|id| uses.contains_key(id))
            .copied()
            .collect::<Vec<_>>();
        let mut held = Vec::new();
        for id in &ids {
            held.push(self.flat_take(*id, 1, scope, output)?);
        }
        let held_count = held.iter().map(|value| value.words.len()).sum::<usize>();
        let arity = held_count
            + results
                .iter()
                .map(|layout| layout.words.len())
                .sum::<usize>();
        let result = result_layout(self.layouts.layout(&expression.ty)?);
        let id = self.reserve_segment(arity, result.words.len())?;
        let mut body = Body::new_segment();
        let mut inner = FlatScope::new();
        let mut offset = 0;
        for (variable, layout) in ids
            .iter()
            .copied()
            .zip(held.iter().map(|value| &value.layout))
            .chain(
                bindings
                    .iter()
                    .map(|binding| binding.binder.id)
                    .zip(&results),
            )
        {
            let mut words = Vec::new();
            for _ in &layout.words {
                words.push(self.hold(&mut body, &format!("captures[{offset}]"))?);
                offset += 1;
            }
            inner.insert(
                variable,
                Local {
                    value: Value {
                        layout: layout.clone(),
                        words,
                    },
                    remaining: uses[&variable],
                    owned: true,
                },
            );
        }
        self.flat_body(expression, Vec::new(), &mut inner, &mut body, &result)?;
        self.finish_segment(id, body);
        let (words, owned) = self.flat_arrays(&held, output)?;
        let children = self.array(output, &tasks)?;
        self.pending_words(
            output,
            &format!(
                "tb_c_word_join(e, {}, {held_count}, {words}, {owned}, {}, {children})",
                id + 2,
                tasks.len()
            ),
            &result,
            false,
        )
    }
}

/// Only native scalar layouts share their value bits with the unary ABI.
/// Finite constructor layouts need their ordinary boxing conversion even
/// when they happen to occupy one word.
fn legacy_tail_mode(layout: &Layout) -> Option<&'static str> {
    if layout.arms.is_some() {
        return None;
    }
    match layout.words.as_slice() {
        [Kind::Box] => Some("TB_RESULT_NONE"),
        [Kind::W32] => Some("TB_RESULT_RAW32"),
        [Kind::W64] => Some("TB_RESULT_RAW64"),
        _ => None,
    }
}

/// An actual unary invocation yields task control in tail position. Erased
/// applications of a variable merely expose that value and must not be read
/// as a task. Exact direct segment calls are handled before this boundary.
fn legacy_call(expression: &Expression) -> bool {
    let mut head = expression;
    let mut live = false;
    while let ExpressionKind::Apply {
        function, quant, ..
    } = &head.kind
    {
        live |= *quant != Quant::None;
        head = function;
    }
    live || matches!(head.kind, ExpressionKind::Definition(_))
}

/// Applying a whole raised telescope does not allocate the intermediate
/// unary closures, so their free-variable uses must not collapse to one capture.
fn invocation_uses(
    program: &ExecutableProgram,
    expression: &Expression,
    arguments: usize,
) -> BTreeMap<usize, usize> {
    match &expression.kind {
        ExpressionKind::Lambda { parameter, body }
            if parameter.quant == Quant::None || arguments != 0 =>
        {
            let mut uses = invocation_uses(
                program,
                body,
                arguments - usize::from(parameter.quant != Quant::None),
            );
            uses.remove(&parameter.id);
            uses
        }
        ExpressionKind::Match {
            parameter,
            fields,
            arm,
            fallback,
            ..
        } if arguments != 0 => {
            let count = fields
                .iter()
                .filter(|field| field.quant != Quant::None)
                .count();
            let mut uses = invocation_uses(program, arm, arguments - 1 + count);
            uses.extend(invocation_uses(program, fallback, arguments));
            uses.remove(&parameter.id);
            for count in uses.values_mut() {
                *count = 1;
            }
            uses
        }
        _ => runtime_uses(program, expression),
    }
}
