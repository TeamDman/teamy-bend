// SPDX-License-Identifier: MPL-2.0
//! Sealed file identities. Host ownership stays on the VM or its pending job.

use super::ARENA_LIMIT;
use super::host_files::NativeFile;
use crate::kernel::KernelError;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Handle(u64);

#[derive(Default)]
pub(super) struct State {
    files: BTreeMap<u64, NativeFile>,
    next: u64,
}

impl State {
    #[cfg(test)]
    pub(super) fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    pub(super) fn insert(&mut self, file: NativeFile) -> Result<Handle, KernelError> {
        let id = self.next;
        self.next = id
            .checked_add(1)
            .ok_or_else(|| KernelError::new("native file identity budget exhausted"))?;
        let handle = Handle(id);
        self.restore(handle, file)?;
        Ok(handle)
    }

    pub(super) fn take(&mut self, handle: Handle) -> Result<NativeFile, KernelError> {
        self.files.remove(&handle.0).ok_or_else(|| {
            KernelError::new("native file handle is closed or belongs to a pending operation")
        })
    }

    pub(super) fn restore(&mut self, handle: Handle, file: NativeFile) -> Result<(), KernelError> {
        if self.files.len() >= ARENA_LIMIT {
            return Err(KernelError::new("native file handle budget exhausted"));
        }
        if self.files.contains_key(&handle.0) {
            return Err(KernelError::new("native file handle was returned twice"));
        }
        self.files.insert(handle.0, file);
        Ok(())
    }
}
