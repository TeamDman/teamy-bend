// SPDX-License-Identifier: MPL-2.0
//! Sealed execution-only binary32 operations. Every argument decoder transition
//! uses the machine's bounded continuation stack, including nested intrinsic calls.

use super::Frame;
use super::Machine;
use super::Thunk;
use super::ThunkId;
use super::Value;
use crate::kernel::KernelError;
use crate::syntax::executable::NumericIntrinsic;

pub(super) struct NumericArguments {
    intrinsic: NumericIntrinsic,
    arguments: Vec<ThunkId>,
    words: Vec<u32>,
}

pub(super) enum NumericFrame {
    Unwrap(NumericArguments),
    Word(NumericArguments, u32, u32),
    Bit(NumericArguments, u32, u32, ThunkId),
}

impl Machine<'_> {
    pub(super) fn apply_numeric(
        &mut self,
        intrinsic: NumericIntrinsic,
        mut arguments: Vec<ThunkId>,
        argument: ThunkId,
        frames: &mut Vec<Frame>,
    ) -> Result<ThunkId, KernelError> {
        arguments.push(argument);
        if arguments.len() < intrinsic.arity() {
            return self.allocate(Thunk::Ready(Value::Numeric {
                intrinsic,
                arguments,
            }));
        }
        if arguments.len() != intrinsic.arity() {
            return Err(KernelError::new("numeric intrinsic has an invalid arity"));
        }
        let first = arguments[0];
        Self::push(
            frames,
            Frame::Numeric(NumericFrame::Unwrap(NumericArguments {
                intrinsic,
                arguments,
                words: Vec::new(),
            })),
        )?;
        Ok(first)
    }

    pub(super) fn numeric_step(
        &mut self,
        frame: NumericFrame,
        value: Value,
        frames: &mut Vec<Frame>,
    ) -> Result<ThunkId, KernelError> {
        let Value::Constructor { name, fields } = value else {
            return Err(KernelError::new("numeric argument is not constructor data"));
        };
        match frame {
            NumericFrame::Unwrap(state) => {
                if name != state.intrinsic.input_type() || fields.len() != 1 {
                    return Err(KernelError::new("numeric argument has an invalid wrapper"));
                }
                Self::push(frames, Frame::Numeric(NumericFrame::Word(state, 0, 0)))?;
                Ok(fields[0])
            }
            NumericFrame::Word(mut state, bit, word) => {
                if bit == 32 {
                    if name != "WNil" || !fields.is_empty() {
                        return Err(KernelError::new(
                            "numeric argument needs exactly 32 Word bits",
                        ));
                    }
                    state.words.push(word);
                    if state.words.len() == state.arguments.len() {
                        return self.numeric_result(state.intrinsic, &state.words);
                    }
                    let next = state.arguments[state.words.len()];
                    Self::push(frames, Frame::Numeric(NumericFrame::Unwrap(state)))?;
                    return Ok(next);
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
                match name.as_str() {
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
    fn numeric_result(
        &mut self,
        intrinsic: NumericIntrinsic,
        words: &[u32],
    ) -> Result<ThunkId, KernelError> {
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
        let result = match intrinsic {
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
        };
        if intrinsic.output_type() == "Bool" {
            return self.ready_constructor(if result == 0 { "False" } else { "True" }, vec![]);
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
        self.ready_constructor(intrinsic.output_type(), vec![word])
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
