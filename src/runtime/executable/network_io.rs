// SPDX-License-Identifier: MPL-2.0
//! Native network requests resume on the VM thread after descriptor readiness.

#[cfg(test)]
mod tests;

use super::super::host_files;
use super::super::host_files::Failure;
use super::super::host_network;
use super::super::host_network::Datagram;
use super::super::host_network::NativeSocket;
use super::super::host_network::Step;
use super::super::network;
use super::super::network::Handle;
use super::super::network::Operation;
use super::super::network::Request;
use super::super::network::Resource;
use super::super::network::TRANSFER_BYTES;
use super::BuiltinForeign;
use super::KernelError;
use super::Machine;
use super::Thunk;
use super::ThunkId;
use super::Value;

enum Answer {
    Connected(Result<(), Failure>),
    Accepted(Result<NativeSocket, Failure>),
    Written(Result<(), Failure>),
    Read(Result<Vec<u8>, Failure>),
    Datagram(Result<Datagram, Failure>),
}

impl Machine<'_> {
    pub(super) fn network_request(
        &mut self,
        builtin: BuiltinForeign,
        arguments: &[ThunkId],
        continuation: ThunkId,
    ) -> Result<Option<ThunkId>, KernelError> {
        let answer = match (builtin, arguments) {
            (BuiltinForeign::TcpListen, [port]) => {
                let port = self.read_u32(*port)?;
                match host_network::listen(port) {
                    Ok(listener) => self.new_network_result(Resource::Listener(listener))?,
                    Err(error) => self.host_failure(error)?,
                }
            }
            (BuiltinForeign::UdpBind, [port]) => {
                let port = self.read_u32(*port)?;
                match host_network::bind(port) {
                    Ok(socket) => self.new_network_result(Resource::Socket(socket))?,
                    Err(error) => self.host_failure(error)?,
                }
            }
            (BuiltinForeign::TcpConnect, [host, port]) => {
                let host = self.read_text(*host)?;
                let port = self.read_u32(*port)?;
                match host_network::connect(&host, port) {
                    Ok(connection) => {
                        let resource = Resource::Socket(connection.socket);
                        if connection.pending {
                            return self.park_network(Request {
                                resource,
                                handle: None,
                                continuation,
                                operation: Operation::Connect,
                            });
                        }
                        self.new_network_result(resource)?
                    }
                    Err(error) => self.host_failure(error)?,
                }
            }
            (BuiltinForeign::TcpAccept, [listener]) => {
                let handle = self.read_network_handle(*listener, true)?;
                let resource = self.network.take(handle)?;
                return self.park_network(Request {
                    resource,
                    handle: Some(handle),
                    continuation,
                    operation: Operation::Accept,
                });
            }
            (BuiltinForeign::TcpRecv | BuiltinForeign::UdpRecvFrom, [socket, max]) => {
                let handle = self.read_network_handle(*socket, false)?;
                let max = self.read_u32(*max)?;
                host_files::read_size(max, TRANSFER_BYTES).map_err(network::host_error)?;
                let resource = self.network.take(handle)?;
                let operation = if builtin == BuiltinForeign::TcpRecv {
                    Operation::Recv { max }
                } else {
                    Operation::RecvFrom { max }
                };
                // IO_READ always parks, even if data or EOF is already ready.
                return self.park_network(Request {
                    resource,
                    handle: Some(handle),
                    continuation,
                    operation,
                });
            }
            (BuiltinForeign::TcpSend, [socket, data]) => {
                let handle = self.read_network_handle(*socket, false)?;
                let data = self.read_text(*data)?;
                let resource = self.network.take(handle)?;
                return self.advance_network(Request {
                    resource,
                    handle: Some(handle),
                    continuation,
                    operation: Operation::Send { data, offset: 0 },
                });
            }
            (BuiltinForeign::UdpSendTo, [socket, host, port, data]) => {
                let handle = self.read_network_handle(*socket, false)?;
                let host = self.read_text(*host)?;
                let port = self.read_u32(*port)?;
                let data = self.read_text(*data)?;
                let resource = self.network.take(handle)?;
                return self.advance_network(Request {
                    resource,
                    handle: Some(handle),
                    continuation,
                    operation: Operation::SendTo { host, port, data },
                });
            }
            (BuiltinForeign::UdpPoll, [socket, max]) => self.poll_network(*socket, *max)?,
            (BuiltinForeign::SocketClose | BuiltinForeign::ListenerClose, [handle]) => {
                let handle =
                    self.read_network_handle(*handle, builtin == BuiltinForeign::ListenerClose)?;
                drop(self.network.take(handle)?);
                self.unit()?
            }
            _ => return Err(KernelError::new("network request has an invalid arity")),
        };
        Ok(Some(
            self.allocate(Thunk::Application(continuation, answer))?,
        ))
    }

    fn park_network(&mut self, request: Request) -> Result<Option<ThunkId>, KernelError> {
        let order = self.scheduler.reserve_wait()?;
        self.network.park(order, request)?;
        Ok(None)
    }

    fn poll_network(&mut self, socket: ThunkId, max: ThunkId) -> Result<ThunkId, KernelError> {
        let handle = self.read_network_handle(socket, false)?;
        let max = self.read_u32(max)?;
        host_files::read_size(max, TRANSFER_BYTES).map_err(network::host_error)?;
        let resource = self.network.take(handle)?;
        let result = host_network::recv_from(resource.socket()?, max, TRANSFER_BYTES);
        let socket = self.restore_network_value(handle, resource)?;
        let result = match result {
            Ok(Step::Wait) => {
                let none = self.host_constructor("None", vec![])?;
                self.host_constructor("Done", vec![none])?
            }
            Ok(Step::Ready(datagram)) => {
                let datagram = self.network_datagram(&datagram)?;
                let some = self.host_constructor("Some", vec![datagram])?;
                self.host_constructor("Done", vec![some])?
            }
            Err(error) => self.host_failure(error)?,
        };
        self.host_constructor("Tuple", vec![socket, result])
    }

    fn send_network(
        &mut self,
        socket: &NativeSocket,
        data: &[u8],
        offset: &mut usize,
    ) -> Result<Option<Answer>, KernelError> {
        loop {
            if *offset == data.len() {
                return Ok(Some(Answer::Written(Ok(()))));
            }
            self.tick()?;
            match host_network::send(socket, &data[*offset..]) {
                Ok(Step::Wait) => return Ok(None),
                Ok(Step::Ready(0)) => {
                    return Err(KernelError::new("native TCP send made no progress"));
                }
                Ok(Step::Ready(count)) => {
                    if count > data.len() - *offset {
                        return Err(KernelError::new(
                            "native TCP send returned an invalid count",
                        ));
                    }
                    *offset += count;
                }
                Err(error) => return Ok(Some(Answer::Written(Err(error)))),
            }
        }
    }

    fn advance_network(&mut self, mut request: Request) -> Result<Option<ThunkId>, KernelError> {
        let answer = match &mut request.operation {
            Operation::Connect => Some(Answer::Connected(host_network::finish_connect(
                request.resource.socket()?,
            ))),
            Operation::Accept => match host_network::accept(request.resource.listener()?) {
                Ok(Step::Wait) => None,
                Ok(Step::Ready(socket)) => Some(Answer::Accepted(Ok(socket))),
                Err(error) => Some(Answer::Accepted(Err(error))),
            },
            Operation::Send { data, offset } => {
                self.send_network(request.resource.socket()?, data, offset)?
            }
            Operation::Recv { max } => {
                match host_network::recv(request.resource.socket()?, *max, TRANSFER_BYTES) {
                    Ok(Step::Wait) => None,
                    Ok(Step::Ready(data)) => Some(Answer::Read(Ok(data))),
                    Err(error) => Some(Answer::Read(Err(error))),
                }
            }
            Operation::SendTo { host, port, data } => {
                match host_network::send_to(request.resource.socket()?, host, *port, data) {
                    Ok(Step::Wait) => None,
                    Ok(Step::Ready(())) => Some(Answer::Written(Ok(()))),
                    Err(error) => Some(Answer::Written(Err(error))),
                }
            }
            Operation::RecvFrom { max } => {
                match host_network::recv_from(request.resource.socket()?, *max, TRANSFER_BYTES) {
                    Ok(Step::Wait) => None,
                    Ok(Step::Ready(data)) => Some(Answer::Datagram(Ok(data))),
                    Err(error) => Some(Answer::Datagram(Err(error))),
                }
            }
        };
        let Some(answer) = answer else {
            return self.park_network(request);
        };
        let result = match answer {
            Answer::Connected(result) => {
                let result = match result {
                    Ok(()) => self.new_network_result(request.resource)?,
                    Err(error) => self.host_failure(error)?,
                };
                return Ok(Some(
                    self.allocate(Thunk::Application(request.continuation, result))?,
                ));
            }
            Answer::Accepted(result) => match result {
                Ok(socket) => self.new_network_result(Resource::Socket(socket))?,
                Err(error) => self.host_failure(error)?,
            },
            Answer::Written(result) => match result {
                Ok(()) => {
                    let unit = self.unit()?;
                    self.host_constructor("Done", vec![unit])?
                }
                Err(error) => self.host_failure(error)?,
            },
            Answer::Read(result) => match result {
                Ok(data) => {
                    let text = self.network_text(&data)?;
                    self.host_constructor("Done", vec![text])?
                }
                Err(error) => self.host_failure(error)?,
            },
            Answer::Datagram(result) => match result {
                Ok(data) => {
                    let value = self.network_datagram(&data)?;
                    self.host_constructor("Done", vec![value])?
                }
                Err(error) => self.host_failure(error)?,
            },
        };
        let handle = request
            .handle
            .ok_or_else(|| KernelError::new("native network response lost its handle"))?;
        let handle = self.restore_network_value(handle, request.resource)?;
        let answer = self.host_constructor("Tuple", vec![handle, result])?;
        Ok(Some(self.allocate(Thunk::Application(
            request.continuation,
            answer,
        ))?))
    }

    pub(super) fn resume_network(&mut self, orders: &[u64], now: u64) -> Result<(), KernelError> {
        for order in orders {
            // Upstream traverses one mixed parked list after collecting workers.
            self.scheduler.wake_before(now, *order);
            let request = self.network.take_pending(*order)?;
            let continuation = request.continuation;
            self.tick()?;
            let action =
                self.with_roots(&[continuation], |machine| machine.advance_network(request))?;
            if let Some(action) = action {
                self.scheduler.resume(action)?;
            }
        }
        self.scheduler.wake(now);
        Ok(())
    }

    fn read_network_handle(
        &mut self,
        value: ThunkId,
        listener: bool,
    ) -> Result<Handle, KernelError> {
        match (self.force(value)?, listener) {
            (Value::Listener(handle), true) | (Value::Socket(handle), false) => Ok(handle),
            _ => Err(KernelError::new(
                "expected a sealed native socket/listener handle",
            )),
        }
    }

    fn new_network_result(&mut self, resource: Resource) -> Result<ThunkId, KernelError> {
        let listener = matches!(resource, Resource::Listener(_));
        let handle = self.network.insert(resource)?;
        let value = if listener {
            Value::Listener(handle)
        } else {
            Value::Socket(handle)
        };
        let value = self.allocate(Thunk::Ready(value))?;
        self.host_constructor("Done", vec![value])
    }

    fn restore_network_value(
        &mut self,
        handle: Handle,
        resource: Resource,
    ) -> Result<ThunkId, KernelError> {
        let listener = matches!(resource, Resource::Listener(_));
        self.network.restore(handle, resource)?;
        self.allocate(Thunk::Ready(if listener {
            Value::Listener(handle)
        } else {
            Value::Socket(handle)
        }))
    }

    fn network_text(&mut self, bytes: &[u8]) -> Result<ThunkId, KernelError> {
        let points =
            host_files::decode_native_text(bytes, TRANSFER_BYTES).map_err(network::host_error)?;
        self.host_text(&points)
    }

    fn network_datagram(&mut self, data: &Datagram) -> Result<ThunkId, KernelError> {
        let host = self.network_text(&data.host)?;
        let port = self.host_word(data.port)?;
        let text = self.network_text(&data.data)?;
        let tail = self.host_constructor("Tuple", vec![port, text])?;
        self.host_constructor("Tuple", vec![host, tail])
    }
}
