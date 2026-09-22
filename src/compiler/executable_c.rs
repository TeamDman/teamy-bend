// SPDX-License-Identifier: Apache-2.0
// Native C representation derived from Bend 2.0.5, Copyright 2026 HigherOrderCO.
// Rust translation and changes: TeamDman. See NOTICE and licenses/Apache-2.0.txt.
//! Checked executable lowering into the packed native C foreign ABI.

use super::CompileError;
use super::executable_c_foreign;
use super::executable_c_layout::ConstructorTable;
use super::executable_c_layout::Kind;
use super::executable_c_layout::Layout;
use super::executable_c_layout::Layouts;
use crate::kernel::Binder;
use crate::kernel::ExecutableBook;
use crate::kernel::ExecutableEntry;
use crate::kernel::Quant;
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::kernel::elaborate::DefinitionBody;
use crate::kernel::elaborate::ExecutableProgram;
use crate::kernel::elaborate::Expression;
use crate::kernel::elaborate::ExpressionKind;
use crate::kernel::elaborate::Field;
use crate::kernel::substitute;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Write;
use std::rc::Rc;

#[path = "executable_c_forks.rs"]
mod forks;
#[path = "executable_c_gpu.rs"]
mod gpu;
#[path = "executable_c_native.rs"]
mod native;
#[path = "executable_c_segments.rs"]
mod segments;
#[path = "executable_c_values.rs"]
mod values;

type Scope = BTreeMap<usize, OwnedLocal>;
type Substitutions = BTreeMap<usize, TermRef>;

#[derive(Clone)]
struct OwnedLocal {
    value: String,
    remaining: usize,
    owned: bool,
}

impl OwnedLocal {
    fn new(value: String, remaining: usize) -> Self {
        Self {
            value,
            remaining,
            owned: true,
        }
    }
}

/// Generate a standalone C program from an execution-only checked book.
///
/// Foreign source is embedded without executing it during compilation. Its
/// contracts remain runtime assumptions, separate from strict proof evidence.
/// The native ABI uses packed Terms, flattened constructor fields and unary
/// closures, with optional CUDA execution for marked calls.
///
/// # Errors
/// Rejects unsupported reachable contracts, invalid layouts, unavailable C
/// imports and exhausted compiler budgets before producing an output program.
pub fn compile_executable_c(book: &ExecutableBook) -> Result<String, CompileError> {
    let program = book
        .lower_for_compilation()
        .map_err(|error| CompileError::new(error.to_string()))?;
    let table = ConstructorTable::build(&program)?;
    let foreign = executable_c_foreign::assemble(&program, &table)?;
    let mut generator = Generator {
        program: &program,
        table,
        layouts: Layouts::new(&program),
        definitions: Vec::new(),
        definition_ids: BTreeMap::new(),
        definition_closures: Vec::new(),
        segment_ids: BTreeMap::new(),
        closures: Vec::new(),
        conversions: Vec::new(),
        conversion_ids: BTreeMap::new(),
        conversion_dependencies: Vec::new(),
        printers: Vec::new(),
        printer_ids: BTreeMap::new(),
        nodes: 0,
        current_origin: None,
        gpu_names: gpu::marked_names(&program),
    };
    let entry = if program.entry == ExecutableEntry::Missing {
        None
    } else {
        Some(generator.definition("main", &Substitutions::new())?)
    };
    let printer = if program.entry == ExecutableEntry::Pure {
        Some(generator.printer(&program.definitions["main"].ty, 0)?)
    } else {
        None
    };
    let gpu = generator.gpu_source(&foreign.requests)?;
    let mut source = generator.runtime_source(gpu.as_deref());
    source.push_str(&foreign.source);
    generator.prototypes(&mut source);
    for definition in &generator.definitions {
        source.push_str(definition);
    }
    for closure in &generator.closures {
        source.push_str(&closure.source);
    }
    for conversion in &generator.conversions {
        source.push_str(conversion);
    }
    for printer in &generator.printers {
        source.push_str(printer);
    }
    source.push_str("static void tb_initialize(Env e) {\n  (void)e;\n  tb_register_builtins();\n");
    generator.registrations(&mut source);
    for initializer in &foreign.initializers {
        writeln!(source, "  {initializer}();").unwrap();
    }
    for cid in foreign.requests.values() {
        writeln!(source, "  tb_require_effect({cid});").unwrap();
    }
    source.push_str("}\n");
    if let Some(printer) = printer {
        writeln!(source, "static void tb_show_main(Env e, Term value) {{ tb_show_{printer}(&e, value, 0, 0); tb_show_text(\"\\n\"); }}").unwrap();
    }
    let entry = entry.map_or_else(
        || "NULL".to_owned(),
        |id| {
            let fid = generator.definition_closures[id] + 2;
            writeln!(
                source,
                "static Term tb_entry(Env e) {{ return tb_apply(e, term_clo({fid}, 0), 0); }}"
            )
            .unwrap();
            "tb_entry".to_owned()
        },
    );
    writeln!(
        source,
        "static int tb_program_main(void) {{ return tb_run({entry}, {}, {}, tb_initialize); }}",
        usize::from(program.entry == ExecutableEntry::Io),
        if printer.is_some() {
            "tb_show_main"
        } else {
            "NULL"
        }
    )
    .unwrap();
    source.push_str(include_str!("executable_cli.c"));
    source.push_str("\n#ifndef TB_NO_MAIN\nint main(int argc, char **argv) { return tb_cli_main(argc, argv, tb_program_main); }\n#endif\n");
    Ok(source)
}

impl Generator<'_> {
    fn runtime_source(&self, device: Option<&str>) -> String {
        let mut source = String::from(
            "/* Generated by teamy-bend. Input imports retain their own licences. */\n#ifndef _WIN32\n#ifndef _POSIX_C_SOURCE\n#define _POSIX_C_SOURCE 200809L\n#endif\n#ifndef _XOPEN_SOURCE\n#define _XOPEN_SOURCE 700\n#endif\n#endif\n#ifdef _MSC_VER\n#define _CRT_SECURE_NO_WARNINGS\n#endif\n#include <stdint.h>\n",
        );
        source.push_str(&self.table.declarations());
        if device.is_some() {
            source.push_str("#define TB_GPU_ENABLED 1\n");
        }
        source.push_str(
            &include_str!("executable_core.c")
                .replace(
                    "/* TB_SHARED_VALUES */",
                    include_str!("executable_value_core.c"),
                )
                .replace(
                    "/* TB_SHARED_ARRAYS */",
                    include_str!("executable_array_core.c"),
                ),
        );
        if device.is_some() {
            self.gpu_marks(&mut source);
        }
        source.push_str(include_str!("executable_tasks.c"));
        if let Some(device) = device {
            source.push_str(include_str!("executable_device_state.h"));
            source.push_str(include_str!("executable_device_control.h"));
            source.push_str(&include_str!("executable_cuda.c").replace(
                "/* TB_CUDA_CACHE */",
                include_str!("executable_cuda_cache.c"),
            ));
            gpu::embed_source(device, &mut source);
            source.push_str(include_str!("executable_gpu.c"));
        }
        source.push_str(include_str!("executable_io.c"));
        source.push_str(include_str!("executable_files.c"));
        source.push_str(include_str!("executable_network.c"));
        source.push_str(include_str!("executable_c_numeric.c"));
        source.push_str(&include_str!("executable_c_bridge.c").replace(
            "/* TB_SHARED_BRIDGES */",
            include_str!("executable_value_bridge.c"),
        ));
        source
    }
}

struct Closure {
    source: String,
    /// Every emitted fid has its resume body here, including definition thunks
    /// whose host resume bodies are stored separately from their wrappers.
    resume_source: String,
    origin: Option<String>,
    dependencies: BodyDependencies,
    captures: usize,
    slots: usize,
    segment: Option<(usize, usize)>,
}

/// Edges refer to assigned closure/conversion indices, including references to
/// cached or recursively reserved bodies. No dependency is recovered from C text.
#[derive(Clone, Default)]
struct BodyDependencies {
    fids: BTreeSet<usize>,
    conversions: BTreeSet<usize>,
    host_operations: BTreeSet<String>,
    dynamic_calls: bool,
}

impl BodyDependencies {
    fn extend(&mut self, other: &Self) {
        self.fids.extend(&other.fids);
        self.conversions.extend(&other.conversions);
        self.host_operations
            .extend(other.host_operations.iter().cloned());
        self.dynamic_calls |= other.dynamic_calls;
    }
}

/// Generated values live in tracked heap frames, never in a C array whose size
/// grows with the input program. Application bodies can suspend and resume;
/// synchronous conversion and printer wrappers release scratch on every return.
struct Body {
    text: String,
    slots: usize,
    resumes: usize,
    words: bool,
    dependencies: BodyDependencies,
}

#[derive(Clone, Copy)]
enum FunctionResult {
    Term,
    Void,
}

impl Body {
    fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            slots: 0,
            resumes: 0,
            words: false,
            dependencies: BodyDependencies::default(),
        }
    }

    fn new_segment() -> Self {
        Self {
            words: true,
            ..Self::new("")
        }
    }

    fn reserve(&mut self, count: usize) -> usize {
        let start = self.slots;
        self.slots += count;
        start
    }

    fn push_str(&mut self, text: &str) {
        self.text.push_str(text);
    }

    /// Resume labels skip all preceding evaluation and ownership transfers.
    /// Scratch cells retain stale aliases as well as live values; the runtime
    /// frees this storage without treating every cell as an owned root.
    fn resumable(self, id: usize) -> String {
        let result_type = if self.words { "TBOutcome" } else { "Term" };
        let mut source = format!(
            "static TB_NOINLINE {result_type} tb_resume_{id}(const Env *e, TBCallFrame *tb_frame) {{\n  Term *tb_values = tb_frame->values;\n  const Term *captures = tb_frame->captures;\n  Term argument = tb_frame->argument;\n  (void)e; (void)tb_values; (void)captures; (void)argument;\n  switch (tb_frame->pc) {{\n  case 0: break;\n"
        );
        for pc in 1..=self.resumes {
            writeln!(source, "  case {pc}: goto tb_resume_{pc};").unwrap();
        }
        source.push_str("  default: err_fail(\"invalid generated continuation state\");\n  }\n");
        source.push_str(&self.text);
        source.push_str("}\n");
        source
    }

    fn function(
        self,
        name: &str,
        parameters: &str,
        arguments: &str,
        result: FunctionResult,
    ) -> String {
        assert_eq!(self.resumes, 0, "synchronous helper cannot suspend");
        let returns_term = matches!(result, FunctionResult::Term);
        let result_type = if returns_term { "Term" } else { "void" };
        let mut source = format!(
            "static TB_NOINLINE {result_type} {name}_body({parameters}, Term *tb_values) {{\n  (void)tb_values;\n{} }}\nstatic TB_NOINLINE {result_type} {name}({parameters}) {{\n  Term *tb_values = tb_frame_push({});\n",
            self.text, self.slots
        );
        if returns_term {
            writeln!(source, "  Term tb_result = {name}_body({arguments}, tb_values);\n  tb_frame_pop(tb_values);\n  return tb_result;\n}}").unwrap();
        } else {
            writeln!(
                source,
                "  {name}_body({arguments}, tb_values);\n  tb_frame_pop(tb_values);\n}}"
            )
            .unwrap();
        }
        source
    }
}

impl Write for Body {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.text.push_str(s);
        Ok(())
    }
}

struct Generator<'a> {
    program: &'a ExecutableProgram,
    table: ConstructorTable,
    layouts: Layouts<'a>,
    definitions: Vec<String>,
    definition_ids: BTreeMap<String, usize>,
    definition_closures: Vec<usize>,
    segment_ids: BTreeMap<String, Option<segments::Signature>>,
    closures: Vec<Closure>,
    conversions: Vec<String>,
    conversion_ids: BTreeMap<String, usize>,
    conversion_dependencies: Vec<BodyDependencies>,
    printers: Vec<String>,
    printer_ids: BTreeMap<String, usize>,
    nodes: usize,
    current_origin: Option<String>,
    gpu_names: BTreeSet<String>,
}

impl Generator<'_> {
    fn registrations(&self, source: &mut String) {
        for (index, closure) in self.closures.iter().enumerate() {
            if let Some((arity, result_width)) = closure.segment {
                writeln!(
                    source,
                    "  tb_register_segment({}, tb_resume_{index}, {arity}, {result_width}, {});",
                    index + 2,
                    closure.slots
                )
                .unwrap();
            } else {
                writeln!(
                    source,
                    "  tb_register_generated({}, tb_function_{index}, tb_resume_{index}, {}, {});",
                    index + 2,
                    closure.captures,
                    closure.slots
                )
                .unwrap();
            }
            writeln!(source, "  tb_register_parallel({});", index + 2).unwrap();
        }
    }

    fn prototypes(&self, source: &mut String) {
        for (index, closure) in self.closures.iter().enumerate() {
            if closure.segment.is_some() {
                writeln!(source, "static TB_NOINLINE TBOutcome tb_resume_{index}(const Env *e, TBCallFrame *tb_frame);").unwrap();
                continue;
            }
            writeln!(
                source,
                "static TB_NOINLINE Term tb_function_{index}(Env e, const Term *captures, Term argument);\nstatic TB_NOINLINE Term tb_resume_{index}(const Env *e, TBCallFrame *tb_frame);"
            )
            .unwrap();
        }
        for index in 0..self.conversions.len() {
            writeln!(source, "static TB_NOINLINE Term tb_box_{index}(const Env *e, const Term *input);\nstatic TB_NOINLINE void tb_unbox_{index}(const Env *e, Term value, Term *output);").unwrap();
        }
        for index in 0..self.printers.len() {
            writeln!(
                source,
                "static TB_NOINLINE void tb_show_{index}(const Env *e, Term value, unsigned depth, char chain);"
            )
            .unwrap();
        }
    }

    fn fresh(&mut self) -> Result<String, CompileError> {
        self.nodes += 1;
        if self.nodes > 250_000 {
            return Err(CompileError::new(
                "executable C generation budget exhausted",
            ));
        }
        Ok(format!("t{}", self.nodes))
    }

    fn hold(&mut self, output: &mut Body, expression: &str) -> Result<String, CompileError> {
        self.fresh()?;
        let name = format!("tb_values[{}]", output.reserve(1));
        writeln!(output, "  {name} = {expression};").unwrap();
        Ok(name)
    }

    fn array(&mut self, output: &mut Body, values: &[String]) -> Result<String, CompileError> {
        self.fresh()?;
        let start = output.reserve(values.len().max(1));
        if values.is_empty() {
            writeln!(output, "  tb_values[{start}] = 0;").unwrap();
        } else {
            for (index, value) in values.iter().enumerate() {
                writeln!(output, "  tb_values[{}] = {value};", start + index).unwrap();
            }
        }
        Ok(format!("(tb_values + {start})"))
    }

    fn owned_use(
        &mut self,
        id: usize,
        scope: &mut Scope,
        output: &mut Body,
    ) -> Result<String, CompileError> {
        let local = scope
            .get_mut(&id)
            .ok_or_else(|| CompileError::new(format!("unbound executable C variable {id}")))?;
        if !local.owned || local.remaining == 0 {
            return Err(CompileError::new(format!(
                "unbalanced executable C ownership for variable {id}"
            )));
        }
        local.remaining -= 1;
        if local.remaining == 0 {
            local.owned = false;
            Ok(local.value.clone())
        } else {
            self.hold(output, &format!("tb_c_duplicate(e, &{})", local.value))
        }
    }

    fn definition(
        &mut self,
        name: &str,
        substitutions: &Substitutions,
    ) -> Result<usize, CompileError> {
        let key = format!("{name}:{substitutions:?}");
        if let Some(index) = self.definition_ids.get(&key) {
            return Ok(*index);
        }
        if self.definitions.len() >= 4096 {
            return Err(CompileError::new(
                "executable C specialization budget exhausted",
            ));
        }
        let previous_origin = self.current_origin.replace(name.to_owned());
        let index = self.definitions.len();
        self.definitions.push(String::new());
        self.definition_ids.insert(key, index);
        // Reserve the thunk before traversing its body: recursive references
        // use the same dispatcher entry rather than recursing through C thunks.
        let closure = self.reserve_closure(0)?;
        self.definition_closures.push(closure);
        self.closures[closure].source = closure_wrapper(closure, 0);
        let definition = self
            .program
            .definitions
            .get(name)
            .ok_or_else(|| CompileError::new(format!("missing executable definition {name}")))?
            .clone();
        let mut output = Body::new("  tb_c_drop(e, argument);\n  tb_tick();\n");
        let live = definition
            .parameters
            .iter()
            .filter(|parameter| parameter.quant != Quant::None)
            .count();
        let value = match definition.body {
            DefinitionBody::OpaqueType(_) => "0".to_owned(),
            DefinitionBody::Foreign(_) => {
                let cid = self.table.get(name)?.cid;
                self.curry(live + 1, &mut output, &mut |generator, arguments, body| {
                    // This constructs an IO request. Its foreign effect handler
                    // executes later on the host and is never device source.
                    let fields = generator.array(body, arguments)?;
                    generator.hold(
                        body,
                        &format!(
                            "tb_c_construct(e, {cid}, {}, {fields}, false, NULL)",
                            arguments.len()
                        ),
                    )
                })?
            }
            DefinitionBody::Numeric(intrinsic) => {
                self.curry(live, &mut output, &mut |generator, arguments, body| {
                    generator.numeric(intrinsic, arguments, body)
                })?
            }
            DefinitionBody::Ordinary(body) => {
                let ty = specialize_type(&definition.ty, substitutions);
                if self.program.base_names.contains(name) && native::optimized(name) {
                    self.curry(live, &mut output, &mut |generator, arguments, body| {
                        generator.native(name, &ty, arguments, body)
                    })?
                } else if self.gpu_names.contains(name)
                    && let Some(signature) = self.segment(name, substitutions)?
                {
                    // Partial ordinary calls retain a closure until their full
                    // live telescope is supplied. That closure then enters the
                    // same marked segment as a statically saturated call.
                    self.curry(
                        signature.arguments.len(),
                        &mut output,
                        &mut |generator, arguments, body| {
                            generator.segment_boundary(&signature, arguments, body, true)
                        },
                    )?
                } else if forks::required_arguments(self.program, name) == Some(0)
                    && let Some(signature) = self.segment(name, substitutions)?
                    && signature.arguments.is_empty()
                {
                    self.segment_boundary(&signature, &[], &mut output, true)?
                } else {
                    self.expression(
                        &specialize_expression(&body, substitutions),
                        &mut Scope::new(),
                        &mut output,
                        true,
                    )?
                }
            }
        };
        writeln!(output, "  return {value};").unwrap();
        self.closures[closure].slots = output.slots;
        self.closures[closure].dependencies = output.dependencies.clone();
        self.definitions[index] = output.resumable(closure);
        self.closures[closure]
            .resume_source
            .clone_from(&self.definitions[index]);
        self.current_origin = previous_origin;
        Ok(index)
    }

    fn reference(
        &mut self,
        id: usize,
        output: &mut Body,
        tail: bool,
    ) -> Result<String, CompileError> {
        let closure = self.definition_closures[id];
        output.dependencies.fids.insert(closure);
        let fid = closure + 2;
        self.known_apply(output, &format!("term_clo({fid}, 0)"), "0", tail)
    }

    /// Box only where a word-segment call returns to the unary closure ABI.
    /// The segment's own body and recursive calls keep their word vectors.
    fn segment_boundary(
        &mut self,
        signature: &segments::Signature,
        arguments: &[String],
        output: &mut Body,
        tail: bool,
    ) -> Result<String, CompileError> {
        output.dependencies.fids.insert(signature.id);
        let (words, owned) = self.segment_arguments(signature, arguments, output)?;
        let arity: usize = signature
            .arguments
            .iter()
            .map(|layout| layout.words.len())
            .sum();
        let task = format!(
            "tb_c_word_task(e, {}, {arity}, {words}, {owned})",
            signature.id + 2
        );
        if tail && signature.result.arms.is_none() && signature.result.words.len() == 1 {
            let mode = match signature.result.words[0] {
                Kind::Box => None,
                Kind::W32 => Some("TB_RESULT_BOX32"),
                Kind::W64 => Some("TB_RESULT_BOX64"),
            };
            return self.hold(
                output,
                &mode.map_or(task.clone(), |mode| {
                    format!("tb_tail_result(tb_frame, {task}, {mode})")
                }),
            );
        }
        let value = self.pending_words(output, &task, &signature.result, false)?;
        let result = self.array(output, &value.words)?;
        self.segment_result(signature, &result, output)
    }

    fn apply(
        &mut self,
        output: &mut Body,
        function: &str,
        argument: &str,
        tail: bool,
    ) -> Result<String, CompileError> {
        output.dependencies.dynamic_calls = true;
        self.known_apply(output, function, argument, tail)
    }

    fn known_apply(
        &mut self,
        output: &mut Body,
        function: &str,
        argument: &str,
        tail: bool,
    ) -> Result<String, CompileError> {
        self.pending(
            output,
            &format!("tb_c_tail_apply(e, {function}, {argument})"),
            tail,
        )
    }

    fn pending(
        &mut self,
        output: &mut Body,
        task: &str,
        tail: bool,
    ) -> Result<String, CompileError> {
        if tail {
            return self.hold(output, task);
        }
        self.fresh()?;
        let slot = output.reserve(1);
        output.resumes += 1;
        let pc = output.resumes;
        let returned = if output.words {
            format!("tb_segment_task({task})")
        } else {
            task.to_owned()
        };
        writeln!(output, "  tb_frame->pc = {pc}; tb_frame->destination = {slot}; tb_frame->expected = 1; tb_frame->waiting = true;\n  return {returned};\ntb_resume_{pc}: ;").unwrap();
        Ok(format!("tb_values[{slot}]"))
    }

    fn instantiation(&self, name: &str, arguments: &[(&Expression, Quant)]) -> Substitutions {
        let definition = &self.program.definitions[name];
        let mut substitutions = Substitutions::new();
        if let DefinitionBody::Ordinary(body) = &definition.body {
            // The declared telescope excludes binders of a returned closure.
            // Pair the actual typed lambdas with the complete application spine;
            // source lambda IDs need not equal the IDs in their type annotations.
            let mut body = body;
            for (argument, quant) in arguments {
                // A returned lambda may close over preceding let bindings.
                // Inspect its binder without moving or duplicating those RHSs.
                while let ExpressionKind::Let { body: next, .. } = &body.kind {
                    body = next;
                }
                let ExpressionKind::Lambda {
                    parameter,
                    body: next,
                } = &body.kind
                else {
                    break;
                };
                if parameter.quant == Quant::None && *quant == Quant::None {
                    substitutions.insert(parameter.id, Rc::clone(&argument.source));
                }
                body = next;
            }
        } else {
            for (parameter, (argument, quant)) in definition.parameters.iter().zip(arguments) {
                if parameter.quant == Quant::None && *quant == Quant::None {
                    substitutions.insert(parameter.id, Rc::clone(&argument.source));
                }
            }
        }
        substitutions
    }

    fn curry<F>(
        &mut self,
        arity: usize,
        output: &mut Body,
        final_body: &mut F,
    ) -> Result<String, CompileError>
    where
        F: FnMut(&mut Self, &[String], &mut Body) -> Result<String, CompileError>,
    {
        self.curry_at(arity, &[], output, final_body)
    }

    fn curry_at<F>(
        &mut self,
        arity: usize,
        arguments: &[String],
        output: &mut Body,
        final_body: &mut F,
    ) -> Result<String, CompileError>
    where
        F: FnMut(&mut Self, &[String], &mut Body) -> Result<String, CompileError>,
    {
        if arguments.len() == arity {
            return final_body(self, arguments, output);
        }
        let id = self.reserve_closure(arguments.len())?;
        let mut body = Body::new("  (void)e; (void)captures; (void)argument;\n");
        let mut next = (0..arguments.len())
            .map(|index| format!("captures[{index}]"))
            .collect::<Vec<_>>();
        next.push("argument".to_owned());
        let value = self.curry_at(arity, &next, &mut body, final_body)?;
        writeln!(body, "  return {value};").unwrap();
        self.finish_closure(id, body);
        self.make_closure(id, arguments, output)
    }

    fn reserve_closure(&mut self, captures: usize) -> Result<usize, CompileError> {
        if self.closures.len() >= 65534 || captures > 254 {
            return Err(CompileError::new(
                "executable C closure table or capture arity exhausted",
            ));
        }
        let id = self.closures.len();
        self.closures.push(Closure {
            source: String::new(),
            resume_source: String::new(),
            origin: self.current_origin.clone(),
            dependencies: BodyDependencies::default(),
            captures,
            slots: 0,
            segment: None,
        });
        Ok(id)
    }

    fn reserve_segment(
        &mut self,
        arity: usize,
        result_width: usize,
    ) -> Result<usize, CompileError> {
        if arity > 255 || !(1..=255).contains(&result_width) {
            return Err(CompileError::new("executable C segment arity exhausted"));
        }
        let id = self.reserve_closure(0)?;
        self.closures[id].segment = Some((arity, result_width));
        Ok(id)
    }

    fn finish_segment(&mut self, id: usize, body: Body) {
        self.closures[id].slots = body.slots;
        self.closures[id].dependencies = body.dependencies.clone();
        self.closures[id].source = body.resumable(id);
        let closure = &mut self.closures[id];
        closure.resume_source.clone_from(&closure.source);
    }

    fn finish_closure(&mut self, id: usize, body: Body) {
        self.closures[id].slots = body.slots;
        self.closures[id].dependencies = body.dependencies.clone();
        let mut source = body.resumable(id);
        self.closures[id].resume_source.clone_from(&source);
        source.push_str(&closure_wrapper(id, self.closures[id].captures));
        self.closures[id].source = source;
    }

    fn make_closure(
        &mut self,
        id: usize,
        captures: &[String],
        output: &mut Body,
    ) -> Result<String, CompileError> {
        output.dependencies.fids.insert(id);
        let array = self.array(output, captures)?;
        self.hold(
            output,
            &format!("tb_c_closure(e, {}, {}, {array})", id + 2, captures.len()),
        )
    }

    fn closure(
        &mut self,
        expression: &Expression,
        parameter: usize,
        scope: &mut Scope,
        output: &mut Body,
    ) -> Result<String, CompileError> {
        let used = runtime_uses(self.program, expression);
        let captures = scope
            .iter()
            .filter(|(id, _)| **id != parameter && used.contains_key(id))
            .map(|(id, _)| *id)
            .collect::<Vec<_>>();
        let id = self.reserve_closure(captures.len())?;
        let mut inner = Scope::new();
        let mut body = Body::new("  (void)e; (void)captures; (void)argument;\n");
        let body_uses = match &expression.kind {
            ExpressionKind::Lambda { body, .. } => runtime_uses(self.program, body),
            _ => BTreeMap::new(),
        };
        for (index, variable) in captures.iter().enumerate() {
            let value = self.hold(&mut body, &format!("captures[{index}]"))?;
            inner.insert(
                *variable,
                OwnedLocal::new(value, body_uses.get(variable).copied().unwrap_or(0)),
            );
        }
        let argument = self.hold(&mut body, "argument")?;
        inner.insert(
            parameter,
            OwnedLocal::new(argument, body_uses.get(&parameter).copied().unwrap_or(0)),
        );
        if matches!(expression.kind, ExpressionKind::Lambda { .. }) {
            drop_unused(&mut inner, &mut body);
        }
        let value = match &expression.kind {
            ExpressionKind::Lambda {
                body: expression, ..
            } => self.expression(expression, &mut inner, &mut body, true)?,
            ExpressionKind::Match { .. } | ExpressionKind::Absurd { .. } => {
                self.match_body(expression, &mut inner, &mut body)?
            }
            _ => return Err(CompileError::new("invalid executable C closure node")),
        };
        drop_owned(&mut inner, &mut body);
        writeln!(body, "  return {value};").unwrap();
        self.finish_closure(id, body);
        let values = captures
            .iter()
            .map(|variable| self.owned_use(*variable, scope, output))
            .collect::<Result<Vec<_>, _>>()?;
        self.make_closure(id, &values, output)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "The complete typed IR traversal keeps argument erasure, specialization and evaluation order together."
    )]
    fn expression(
        &mut self,
        expression: &Expression,
        scope: &mut Scope,
        output: &mut Body,
        tail: bool,
    ) -> Result<String, CompileError> {
        self.fresh()?;
        match &expression.kind {
            ExpressionKind::Erased => Ok("0".to_owned()),
            ExpressionKind::Variable(id) => self.owned_use(*id, scope, output),
            ExpressionKind::Definition(name) => {
                if forks::required_arguments(self.program, name) == Some(0)
                    && let Some(signature) = self.segment(name, &Substitutions::new())?
                    && signature.arguments.is_empty()
                {
                    return self.segment_boundary(&signature, &[], output, tail);
                }
                let id = self.definition(name, &Substitutions::new())?;
                self.reference(id, output, tail)
            }
            ExpressionKind::Lambda { parameter, body } if parameter.quant == Quant::None => {
                self.expression(body, scope, output, tail)
            }
            ExpressionKind::Lambda { parameter, .. }
            | ExpressionKind::Match { parameter, .. }
            | ExpressionKind::Absurd { parameter, .. } => {
                self.closure(expression, parameter.id, scope, output)
            }
            ExpressionKind::Apply { .. } => {
                let mut arguments = Vec::new();
                let mut head = expression;
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
                let mut remaining = arguments.iter().filter(|(_, q)| *q != Quant::None).count();
                let mut function = if let ExpressionKind::Definition(name) = &head.kind {
                    if let Some(value) = self.direct_native(name, &arguments, scope, output)? {
                        return Ok(value);
                    }
                    let substitutions = self.instantiation(name, &arguments);
                    if forks::required_arguments(self.program, name)
                        .is_some_and(|required| remaining >= required)
                        && let Some(signature) = self.segment(name, &substitutions)?
                        && remaining >= signature.arguments.len()
                    {
                        let mut values = Vec::with_capacity(signature.arguments.len());
                        let mut live = arguments.iter().filter(|(_, quant)| *quant != Quant::None);
                        for (argument, _) in live.by_ref().take(signature.arguments.len()) {
                            values.push(self.expression(argument, scope, output, false)?);
                        }
                        let mut value = self.segment_boundary(
                            &signature,
                            &values,
                            output,
                            tail && remaining == values.len(),
                        )?;
                        remaining -= values.len();
                        for (argument, _) in live {
                            let argument = self.expression(argument, scope, output, false)?;
                            remaining -= 1;
                            value =
                                self.apply(output, &value, &argument, tail && remaining == 0)?;
                        }
                        return Ok(value);
                    }
                    let id = self.definition(name, &substitutions)?;
                    self.reference(id, output, tail && remaining == 0)?
                } else {
                    self.expression(head, scope, output, tail && remaining == 0)?
                };
                for (argument, quant) in arguments {
                    if quant == Quant::None {
                        continue;
                    }
                    let argument = self.expression(argument, scope, output, false)?;
                    remaining -= 1;
                    function = self.apply(output, &function, &argument, tail && remaining == 0)?;
                }
                Ok(function)
            }
            ExpressionKind::Constructor {
                owner,
                name,
                fields,
            } => {
                if self.program.base_names.contains(owner)
                    && matches!(owner.as_str(), "U32" | "F32")
                    && fields.len() == 1
                    && let Some(word) = word_literal(self.program, &fields[0].value)
                {
                    return Ok(format!("UINT64_C({word})"));
                }
                let values = fields
                    .iter()
                    .filter(|field| field.binder.quant != Quant::None)
                    .map(|field| self.expression(&field.value, scope, output, false))
                    .collect::<Result<Vec<_>, _>>()?;
                if self.program.base_names.contains(owner) {
                    match owner.as_str() {
                        "Nat" => {
                            return if values.is_empty() {
                                Ok("0".into())
                            } else {
                                self.hold(output, &format!("tb_c_nat_chk(e, {} + 1)", values[0]))
                            };
                        }
                        "U32" | "F32" => {
                            return self.hold(output, &format!("tb_c_term_word(e, {})", values[0]));
                        }
                        "Array" => {
                            return self.array_constructor(name, &expression.ty, &values, output);
                        }
                        _ => (),
                    }
                }
                self.construct(name, &values, output)
            }
            ExpressionKind::Let { bindings, body } => {
                let uses = runtime_uses(self.program, body);
                let bindings = live_bindings(bindings, &uses);
                if fork_group(self.program, &bindings) {
                    return self.fork(&bindings, body, scope, output, tail);
                }
                let mut locals = Vec::new();
                for binding in bindings {
                    let value = self.expression(&binding.value, scope, output, false)?;
                    let value = self.hold(output, &value)?;
                    locals.push((binding.binder.id, value));
                }
                for (id, value) in &locals {
                    scope.insert(
                        *id,
                        OwnedLocal::new(value.clone(), uses.get(id).copied().unwrap_or(0)),
                    );
                }
                drop_unused(scope, output);
                let result = self.expression(body, scope, output, tail)?;
                for (id, _) in locals {
                    if let Some(local) = scope.remove(&id)
                        && local.owned
                    {
                        writeln!(output, "  tb_c_drop(e, {});", local.value).unwrap();
                    }
                }
                Ok(result)
            }
        }
    }

    /// Prepare every operand before publishing a child. Only the last live
    /// application becomes work in the fork; nested calls keep their strict order.
    fn fork_application(
        &mut self,
        expression: &Expression,
        scope: &mut Scope,
        output: &mut Body,
    ) -> Result<(String, String), CompileError> {
        let mut arguments = Vec::new();
        let mut head = expression;
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
        let mut remaining = arguments
            .iter()
            .filter(|(_, quant)| *quant != Quant::None)
            .count();
        let mut function = if let ExpressionKind::Definition(name) = &head.kind {
            let substitutions = self.instantiation(name, &arguments);
            let id = self.definition(name, &substitutions)?;
            if remaining == 0 {
                output
                    .dependencies
                    .fids
                    .insert(self.definition_closures[id]);
                return Ok((
                    format!("term_clo({}, 0)", self.definition_closures[id] + 2),
                    "0".into(),
                ));
            }
            self.reference(id, output, false)?
        } else {
            self.expression(head, scope, output, false)?
        };
        for (argument, quant) in arguments {
            if quant == Quant::None {
                continue;
            }
            let argument = self.expression(argument, scope, output, false)?;
            remaining -= 1;
            if remaining == 0 {
                output.dependencies.dynamic_calls = true;
                return Ok((function, argument));
            }
            function = self.apply(output, &function, &argument, false)?;
        }
        Err(CompileError::new("fork child has no callable application"))
    }

    fn fork(
        &mut self,
        bindings: &[&Field],
        expression: &Expression,
        scope: &mut Scope,
        output: &mut Body,
        tail: bool,
    ) -> Result<String, CompileError> {
        let mut children = Vec::with_capacity(bindings.len() * 2);
        for binding in bindings {
            let (function, argument) = self.fork_application(&binding.value, scope, output)?;
            children.extend([function, argument]);
        }
        let uses = runtime_uses(self.program, expression);
        let captures = scope
            .keys()
            .filter(|id| uses.contains_key(id))
            .copied()
            .collect::<Vec<_>>();
        let id = self.reserve_closure(captures.len() + bindings.len())?;
        output.dependencies.fids.insert(id);
        let mut inner = Scope::new();
        let mut body = Body::new("  tb_c_drop(e, argument);\n");
        for (index, variable) in captures
            .iter()
            .copied()
            .chain(bindings.iter().map(|binding| binding.binder.id))
            .enumerate()
        {
            let value = self.hold(&mut body, &format!("captures[{index}]"))?;
            inner.insert(variable, OwnedLocal::new(value, uses[&variable]));
        }
        let value = self.expression(expression, &mut inner, &mut body, true)?;
        drop_owned(&mut inner, &mut body);
        writeln!(body, "  return {value};").unwrap();
        self.finish_closure(id, body);
        let held = captures
            .iter()
            .map(|variable| self.owned_use(*variable, scope, output))
            .collect::<Result<Vec<_>, _>>()?;
        let held_array = self.array(output, &held)?;
        let children_array = self.array(output, &children)?;
        self.pending(
            output,
            &format!(
                "tb_c_join(e, {}, {}, {held_array}, {}, {children_array})",
                id + 2,
                held.len(),
                bindings.len()
            ),
            tail,
        )
    }

    fn match_body(
        &mut self,
        expression: &Expression,
        scope: &mut Scope,
        output: &mut Body,
    ) -> Result<String, CompileError> {
        let ExpressionKind::Match {
            parameter,
            owner,
            constructor,
            fields,
            arm,
            fallback,
        } = &expression.kind
        else {
            return Ok("tb_impossible()".into());
        };
        if parameter.quant == Quant::None {
            return Err(CompileError::new(
                "executable C cannot match an erased scrutinee",
            ));
        }
        let value = scope[&parameter.id].value.clone();
        writeln!(output, "  tb_reject_request({value});").unwrap();
        let condition = if self.program.base_names.contains(owner) {
            match owner.as_str() {
                "Nat" => format!(
                    "{value} {} 0",
                    if constructor == "Zero" { "==" } else { "!=" }
                ),
                "U32" | "F32" => "1".into(),
                "Array" => {
                    let (_, lgs, _) = self.array_layout(&parameter.ty)?;
                    format!(
                        "blk_cls({value}) {} {lgs}",
                        if constructor == "ALeaf" { "==" } else { "!=" }
                    )
                }
                _ => format!("term_aux({value}) == {}", self.table.get(constructor)?.cid),
            }
        } else {
            format!("term_aux({value}) == {}", self.table.get(constructor)?.cid)
        };
        let result = self.hold(output, "0")?;
        if condition != "1" {
            writeln!(output, "  if ({condition}) {{").unwrap();
        }
        let mut arm_scope = branch_scope(self.program, scope, arm, parameter.id);
        drop_unused(&mut arm_scope, output);
        let arm_value = self.owned_use(parameter.id, &mut arm_scope, output)?;
        let values = if self.program.base_names.contains(owner) {
            match owner.as_str() {
                "Nat" => {
                    if constructor == "Zero" {
                        Vec::new()
                    } else {
                        vec![format!("({arm_value} - 1)")]
                    }
                }
                "U32" | "F32" => {
                    vec![self.hold(output, &format!("tb_c_word(e, (u32){arm_value})"))?]
                }
                "Array" => self.array_fields(constructor, &parameter.ty, &arm_value, output)?,
                _ => self.fields(constructor, &arm_value, output)?,
            }
        } else {
            self.fields(constructor, &arm_value, output)?
        };
        if values.len()
            != fields
                .iter()
                .filter(|field| field.quant != Quant::None)
                .count()
        {
            return Err(CompileError::new(
                "C match field arity differs from its layout",
            ));
        }
        let mut branch = self.expression(arm, &mut arm_scope, output, values.is_empty())?;
        let count = values.len();
        for (index, field) in values.into_iter().enumerate() {
            branch = self.apply(output, &branch, &field, index + 1 == count)?;
        }
        drop_owned(&mut arm_scope, output);
        if condition == "1" {
            disown_scope(scope);
            return Ok(branch);
        }
        writeln!(output, "  {result} = {branch};\n  }} else {{").unwrap();
        let mut fallback_scope = branch_scope(self.program, scope, fallback, parameter.id);
        drop_unused(&mut fallback_scope, output);
        let branch = self.expression(fallback, &mut fallback_scope, output, false)?;
        let fallback_value = self.owned_use(parameter.id, &mut fallback_scope, output)?;
        output.dependencies.dynamic_calls = true;
        writeln!(
            output,
            "  {result} = tb_c_tail_apply(e, {branch}, {fallback_value});"
        )
        .unwrap();
        drop_owned(&mut fallback_scope, output);
        output.push_str("  }\n");
        disown_scope(scope);
        Ok(result)
    }
}

/// Each emitted branch has transferred or released its source owners.
fn disown_scope(scope: &mut Scope) {
    for local in scope.values_mut() {
        local.owned = false;
    }
}

fn closure_wrapper(id: usize, captures: usize) -> String {
    // Keep the foreign callback ABI callable. The dispatcher recognizes the
    // generated entry and runs its resume handler directly, without this wrapper.
    format!(
        "static TB_NOINLINE Term tb_function_{id}(Env e, const Term *captures, Term argument) {{\n  return tb_apply(e, tb_closure(e, {}, {captures}, captures), argument);\n}}\n",
        id + 2
    )
}

/// Count owned uses in one generated function. Creating a closure consumes one
/// capture per free variable; uses inside that closure belong to its own body.
/// Match branches are counted independently when their function is generated.
fn runtime_uses(program: &ExecutableProgram, expression: &Expression) -> BTreeMap<usize, usize> {
    fn merge(target: &mut BTreeMap<usize, usize>, source: BTreeMap<usize, usize>) {
        for (id, count) in source {
            *target.entry(id).or_default() += count;
        }
    }
    let mut output = BTreeMap::new();
    match &expression.kind {
        ExpressionKind::Variable(id) => {
            output.insert(*id, 1);
        }
        ExpressionKind::Lambda { parameter, body } => {
            output = runtime_uses(program, body);
            output.remove(&parameter.id);
            if parameter.quant != Quant::None {
                for count in output.values_mut() {
                    *count = 1;
                }
            }
        }
        ExpressionKind::Apply {
            function,
            argument,
            quant,
        } => {
            output = runtime_uses(program, function);
            if *quant != Quant::None {
                merge(&mut output, runtime_uses(program, argument));
            }
        }
        ExpressionKind::Constructor { fields, .. } => {
            for field in fields
                .iter()
                .filter(|field| field.binder.quant != Quant::None)
            {
                merge(&mut output, runtime_uses(program, &field.value));
            }
        }
        ExpressionKind::Match {
            parameter,
            arm,
            fallback,
            ..
        } => {
            output = runtime_uses(program, arm);
            merge(&mut output, runtime_uses(program, fallback));
            output.remove(&parameter.id);
            for count in output.values_mut() {
                *count = 1;
            }
        }
        ExpressionKind::Let { bindings, body } => {
            output = runtime_uses(program, body);
            let live = live_bindings(bindings, &output);
            for binding in bindings {
                output.remove(&binding.binder.id);
            }
            if fork_group(program, &live) {
                // The join captures each outer owner once; its body performs
                // any internal duplication after the child results arrive.
                for count in output.values_mut() {
                    *count = 1;
                }
            }
            for binding in live {
                merge(&mut output, runtime_uses(program, &binding.value));
            }
        }
        ExpressionKind::Erased | ExpressionKind::Definition(_) | ExpressionKind::Absurd { .. } => {}
    }
    output
}

fn live_bindings<'a>(bindings: &'a [Field], uses: &BTreeMap<usize, usize>) -> Vec<&'a Field> {
    bindings
        .iter()
        .filter(|binding| {
            binding.binder.quant != Quant::None && uses.contains_key(&binding.binder.id)
        })
        .collect()
}

fn fork_group(program: &ExecutableProgram, bindings: &[&Field]) -> bool {
    bindings.len() > 1
        && bindings
            .iter()
            .all(|binding| forks::is_call(program, &binding.value))
}

fn branch_scope(
    program: &ExecutableProgram,
    scope: &Scope,
    expression: &Expression,
    parameter: usize,
) -> Scope {
    let mut uses = runtime_uses(program, expression);
    *uses.entry(parameter).or_default() += 1;
    scope
        .iter()
        .map(|(id, local)| {
            let mut local = local.clone();
            local.remaining = uses.get(id).copied().unwrap_or(0);
            (*id, local)
        })
        .collect()
}

fn drop_owned(scope: &mut Scope, output: &mut Body) {
    for local in scope.values_mut().filter(|local| local.owned) {
        writeln!(output, "  tb_c_drop(e, {});", local.value).unwrap();
        local.owned = false;
    }
}

fn drop_unused(scope: &mut Scope, output: &mut Body) {
    for local in scope
        .values_mut()
        .filter(|local| local.owned && local.remaining == 0)
    {
        writeln!(output, "  tb_c_drop(e, {});", local.value).unwrap();
        local.owned = false;
    }
}

fn replace_type(ty: &TermRef, substitutions: &Substitutions) -> TermRef {
    substitutions
        .iter()
        .fold(Rc::clone(ty), |ty, (id, value)| substitute(&ty, *id, value))
}

fn specialize_type(ty: &TermRef, substitutions: &Substitutions) -> TermRef {
    if let Term::All { id, body, .. } = ty.as_ref()
        && let Some(argument) = substitutions.get(id)
    {
        return specialize_type(&substitute(body, *id, argument), substitutions);
    }
    replace_type(ty, substitutions)
}

fn specialize_binder(binder: &Binder, substitutions: &Substitutions) -> Binder {
    let mut result = binder.clone();
    result.ty = replace_type(&binder.ty, substitutions);
    result
}

fn specialize_expression(expression: &Expression, substitutions: &Substitutions) -> Expression {
    specialize_expression_shape(expression, substitutions, false)
}

fn specialize_segment_expression(
    expression: &Expression,
    substitutions: &Substitutions,
) -> Expression {
    specialize_expression_shape(expression, substitutions, true)
}

fn specialize_expression_shape(
    expression: &Expression,
    substitutions: &Substitutions,
    preserve_erased: bool,
) -> Expression {
    if !preserve_erased
        && let ExpressionKind::Lambda { parameter, body } = &expression.kind
        && parameter.quant == Quant::None
        && substitutions.contains_key(&parameter.id)
    {
        return specialize_expression_shape(body, substitutions, preserve_erased);
    }
    let mut result = expression.clone();
    result.ty = if preserve_erased {
        replace_type(&result.ty, substitutions)
    } else {
        specialize_type(&result.ty, substitutions)
    };
    result.source = replace_type(&result.source, substitutions);
    match &mut result.kind {
        ExpressionKind::Lambda { parameter, body } => {
            *parameter = specialize_binder(parameter, substitutions);
            **body = specialize_expression_shape(body, substitutions, preserve_erased);
            // Lowering retains the original annotated telescope on `ty`, whose
            // binder IDs can differ from the source lambda. Rebuild it from the
            // actual binder and specialized body, retaining all live lambdas.
            result.ty = Rc::new(Term::All {
                quant: parameter.quant,
                name: parameter.name.clone(),
                id: parameter.id,
                domain: Rc::clone(&parameter.ty),
                body: Rc::clone(&body.ty),
            });
        }
        ExpressionKind::Apply {
            function, argument, ..
        } => {
            **function = specialize_expression_shape(function, substitutions, preserve_erased);
            **argument = specialize_expression_shape(argument, substitutions, preserve_erased);
        }
        ExpressionKind::Constructor { fields, .. } => {
            for field in fields {
                field.binder = specialize_binder(&field.binder, substitutions);
                field.value =
                    specialize_expression_shape(&field.value, substitutions, preserve_erased);
            }
        }
        ExpressionKind::Match {
            parameter,
            fields,
            arm,
            fallback,
            ..
        } => {
            *parameter = specialize_binder(parameter, substitutions);
            for field in fields {
                *field = specialize_binder(field, substitutions);
            }
            **arm = specialize_expression_shape(arm, substitutions, preserve_erased);
            **fallback = specialize_expression_shape(fallback, substitutions, preserve_erased);
        }
        ExpressionKind::Absurd { parameter, .. } => {
            *parameter = specialize_binder(parameter, substitutions);
        }
        ExpressionKind::Let { bindings, body } => {
            let mut inner = substitutions.clone();
            for field in bindings {
                field.binder = specialize_binder(&field.binder, substitutions);
                field.value =
                    specialize_expression_shape(&field.value, substitutions, preserve_erased);
                // Let right-hand sides see only the outer scope. Install their
                // erased aliases together for the body, never in a sibling RHS.
                if field.binder.quant == Quant::None
                    || matches!(field.value.kind, ExpressionKind::Erased)
                {
                    inner.insert(field.binder.id, Rc::clone(&field.value.source));
                }
            }
            **body = specialize_expression_shape(body, &inner, preserve_erased);
            result.ty = Rc::clone(&body.ty);
        }
        ExpressionKind::Erased | ExpressionKind::Variable(_) | ExpressionKind::Definition(_) => (),
    }
    result
}

fn word_literal(program: &ExecutableProgram, mut expression: &Expression) -> Option<u32> {
    let mut word = 0;
    for bit in 0..32 {
        let ExpressionKind::Constructor {
            owner,
            name,
            fields,
        } = &expression.kind
        else {
            return None;
        };
        if owner != "Word.Con"
            || name != "WCon"
            || fields.len() != 2
            || !program.base_names.contains(owner)
        {
            return None;
        }
        let ExpressionKind::Constructor {
            owner,
            name,
            fields: bits,
        } = &fields[0].value.kind
        else {
            return None;
        };
        if owner != "Bool" || !program.base_names.contains(owner) || !bits.is_empty() {
            return None;
        }
        match name.as_str() {
            "True" => word |= 1 << bit,
            "False" => (),
            _ => return None,
        }
        expression = &fields[1].value;
    }
    matches!(&expression.kind, ExpressionKind::Constructor { owner, name, fields } if owner == "Word.Nil" && name == "WNil" && fields.is_empty() && program.base_names.contains(owner)).then_some(word)
}

fn c_string(value: &str) -> String {
    let mut output = String::from("\"");
    for byte in value.bytes() {
        write!(output, "\\{byte:03o}").unwrap();
    }
    output.push('"');
    output
}
