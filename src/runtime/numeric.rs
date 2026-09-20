// SPDX-License-Identifier: MPL-2.0
//! Sealed execution-only numeric operations. Every argument decoder transition
//! uses the machine's bounded continuation stack, including nested intrinsic calls.

use super::Frame;
use super::Machine;
use super::Thunk;
use super::ThunkId;
use super::Value;
use super::packed::Wrapper;
use crate::kernel::DefDecl;
use crate::kernel::KernelError;
use crate::kernel::Quant;
use crate::kernel::Term;
use crate::kernel::TermRef;
use crate::syntax::executable::NumericIntrinsic;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[path = "numeric_text.rs"]
mod text;

#[cfg(test)]
#[path = "numeric_tests.rs"]
mod tests;

/// An optimization of a checked ordinary body, never an opaque assumption.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum PureOptimization {
    Add,
    Mul,
    Shl,
}

impl PureOptimization {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Add => "U32.add",
            Self::Mul => "U32.mul",
            Self::Shl => "U32.shl",
        }
    }

    const fn arity(self) -> usize {
        match self {
            Self::Add | Self::Mul => 2,
            Self::Shl => 1,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum NumericOperation {
    Intrinsic(NumericIntrinsic),
    Optimized(PureOptimization),
}

impl NumericOperation {
    const fn arity(self) -> usize {
        match self {
            Self::Intrinsic(intrinsic) => intrinsic.arity(),
            Self::Optimized(operation) => operation.arity(),
        }
    }

    const fn input_type(self) -> &'static str {
        match self {
            Self::Intrinsic(intrinsic) => intrinsic.input_type(),
            Self::Optimized(_) => "U32",
        }
    }

    const fn output_type(self) -> &'static str {
        match self {
            Self::Intrinsic(intrinsic) => intrinsic.output_type(),
            Self::Optimized(_) => "U32",
        }
    }
}

/// Called only after executable checking, with origins minted by the loader.
/// A user definition with the same name or type must keep its ordinary body.
pub(super) fn checked_optimizations(
    definitions: &BTreeMap<String, DefDecl>,
    base_names: &BTreeSet<String>,
) -> BTreeSet<PureOptimization> {
    let mut result = BTreeSet::new();
    for operation in [
        PureOptimization::Add,
        PureOptimization::Mul,
        PureOptimization::Shl,
    ] {
        if base_names.contains(operation.name())
            && base_names.contains("U32")
            && definitions.get(operation.name()).is_some_and(|definition| {
                definition.name == operation.name()
                    && is_checked_word_operation(definition, operation.arity())
            })
        {
            result.insert(operation);
        }
    }
    result
}

fn strip_annotations(mut value: &TermRef) -> &TermRef {
    while let Term::Ann(inner, _) = value.as_ref() {
        value = inner;
    }
    value
}

fn is_checked_word_operation(definition: &DefDecl, arity: usize) -> bool {
    if definition.body.is_none()
        || definition.foreign
        || definition.unsafe_
        || definition.parameters.len() != arity
    {
        return false;
    }
    let mut telescope = strip_annotations(&definition.ty);
    for parameter in &definition.parameters {
        let Term::All {
            quant,
            id,
            domain,
            body,
            ..
        } = telescope.as_ref()
        else {
            return false;
        };
        if *quant != Quant::Lone
            || parameter.quant != Quant::Lone
            || *id != parameter.id
            || !matches!(strip_annotations(domain).as_ref(), Term::Ref(name) if name == "U32")
            || !matches!(strip_annotations(&parameter.ty).as_ref(), Term::Ref(name) if name == "U32")
        {
            return false;
        }
        telescope = strip_annotations(body);
    }
    matches!(telescope.as_ref(), Term::Ref(name) if name == "U32")
}

pub(super) struct NumericArguments {
    operation: NumericOperation,
    arguments: Vec<ThunkId>,
    words: Vec<u32>,
}

pub(super) enum NumericFrame {
    Ordinary(OrdinaryArguments),
    Unwrap(WordTarget),
    Word(WordTarget, u32, u32),
    Bit(WordTarget, u32, u32, ThunkId),
    Text(String),
    Character(String, ThunkId),
}

impl NumericFrame {
    pub(super) fn visit_roots(
        &self,
        mut visit: impl FnMut(ThunkId) -> Result<(), KernelError>,
    ) -> Result<(), KernelError> {
        match self {
            Self::Ordinary(state) => {
                for argument in &state.arguments {
                    visit(*argument)?;
                }
                Ok(())
            }
            Self::Unwrap(target) | Self::Word(target, ..) => target.visit_roots(visit),
            Self::Bit(target, _, _, tail) => {
                target.visit_roots(&mut visit)?;
                visit(*tail)
            }
            Self::Character(_, tail) => visit(*tail),
            Self::Text(_) => Ok(()),
        }
    }
}

pub(super) struct OrdinaryArguments {
    operation: PureOptimization,
    arguments: Vec<ThunkId>,
    words: Vec<u32>,
}

pub(super) enum WordTarget {
    Arguments(NumericArguments),
    Character(String, ThunkId),
}

impl WordTarget {
    fn visit_roots(
        &self,
        mut visit: impl FnMut(ThunkId) -> Result<(), KernelError>,
    ) -> Result<(), KernelError> {
        match self {
            Self::Arguments(state) => {
                for argument in &state.arguments {
                    visit(*argument)?;
                }
                Ok(())
            }
            Self::Character(_, tail) => visit(*tail),
        }
    }

    fn wrapper(&self) -> &'static str {
        match self {
            Self::Arguments(arguments) => arguments.operation.input_type(),
            Self::Character(..) => "U32",
        }
    }
}

impl Machine<'_> {
    pub(super) fn apply_numeric(
        &mut self,
        operation: NumericOperation,
        mut arguments: Vec<ThunkId>,
        argument: ThunkId,
        frames: &mut Vec<Frame>,
    ) -> Result<ThunkId, KernelError> {
        arguments.push(argument);
        if arguments.len() < operation.arity() {
            return self.allocate(Thunk::Ready(Value::Numeric {
                operation,
                arguments,
            }));
        }
        if arguments.len() != operation.arity() {
            return Err(KernelError::new("numeric intrinsic has an invalid arity"));
        }
        if let NumericOperation::Optimized(ordinary) = operation {
            // These checked Base bodies match their outer U32 arguments in
            // declaration order. Demand only those wrappers; forcing their
            // Word fields here would change the bodies' lazy behavior.
            let first = arguments[0];
            Self::push(
                frames,
                Frame::Numeric(NumericFrame::Ordinary(OrdinaryArguments {
                    operation: ordinary,
                    arguments,
                    words: Vec::new(),
                })),
            )?;
            return Ok(first);
        }
        let first = arguments[0];
        let frame = if matches!(
            operation,
            NumericOperation::Intrinsic(NumericIntrinsic::Read)
        ) {
            NumericFrame::Text(String::new())
        } else {
            NumericFrame::Unwrap(WordTarget::Arguments(NumericArguments {
                operation,
                arguments,
                words: Vec::new(),
            }))
        };
        Self::push(frames, Frame::Numeric(frame))?;
        Ok(first)
    }

    // A checked ordinary function may ignore parts of a Word. Once an outer
    // value is ordinary, its fields must retain the checked body's demand.
    // Enter its stored body directly so reference lookup cannot select this fast
    // path again, and preserve every original lazy argument thunk.
    fn ordinary_numeric(
        &mut self,
        operation: PureOptimization,
        arguments: &[ThunkId],
    ) -> Result<ThunkId, KernelError> {
        let body = self
            .program
            .definitions
            .get(operation.name())
            .and_then(|definition| definition.body.as_ref())
            .cloned()
            .ok_or_else(|| KernelError::new("ordinary numeric optimization has no checked body"))?;
        let mut function = self.expression(body, 0)?;
        for argument in arguments {
            function = self.allocate(Thunk::Application(function, *argument))?;
        }
        Ok(function)
    }

    pub(super) fn numeric_step(
        &mut self,
        frame: NumericFrame,
        value: Value,
        frames: &mut Vec<Frame>,
    ) -> Result<ThunkId, KernelError> {
        match (frame, value) {
            (
                NumericFrame::Ordinary(mut state),
                Value::PackedWord {
                    wrapper: Wrapper::U32,
                    bits,
                },
            ) => {
                state.words.push(bits);
                if state.words.len() == state.arguments.len() {
                    self.numeric_result(NumericOperation::Optimized(state.operation), &state.words)
                } else {
                    let next = state.arguments[state.words.len()];
                    Self::push(frames, Frame::Numeric(NumericFrame::Ordinary(state)))?;
                    Ok(next)
                }
            }
            (NumericFrame::Ordinary(state), _) => {
                self.ordinary_numeric(state.operation, &state.arguments)
            }
            (NumericFrame::Unwrap(state), Value::PackedWord { wrapper, bits }) => {
                if wrapper.name() != state.wrapper() {
                    return Err(KernelError::new("numeric argument has an invalid wrapper"));
                }
                self.numeric_word(state, bits, frames)
            }
            (NumericFrame::Word(state, bit, word), Value::PackedBits { bits, width }) => {
                if bit > 32
                    || u32::from(width) != 32 - bit
                    || !super::packed::valid_bits(bits, width)
                {
                    return Err(KernelError::new(
                        "numeric argument needs exactly 32 Word bits",
                    ));
                }
                let result = if bit == 32 {
                    word
                } else {
                    word | (bits << bit)
                };
                self.numeric_word(state, result, frames)
            }
            (frame, Value::Constructor { name, fields }) => {
                self.numeric_constructor_step(frame, &name, &fields, frames)
            }
            _ => Err(KernelError::new("numeric argument is not constructor data")),
        }
    }

    fn numeric_constructor_step(
        &mut self,
        frame: NumericFrame,
        name: &str,
        fields: &[ThunkId],
        frames: &mut Vec<Frame>,
    ) -> Result<ThunkId, KernelError> {
        match frame {
            NumericFrame::Ordinary(state) => {
                self.ordinary_numeric(state.operation, &state.arguments)
            }
            NumericFrame::Unwrap(state) => {
                if name != state.wrapper() || fields.len() != 1 {
                    return Err(KernelError::new("numeric argument has an invalid wrapper"));
                }
                Self::push(frames, Frame::Numeric(NumericFrame::Word(state, 0, 0)))?;
                Ok(fields[0])
            }
            NumericFrame::Word(state, bit, word) => {
                if bit == 32 {
                    if name != "WNil" || !fields.is_empty() {
                        return Err(KernelError::new(
                            "numeric argument needs exactly 32 Word bits",
                        ));
                    }
                    return self.numeric_word(state, word, frames);
                }
                if name != "WCon" || fields.len() != 2 {
                    return Err(KernelError::new("numeric argument needs a 32-bit Word"));
                }
                Self::push(
                    frames,
                    Frame::Numeric(NumericFrame::Bit(state, bit, word, fields[1])),
                )?;
                Ok(fields[0])
            }
            NumericFrame::Bit(state, bit, mut word, tail) => {
                if !fields.is_empty() {
                    return Err(KernelError::new(
                        "numeric argument has an invalid Boolean bit",
                    ));
                }
                match name {
                    "True" => word |= 1 << bit,
                    "False" => {}
                    _ => {
                        return Err(KernelError::new(
                            "numeric argument has an invalid Boolean bit",
                        ));
                    }
                }
                Self::push(
                    frames,
                    Frame::Numeric(NumericFrame::Word(state, bit + 1, word)),
                )?;
                Ok(tail)
            }
            NumericFrame::Text(text) => match (name, fields) {
                ("SNil", []) => match text::read(&text) {
                    Some(bits) => {
                        let value = self.numeric_word_value("F32", bits)?;
                        self.ready_constructor("Some", vec![value])
                    }
                    None => self.ready_constructor("None", vec![]),
                },
                ("SCon", [head, tail]) => {
                    Self::push(frames, Frame::Numeric(NumericFrame::Character(text, *tail)))?;
                    Ok(*head)
                }
                _ => Err(KernelError::new("numeric argument needs a String value")),
            },
            NumericFrame::Character(text, tail) => {
                if name != "Chr" || fields.len() != 1 {
                    return Err(KernelError::new("numeric text needs a Char value"));
                }
                Self::push(
                    frames,
                    Frame::Numeric(NumericFrame::Unwrap(WordTarget::Character(text, tail))),
                )?;
                Ok(fields[0])
            }
        }
    }

    fn numeric_word(
        &mut self,
        target: WordTarget,
        word: u32,
        frames: &mut Vec<Frame>,
    ) -> Result<ThunkId, KernelError> {
        match target {
            WordTarget::Arguments(mut state) => {
                state.words.push(word);
                if state.words.len() == state.arguments.len() {
                    return self.numeric_result(state.operation, &state.words);
                }
                let next = state.arguments[state.words.len()];
                Self::push(
                    frames,
                    Frame::Numeric(NumericFrame::Unwrap(WordTarget::Arguments(state))),
                )?;
                Ok(next)
            }
            WordTarget::Character(mut text, tail) => {
                let scalar = char::from_u32(word).ok_or_else(|| {
                    KernelError::new("numeric text contains an invalid Unicode scalar")
                })?;
                if text.len() + scalar.len_utf8() > super::executable::TEXT_BYTES {
                    return Err(KernelError::new("numeric text byte budget exhausted"));
                }
                text.push(scalar);
                Self::push(frames, Frame::Numeric(NumericFrame::Text(text)))?;
                Ok(tail)
            }
        }
    }

    fn numeric_result(
        &mut self,
        operation: NumericOperation,
        words: &[u32],
    ) -> Result<ThunkId, KernelError> {
        if matches!(
            operation,
            NumericOperation::Intrinsic(NumericIntrinsic::Show)
        ) {
            let shown = text::show(f32::from_bits(words[0]));
            let mut string = self.ready_constructor("SNil", vec![])?;
            for byte in shown.bytes().rev() {
                self.tick()?;
                let code = self.numeric_word_value("U32", u32::from(byte))?;
                let character = self.ready_constructor("Chr", vec![code])?;
                string = self.ready_constructor("SCon", vec![character, string])?;
            }
            return Ok(string);
        }
        let result = match operation {
            NumericOperation::Intrinsic(intrinsic) => intrinsic_result(intrinsic, words),
            NumericOperation::Optimized(PureOptimization::Add) => words[0].wrapping_add(words[1]),
            NumericOperation::Optimized(PureOptimization::Mul) => words[0].wrapping_mul(words[1]),
            NumericOperation::Optimized(PureOptimization::Shl) => words[0].wrapping_shl(1),
        };
        if operation.output_type() == "Bool" {
            return self.ready_constructor(if result == 0 { "False" } else { "True" }, vec![]);
        }
        self.numeric_word_value(operation.output_type(), result)
    }

    fn numeric_word_value(&mut self, wrapper: &str, result: u32) -> Result<ThunkId, KernelError> {
        if self.program.packed {
            let wrapper = Wrapper::from_name(wrapper)
                .ok_or_else(|| KernelError::new("numeric result has an invalid wrapper"))?;
            return self.allocate(Thunk::Ready(Value::PackedWord {
                wrapper,
                bits: result,
            }));
        }
        let mut word = self.ready_constructor("WNil", vec![])?;
        for bit in (0..32).rev() {
            let head = self.ready_constructor(
                if result & (1 << bit) == 0 {
                    "False"
                } else {
                    "True"
                },
                vec![],
            )?;
            word = self.ready_constructor("WCon", vec![head, word])?;
        }
        self.ready_constructor(wrapper, vec![word])
    }

    fn ready_constructor(
        &mut self,
        name: &str,
        fields: Vec<ThunkId>,
    ) -> Result<ThunkId, KernelError> {
        self.allocate(Thunk::Ready(Value::Constructor {
            name: name.into(),
            fields,
        }))
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "U32 to binary32 is the specified rounding operation"
)]
#[expect(
    clippy::float_cmp,
    reason = "Bend float comparison primitives use IEEE equality"
)]
#[expect(
    clippy::cast_possible_truncation,
    reason = "upstream C computes transcendental functions in double precision, then rounds to binary32"
)]
fn intrinsic_result(intrinsic: NumericIntrinsic, words: &[u32]) -> u32 {
    use NumericIntrinsic::Abs;
    use NumericIntrinsic::Add;
    use NumericIntrinsic::Bits;
    use NumericIntrinsic::Div;
    use NumericIntrinsic::F32ToU32;
    use NumericIntrinsic::IsEq;
    use NumericIntrinsic::IsGe;
    use NumericIntrinsic::IsGt;
    use NumericIntrinsic::IsLe;
    use NumericIntrinsic::IsLt;
    use NumericIntrinsic::IsNe;
    use NumericIntrinsic::Mod;
    use NumericIntrinsic::Mul;
    use NumericIntrinsic::Neg;
    use NumericIntrinsic::Sub;
    use NumericIntrinsic::U32ToF32;
    let a = f32::from_bits(words[0]);
    let b = f32::from_bits(words.get(1).copied().unwrap_or(0));
    let double = f64::from(a);
    match intrinsic {
        U32ToF32 => (words[0] as f32).to_bits(),
        F32ToU32 => float_to_u32(a),
        Add => (a + b).to_bits(),
        Sub => (a - b).to_bits(),
        Mul => (a * b).to_bits(),
        Div => (a / b).to_bits(),
        Mod => (a % b).to_bits(),
        // Sign operations preserve payloads, including signaling NaNs.
        Neg => words[0] ^ 0x8000_0000,
        Abs => words[0] & 0x7fff_ffff,
        Bits => words[0],
        IsEq => u32::from(a == b),
        IsNe => u32::from(a != b),
        IsLt => u32::from(a < b),
        IsLe => u32::from(a <= b),
        IsGt => u32::from(a > b),
        IsGe => u32::from(a >= b),
        NumericIntrinsic::Pow => (double.powf(f64::from(b)) as f32).to_bits(),
        NumericIntrinsic::Atan2 => (double.atan2(f64::from(b)) as f32).to_bits(),
        NumericIntrinsic::Sqrt => (double.sqrt() as f32).to_bits(),
        NumericIntrinsic::Exp => (double.exp() as f32).to_bits(),
        NumericIntrinsic::Log => (double.ln() as f32).to_bits(),
        NumericIntrinsic::Log2 => (double.log2() as f32).to_bits(),
        NumericIntrinsic::Log10 => (double.log10() as f32).to_bits(),
        NumericIntrinsic::Sin => (double.sin() as f32).to_bits(),
        NumericIntrinsic::Cos => (double.cos() as f32).to_bits(),
        NumericIntrinsic::Tan => (double.tan() as f32).to_bits(),
        NumericIntrinsic::Asin => (double.asin() as f32).to_bits(),
        NumericIntrinsic::Acos => (double.acos() as f32).to_bits(),
        NumericIntrinsic::Atan => (double.atan() as f32).to_bits(),
        NumericIntrinsic::Sinh => (double.sinh() as f32).to_bits(),
        NumericIntrinsic::Cosh => (double.cosh() as f32).to_bits(),
        NumericIntrinsic::Tanh => (double.tanh() as f32).to_bits(),
        NumericIntrinsic::Floor => (double.floor() as f32).to_bits(),
        NumericIntrinsic::Ceil => (double.ceil() as f32).to_bits(),
        NumericIntrinsic::Trunc => (double.trunc() as f32).to_bits(),
        NumericIntrinsic::Show | NumericIntrinsic::Read => {
            unreachable!("text contracts have dedicated runtime continuations")
        }
    }
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "finite values in the checked U32 range truncate toward zero; every other input returns zero"
)]
fn float_to_u32(value: f32) -> u32 {
    if (0.0..4_294_967_296.0).contains(&value) {
        value as u32
    } else {
        0
    }
}
