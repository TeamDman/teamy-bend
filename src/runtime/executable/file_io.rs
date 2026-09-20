// SPDX-License-Identifier: MPL-2.0
//! Encode host answers and suspend checked file requests without moving VM
//! values to worker threads. Host failures remain execution-only Result data.

use super::super::file_handles::Handle;
use super::super::host_files;
use super::super::host_files::Failure;
use super::super::host_jobs;
use super::super::host_jobs::Reply;
use super::super::host_jobs::Task;
use super::BuiltinForeign;
use super::KernelError;
use super::Machine;
use super::Thunk;
use super::ThunkId;
use super::Value;
use super::Wrapper;

impl Machine<'_> {
    pub(super) fn file_request(
        &mut self,
        builtin: BuiltinForeign,
        arguments: &[ThunkId],
        continuation: ThunkId,
    ) -> Result<Option<ThunkId>, KernelError> {
        let answer = match (builtin, arguments) {
            (BuiltinForeign::GetEnv, [name]) => {
                let name = self.read_text(*name)?;
                match host_files::get_env(&name, host_jobs::JOB_BYTES) {
                    Ok(text) => {
                        let value = self.host_text(&text)?;
                        self.host_constructor("Done", vec![value])?
                    }
                    Err(error) => self.host_failure(error)?,
                }
            }
            (BuiltinForeign::FileOpen, [path, mode]) => {
                let path = self.read_text(*path)?;
                let mode = self.read_text(*mode)?;
                if let Err(error) = host_files::validate_open(&path, &mode) {
                    self.host_failure(error)?
                } else {
                    let bytes = path.len() + mode.len();
                    self.jobs
                        .submit(Task::Open { path, mode }, continuation, None, bytes)?;
                    return Ok(None);
                }
            }
            (BuiltinForeign::FileRead | BuiltinForeign::FileReadBytes, [file, max]) => {
                let handle = self.read_file(*file)?;
                let max = self.read_u32(*max)?;
                let bytes = host_files::read_size(max, host_jobs::JOB_BYTES).map_err(host_limit)?;
                let file = self.files.take(handle)?;
                self.jobs.submit(
                    Task::Read {
                        file,
                        max,
                        bytes: builtin == BuiltinForeign::FileReadBytes,
                    },
                    continuation,
                    Some(handle),
                    bytes,
                )?;
                return Ok(None);
            }
            (BuiltinForeign::FileWrite, [file, data]) => {
                let handle = self.read_file(*file)?;
                let data = self.read_text(*data)?;
                let bytes = data.len();
                let file = self.files.take(handle)?;
                self.jobs.submit(
                    Task::Write { file, data },
                    continuation,
                    Some(handle),
                    bytes,
                )?;
                return Ok(None);
            }
            (BuiltinForeign::FileClose, [file]) => {
                let handle = self.read_file(*file)?;
                host_files::close(self.files.take(handle)?);
                self.unit()?
            }
            _ => {
                return Err(KernelError::new(
                    "file/environment request has an invalid arity",
                ));
            }
        };
        Ok(Some(
            self.allocate(Thunk::Application(continuation, answer))?,
        ))
    }

    pub(super) fn resume_host_jobs(&mut self) -> Result<(), KernelError> {
        while let Some(completion) = self.jobs.completed()? {
            self.tick()?;
            let continuation = completion.continuation;
            let answer = self.with_roots(&[continuation], |machine| match completion.reply {
                Reply::Open(result) => match result {
                    Ok(file) => {
                        let handle = machine.files.insert(file)?;
                        let value = machine.allocate(Thunk::Ready(Value::File(handle)))?;
                        machine.host_constructor("Done", vec![value])
                    }
                    Err(error) => machine.host_failure(error),
                },
                Reply::Read { file, data, bytes } => {
                    let handle = completion
                        .handle
                        .ok_or_else(|| KernelError::new("native read lost its handle"))?;
                    machine.files.restore(handle, file)?;
                    let result = match data {
                        Ok(data) => {
                            let value = if bytes {
                                machine.host_bytes(&data)?
                            } else {
                                let points =
                                    host_files::decode_native_text(&data, host_jobs::JOB_BYTES)
                                        .map_err(host_limit)?;
                                machine.host_text(&points)?
                            };
                            machine.host_constructor("Done", vec![value])?
                        }
                        Err(error) => machine.host_failure(error)?,
                    };
                    machine.file_result(handle, result)
                }
                Reply::Write { file, result } => {
                    let handle = completion
                        .handle
                        .ok_or_else(|| KernelError::new("native write lost its handle"))?;
                    machine.files.restore(handle, file)?;
                    let result = match result {
                        Ok(()) => {
                            let unit = machine.unit()?;
                            machine.host_constructor("Done", vec![unit])?
                        }
                        Err(error) => machine.host_failure(error)?,
                    };
                    machine.file_result(handle, result)
                }
                Reply::Panicked => Err(KernelError::new("native file worker panicked")),
            })?;
            let action = self.allocate(Thunk::Application(continuation, answer))?;
            self.scheduler.resume(action)?;
        }
        Ok(())
    }

    fn read_file(&mut self, thunk: ThunkId) -> Result<Handle, KernelError> {
        match self.force(thunk)? {
            Value::File(handle) => Ok(handle),
            Value::Request { .. } => Err(KernelError::new(
                "runtime fail-stop: a foreign effect request escaped the IO driver",
            )),
            _ => Err(KernelError::new("expected a sealed native file handle")),
        }
    }

    fn file_result(&mut self, handle: Handle, result: ThunkId) -> Result<ThunkId, KernelError> {
        let file = self.allocate(Thunk::Ready(Value::File(handle)))?;
        self.host_constructor("Tuple", vec![file, result])
    }

    pub(super) fn host_failure(&mut self, error: Failure) -> Result<ThunkId, KernelError> {
        let error = match error {
            Failure::Io(error) => error,
            Failure::Limit(message) => return Err(KernelError::new(message)),
        };
        let code = self.host_word(error.code)?;
        let text = host_files::decode_native_text(&error.message, host_jobs::JOB_BYTES)
            .map_err(host_limit)?;
        let message = self.host_text(&text)?;
        let pair = self.host_constructor("Tuple", vec![code, message])?;
        self.host_constructor("Fail", vec![pair])
    }

    pub(super) fn host_word(&mut self, bits: u32) -> Result<ThunkId, KernelError> {
        self.allocate(Thunk::Ready(Value::PackedWord {
            wrapper: Wrapper::U32,
            bits,
        }))
    }

    pub(super) fn host_text(&mut self, points: &[u32]) -> Result<ThunkId, KernelError> {
        let mut text = self.host_constructor("SNil", vec![])?;
        for point in points.iter().rev() {
            self.tick()?;
            let code = self.host_word(*point)?;
            let character = self.host_constructor("Chr", vec![code])?;
            text = self.host_constructor("SCon", vec![character, text])?;
        }
        Ok(text)
    }

    fn host_bytes(&mut self, bytes: &[u8]) -> Result<ThunkId, KernelError> {
        let mut list = self.host_constructor("Nil", vec![])?;
        for byte in bytes.iter().rev() {
            self.tick()?;
            let head = self.host_word(u32::from(*byte))?;
            list = self.host_constructor("Con", vec![head, list])?;
        }
        Ok(list)
    }

    pub(super) fn host_constructor(
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

fn host_limit(error: Failure) -> KernelError {
    match error {
        Failure::Limit(message) => KernelError::new(message),
        Failure::Io(error) => {
            KernelError::new(format!("unexpected host conversion failure {}", error.code))
        }
    }
}
