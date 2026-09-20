// SPDX-License-Identifier: MPL-2.0
//! Callback-local views. No pointer escapes into retained state or another thread.
use napi::Env;
use napi::JsValue;
use napi::Result;
use napi::Unknown;
use napi::ValueType;
use napi::sys;
use std::ptr;

#[derive(Clone, Copy)]
pub(super) struct View {
    pointer: *mut u8,
    length: usize,
}

impl View {
    pub(super) fn optional(env: Env, value: Unknown<'_>) -> Result<Option<Self>> {
        if matches!(value.get_type()?, ValueType::Null | ValueType::Undefined) {
            return Ok(None);
        }
        Self::new(env, value).map(Some)
    }

    pub(super) fn new(env: Env, value: Unknown<'_>) -> Result<Self> {
        let mut kind = 0;
        let mut count = 0;
        let mut data = ptr::null_mut();
        let mut buffer = ptr::null_mut();
        let mut offset = 0;
        // SAFETY: The NAPI arguments and all output pointers remain live for this
        // synchronous call. NAPI validates the JavaScript value's view type.
        let status = unsafe {
            sys::napi_get_typedarray_info(
                env.raw(),
                value.raw(),
                &mut kind,
                &mut count,
                &mut data,
                &mut buffer,
                &mut offset,
            )
        };
        if status != sys::Status::napi_ok {
            return Err(napi::Error::from_reason(
                "syscall pointer must be a TypedArray view",
            ));
        }
        let mut ordinary_buffer = false;
        // SAFETY: buffer was returned by NAPI in the same live environment.
        let status = unsafe { sys::napi_is_arraybuffer(env.raw(), buffer, &mut ordinary_buffer) };
        if status != sys::Status::napi_ok || !ordinary_buffer {
            return Err(napi::Error::from_reason(
                "shared syscall buffers are unsupported",
            ));
        }
        let width = match kind {
            0..=2 => 1,
            3 | 4 => 2,
            5..=7 => 4,
            8..=10 => 8,
            _ => {
                return Err(napi::Error::from_reason(
                    "unsupported TypedArray element type",
                ));
            }
        };
        let length = count
            .checked_mul(width)
            .ok_or_else(|| napi::Error::from_reason("TypedArray byte length overflow"))?;
        if length != 0 && data.is_null() {
            return Err(napi::Error::from_reason("detached syscall buffer"));
        }
        // A non-null pointer is needed even for a zero-length syscall buffer.
        let pointer = if data.is_null() {
            ptr::NonNull::<u8>::dangling().as_ptr()
        } else {
            data.cast()
        };
        Ok(Self { pointer, length })
    }

    pub(super) fn len(self) -> usize {
        self.length
    }
    pub(super) fn pointer(self) -> *mut u8 {
        self.pointer
    }
    pub(super) fn valid_length(self, length: f64) -> bool {
        length.is_finite()
            && length >= 0.0
            && length <= f64::from(i32::MAX)
            && length.fract() == 0.0
            && length as usize <= self.length
    }
    pub(super) fn read<const N: usize>(self, offset: usize) -> [u8; N] {
        assert!(offset <= self.length && N <= self.length - offset);
        let mut bytes = [0; N];
        // SAFETY: Checked view bounds; JS cannot execute or detach its ordinary
        // ArrayBuffer during this synchronous callback. Dest is separate storage.
        unsafe {
            ptr::copy_nonoverlapping(self.pointer.add(offset), bytes.as_mut_ptr(), N);
        }
        bytes
    }
    pub(super) fn write(self, offset: usize, bytes: &[u8]) {
        assert!(offset <= self.length && bytes.len() <= self.length - offset);
        // SAFETY: Same callback-local lifetime and checked bounds as read. The
        // source is Rust-owned; raw writes avoid inventing aliasing references.
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), self.pointer.add(offset), bytes.len());
        }
    }
    pub(super) fn read_i32(self, offset: usize) -> i32 {
        i32::from_ne_bytes(self.read(offset))
    }
    pub(super) fn read_u32(self, offset: usize) -> u32 {
        u32::from_ne_bytes(self.read(offset))
    }
    pub(super) fn write_i32(self, offset: usize, value: i32) {
        self.write(offset, &value.to_ne_bytes());
    }
    pub(super) fn write_u32(self, offset: usize, value: u32) {
        self.write(offset, &value.to_ne_bytes());
    }
}
