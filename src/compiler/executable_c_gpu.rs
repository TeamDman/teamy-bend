// SPDX-License-Identifier: MPL-2.0
//! CUDA assembly from the same typed resumes used by the CPU dispatcher.

use super::CompileError;
use super::DEVICE_ARRAY_NEW_STATE_WORDS;
use super::DEVICE_DUPLICATE_STATE_WORDS;
use super::DEVICE_PAYLOAD_STATE_WORDS;
use super::DefinitionBody;
use super::ExecutableProgram;
use super::Expression;
use super::ExpressionKind;
use super::Generator;
use super::Kind;
use super::Quant;
use super::segments::Signature;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Write;

/// Scheduling marks are global definition properties in upstream's reachable
/// source book. Erased operands do not introduce executable dependencies.
pub(super) fn marked_names(program: &ExecutableProgram) -> BTreeSet<String> {
    let mut marks = BTreeSet::new();
    let mut seen = BTreeSet::new();
    let mut queue = vec!["main".to_owned()];
    while let Some(name) = queue.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        if let Some(definition) = program.definitions.get(&name)
            && let DefinitionBody::Ordinary(body) = &definition.body
        {
            expression_references(body, &mut marks, &mut queue);
        }
    }
    marks
}

fn expression_references(
    expression: &Expression,
    marks: &mut BTreeSet<String>,
    references: &mut Vec<String>,
) {
    match &expression.kind {
        ExpressionKind::Definition(name) => {
            if expression.is_gpu_reference() {
                marks.insert(name.clone());
            }
            references.push(name.clone());
        }
        ExpressionKind::Lambda { body, .. } => expression_references(body, marks, references),
        ExpressionKind::Apply {
            function,
            argument,
            quant,
        } => {
            expression_references(function, marks, references);
            if *quant != Quant::None {
                expression_references(argument, marks, references);
            }
        }
        ExpressionKind::Constructor { fields, .. } => {
            for field in fields
                .iter()
                .filter(|field| field.binder.quant != Quant::None)
            {
                expression_references(&field.value, marks, references);
            }
        }
        ExpressionKind::Match { arm, fallback, .. } => {
            expression_references(arm, marks, references);
            expression_references(fallback, marks, references);
        }
        ExpressionKind::Let { bindings, body } => {
            for field in bindings
                .iter()
                .filter(|field| field.binder.quant != Quant::None)
            {
                expression_references(&field.value, marks, references);
            }
            expression_references(body, marks, references);
        }
        ExpressionKind::Erased | ExpressionKind::Variable(_) | ExpressionKind::Absurd { .. } => {}
    }
}

impl Generator<'_> {
    fn gpu_segment(&self, id: usize) -> bool {
        self.closures[id]
            .origin
            .as_ref()
            .is_some_and(|name| self.gpu_names.contains(name))
    }

    /// A marked call must reach the coordinator's task boundary. In particular,
    /// direct-call frame reuse must not consume its offload opportunity first.
    pub(super) fn segment_outcome(
        expression: &Expression,
        signature: &Signature,
        count: usize,
        words: &str,
        owned: &str,
    ) -> String {
        let fid = signature.id + 2;
        let mut head = expression;
        while let ExpressionKind::Apply { function, .. } = &head.kind {
            head = function;
        }
        if head.is_gpu_reference() {
            format!("tb_segment_task(tb_c_word_task(e, {fid}, {count}, {words}, {owned}))")
        } else {
            format!("tb_segment_call({fid}, {count}, {words}, {owned})")
        }
    }

    fn gpu_roots(&self) -> BTreeSet<usize> {
        self.segment_ids
            .values()
            .flatten()
            .filter(|signature| self.gpu_segment(signature.id))
            .map(|signature| signature.id)
            .collect()
    }

    pub(super) fn gpu_marks(&self, source: &mut String) {
        source.push_str("static TBOutcome tb_gpu_execute(Env e, Term root);\nstatic bool tb_gpu_marked(Fid fid) {\n  switch (fid) {\n");
        for id in self.gpu_roots() {
            writeln!(source, "  case {}: return true;", id + 2).unwrap();
        }
        source.push_str("  default: return false;\n  }\n}\n");
    }

    pub(super) fn gpu_source(
        &self,
        requests: &BTreeMap<String, u16>,
    ) -> Result<Option<String>, CompileError> {
        let roots = self.gpu_roots();
        if roots.is_empty() {
            return Ok(None);
        }
        let mut reachable = BTreeSet::new();
        let mut queue: Vec<_> = roots.iter().copied().collect();
        // A boxed live argument can contain a closure arbitrarily deep inside
        // an ADT. Include every minted closure conservatively, as upstream does.
        let mut wide = self.segment_ids.values().flatten().any(|signature| {
            roots.contains(&signature.id)
                && signature
                    .arguments
                    .iter()
                    .any(|layout| layout.words.contains(&Kind::Box))
        });
        if wide {
            queue.extend(0..self.closures.len());
        }
        while let Some(id) = queue.pop() {
            if !reachable.insert(id) {
                continue;
            }
            let closure = self
                .closures
                .get(id)
                .ok_or_else(|| CompileError::new("invalid CUDA function dependency"))?;
            queue.extend(&closure.dependencies.fids);
            if closure.dependencies.dynamic_calls && !wide {
                wide = true;
                queue.extend(0..self.closures.len());
            }
        }
        // Host capabilities are excluded from dispatch, not executed through an
        // accidental device callback. Their values may still be held or dropped.
        reachable.retain(|id| self.closures[*id].dependencies.host_operations.is_empty());
        let mut source = include_str!("executable_device_core.cu").replace(
            "/* TB_DEVICE_STATE */",
            include_str!("executable_device_state.h"),
        );
        source.push_str(
            &self
                .table
                .declarations()
                .replace("static const", "static __device__ const"),
        );
        self.device_tables(&reachable, &mut source);
        source.push_str(include_str!("executable_value_core.c"));
        source.push_str(include_str!("executable_array_core.c"));
        source.push_str("INLINE void tb_reject_request(Term value) {\n  if (term_tag(value) != TAG_CTR) return;\n  switch (term_aux(value)) {\n");
        for cid in requests.values().collect::<BTreeSet<_>>() {
            writeln!(
                source,
                "  case {cid}: err_fail(\"foreign request inspected as ordinary data\");"
            )
            .unwrap();
        }
        source.push_str("  default: return;\n  }\n}\n");
        source.push_str(include_str!("executable_device_control.h"));
        source.push_str(include_str!("executable_device_tasks.cu"));
        source.push_str(include_str!("executable_device_primitives.cu"));
        source.push_str(include_str!("executable_device_duplicate.cu"));
        writeln!(source, "#if TB_DEVICE_ARRAY_NEW_STATE_WORDS != {DEVICE_ARRAY_NEW_STATE_WORDS}\n#error incompatible generated Array.new state layout\n#endif").unwrap();
        writeln!(source, "#if TB_DEVICE_PAYLOAD_STATE_WORDS != {DEVICE_PAYLOAD_STATE_WORDS}\n#error incompatible generated payload state layout\n#endif").unwrap();
        writeln!(source, "#if TB_DEVICE_DUPLICATE_STATE_WORDS != {DEVICE_DUPLICATE_STATE_WORDS}\n#error incompatible generated duplication state layout\n#endif").unwrap();
        source.push_str(include_str!("executable_value_bridge.c"));
        source.push_str(
            "INLINE Term tb_impossible(void) { err_fail(\"entered an impossible match\"); }\n",
        );
        for id in &reachable {
            let result = if self.closures[*id].segment.is_some() {
                "TBOutcome"
            } else {
                "Term"
            };
            writeln!(
                source,
                "static __device__ TB_NOINLINE {result} tb_resume_{id}(const Env*, TBCallFrame*);"
            )
            .unwrap();
        }
        // Conversions share the exact CPU layout and ownership rules. Their
        // explicit dependency metadata permits later pruning without C parsing.
        for id in 0..self.conversions.len() {
            writeln!(source, "static __device__ TB_NOINLINE Term tb_box_{id}(const Env*, const Term*);\nstatic __device__ TB_NOINLINE void tb_unbox_{id}(const Env*, Term, Term*);").unwrap();
        }
        for id in &reachable {
            source.push_str(&device_functions(&self.closures[*id].resume_source));
        }
        for conversion in &self.conversions {
            source.push_str(&device_functions(conversion));
        }
        source.push_str("OUTLINE TBOutcome tb_device_resume(u32 fid, const Env *e, TBCallFrame *frame) {\n  switch (fid) {\n");
        for id in &reachable {
            if self.closures[*id].segment.is_some() {
                writeln!(
                    source,
                    "  case {}: return tb_resume_{id}(e, frame);",
                    id + 2
                )
                .unwrap();
            } else {
                writeln!(source, "  case {}: {{ Term result = tb_resume_{id}(e, frame); if (frame->yielded) return tb_segment_yield(); *frame->result = result; return term_tag(result) == TAG_TSK ? tb_segment_task(result) : tb_segment_words(frame->result, NULL, 1); }}", id + 2).unwrap();
            }
        }
        source.push_str("  default: err_fail(\"host-only or unknown GPU function\");\n  }\n}\n");
        Ok(Some(source))
    }

    fn device_tables(&self, reachable: &BTreeSet<usize>, source: &mut String) {
        source.push_str("INLINE u32 tb_device_fid_kind(u32 fid) { switch (fid) {\n");
        for id in reachable {
            writeln!(
                source,
                "case {}: return {};",
                id + 2,
                if self.closures[*id].segment.is_some() {
                    2
                } else {
                    1
                }
            )
            .unwrap();
        }
        source.push_str("default: return 0; } }\n");
        for (name, field) in [
            ("fid_arity", 0),
            ("fid_result_width", 1),
            ("tb_device_fid_slots", 2),
        ] {
            writeln!(source, "INLINE u32 {name}(u32 fid) {{ switch (fid) {{").unwrap();
            if field != 2 {
                writeln!(
                    source,
                    "case FID_CLO_APPLY: return {}; case FID_IO_EMIT: return 1;",
                    if field == 0 { 2 } else { 1 }
                )
                .unwrap();
            }
            for (id, closure) in self.closures.iter().enumerate() {
                let (arity, width) = closure.segment.unwrap_or((closure.captures + 1, 1));
                let value = [arity, width, closure.slots][field];
                writeln!(source, "case {}: return {value};", id + 2).unwrap();
            }
            source.push_str("default: err_fail(\"unknown GPU function descriptor\"); } }\n");
        }
        source.push_str("extern \"C\" __global__ void tb_device_tables_initialize(void) {\n  if (blockIdx.x != 0 || threadIdx.x != 0) return;\n");
        for (id, closure) in self.closures.iter().enumerate() {
            writeln!(
                source,
                "  tb_closure_captures[{}] = {};",
                id + 2,
                closure.captures
            )
            .unwrap();
        }
        source.push_str("}\n");
    }
}

// This only decorates declarations produced by Body; dependency discovery uses
// assigned fid metadata, never generated-text pattern matching.
fn device_functions(source: &str) -> String {
    source.replace("static TB_NOINLINE", "static __device__ TB_NOINLINE")
}

/// Separate string objects avoid MSVC's limit on a concatenated string literal.
pub(super) fn embed_source(device: &str, source: &mut String) {
    source.push_str("static const char *const tb_gpu_source_parts[] = {\n");
    for chunk in device.as_bytes().chunks(1024) {
        source.push_str("  \"");
        for byte in chunk {
            match byte {
                b'\n' => source.push_str("\\n"),
                b'\r' => source.push_str("\\r"),
                b'\t' => source.push_str("\\t"),
                b'\\' => source.push_str("\\\\"),
                b'\"' => source.push_str("\\\""),
                32..=126 => source.push(char::from(*byte)),
                _ => write!(source, "\\{byte:03o}").unwrap(),
            }
        }
        source.push_str("\",\n");
    }
    source.push_str("  NULL\n};\n");
}
