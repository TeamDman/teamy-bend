// SPDX-License-Identifier: MPL-2.0
//! Owned native descriptors and parked requests. No OS call uses file workers.

use super::ARENA_LIMIT;
use super::ThunkId;
use super::host_files::Failure;
use super::host_network;
use super::host_network::Interest;
use super::host_network::NativeListener;
use super::host_network::NativeSocket;
use super::host_network::Registration;
use super::host_network::Source;
use crate::kernel::KernelError;
use std::collections::BTreeMap;

pub(super) const TRANSFER_BYTES: usize = 8 * 1024 * 1024;
const RETAINED_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct Handle(u64);

pub(super) enum Resource {
    Socket(NativeSocket),
    Listener(NativeListener),
}

impl Resource {
    pub(super) fn socket(&self) -> Result<&NativeSocket, KernelError> {
        match self {
            Self::Socket(socket) => Ok(socket),
            Self::Listener(_) => Err(KernelError::new("expected a native socket")),
        }
    }

    pub(super) fn listener(&self) -> Result<&NativeListener, KernelError> {
        match self {
            Self::Listener(listener) => Ok(listener),
            Self::Socket(_) => Err(KernelError::new("expected a native listener")),
        }
    }

    fn source(&self) -> Source<'_> {
        match self {
            Self::Socket(socket) => Source::Socket(socket),
            Self::Listener(listener) => Source::Listener(listener),
        }
    }
}

pub(super) enum Operation {
    Connect,
    Accept,
    Send {
        data: Vec<u8>,
        offset: usize,
    },
    Recv {
        max: u32,
    },
    SendTo {
        host: Vec<u8>,
        port: u32,
        data: Vec<u8>,
    },
    RecvFrom {
        max: u32,
    },
}

impl Operation {
    fn interest(&self) -> Interest {
        match self {
            Self::Connect | Self::Send { .. } | Self::SendTo { .. } => Interest::Write,
            Self::Accept | Self::Recv { .. } | Self::RecvFrom { .. } => Interest::Read,
        }
    }

    fn bytes(&self) -> usize {
        match self {
            Self::Connect | Self::Accept => 0,
            Self::Send { data, .. } => data.len(),
            Self::SendTo { host, data, .. } => host.len() + data.len(),
            Self::Recv { max } | Self::RecvFrom { max } => {
                usize::try_from(*max).unwrap_or(usize::MAX)
            }
        }
    }
}

pub(super) struct Request {
    pub(super) resource: Resource,
    pub(super) handle: Option<Handle>,
    pub(super) continuation: ThunkId,
    pub(super) operation: Operation,
}

#[derive(Default)]
pub(super) struct State {
    resources: BTreeMap<Handle, Resource>,
    pending: BTreeMap<u64, Request>,
    next: u64,
    retained_bytes: usize,
}

impl State {
    pub(super) fn insert(&mut self, resource: Resource) -> Result<Handle, KernelError> {
        let id = self.next;
        self.next = id
            .checked_add(1)
            .ok_or_else(|| KernelError::new("native network handle identity exhausted"))?;
        let handle = Handle(id);
        self.restore(handle, resource)?;
        Ok(handle)
    }

    pub(super) fn restore(
        &mut self,
        handle: Handle,
        resource: Resource,
    ) -> Result<(), KernelError> {
        if self.resources.len() + self.pending.len() >= ARENA_LIMIT {
            return Err(KernelError::new("native network handle budget exhausted"));
        }
        if self.resources.contains_key(&handle) {
            return Err(KernelError::new("native network handle returned twice"));
        }
        self.resources.insert(handle, resource);
        Ok(())
    }

    pub(super) fn take(&mut self, handle: Handle) -> Result<Resource, KernelError> {
        self.resources.remove(&handle).ok_or_else(|| {
            KernelError::new("native network handle is closed or belongs to a pending operation")
        })
    }

    pub(super) fn park(&mut self, order: u64, request: Request) -> Result<(), KernelError> {
        if self.pending.len() + self.resources.len() >= ARENA_LIMIT {
            return Err(KernelError::new("native network wait budget exhausted"));
        }
        let bytes = self
            .retained_bytes
            .checked_add(request.operation.bytes())
            .filter(|bytes| *bytes <= RETAINED_BYTES)
            .ok_or_else(|| KernelError::new("native network buffer budget exhausted"))?;
        if self.pending.contains_key(&order) {
            return Err(KernelError::new("duplicate native network wait identity"));
        }
        self.pending.insert(order, request);
        self.retained_bytes = bytes;
        Ok(())
    }

    pub(super) fn take_pending(&mut self, order: u64) -> Result<Request, KernelError> {
        let request = self
            .pending
            .remove(&order)
            .ok_or_else(|| KernelError::new("missing native network wait"))?;
        self.retained_bytes -= request.operation.bytes();
        Ok(request)
    }

    pub(super) fn poll(
        &self,
        wake: Option<&NativeSocket>,
        millis: u32,
    ) -> Result<Vec<u64>, KernelError> {
        let mut registrations: Vec<_> = self
            .pending
            .values()
            .map(|request| Registration {
                source: request.resource.source(),
                interest: request.operation.interest(),
            })
            .collect();
        if let Some(wake) = wake {
            registrations.push(Registration {
                source: Source::Socket(wake),
                interest: Interest::Read,
            });
        }
        let ready = host_network::poll(&registrations, millis).map_err(host_error)?;
        if ready.len() != registrations.len() {
            return Err(KernelError::new(
                "native poll returned invalid readiness count",
            ));
        }
        Ok(self
            .pending
            .keys()
            .zip(ready)
            .filter_map(|(order, ready)| ready.then_some(*order))
            .collect())
    }

    pub(super) fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    #[cfg(test)]
    pub(super) fn is_empty(&self) -> bool {
        self.resources.is_empty() && self.pending.is_empty() && self.retained_bytes == 0
    }

    pub(super) fn visit_roots(
        &self,
        mut visit: impl FnMut(ThunkId) -> Result<(), KernelError>,
    ) -> Result<(), KernelError> {
        for request in self.pending.values() {
            visit(request.continuation)?;
        }
        Ok(())
    }
}

pub(super) fn host_error(error: Failure) -> KernelError {
    match error {
        Failure::Limit(message) => KernelError::new(message),
        Failure::Io(error) => KernelError::new(format!(
            "native readiness failed with OS error {}: {}",
            error.code,
            String::from_utf8_lossy(&error.message)
        )),
    }
}
