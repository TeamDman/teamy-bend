// SPDX-License-Identifier: MPL-2.0
//! Bounded host workers. Only owned host data crosses the thread boundary;
//! continuations and all Bend values remain on the VM thread.

use super::ARENA_LIMIT;
use super::ThunkId;
use super::file_handles::Handle;
use super::host_files;
use super::host_files::Failure;
use super::host_files::NativeFile;
use super::host_network::NativeSocket;
use super::host_network::WakePair;
use super::host_network::WakeSender;
use crate::kernel::KernelError;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::time::Duration;

pub(super) const JOB_BYTES: usize = 8 * 1024 * 1024;
const RETAINED_BYTES: usize = 64 * 1024 * 1024;
const WORKERS: usize = 64; // Upstream native IO_HELP.

pub(super) enum Task {
    Open {
        path: Vec<u8>,
        mode: Vec<u8>,
    },
    Read {
        file: NativeFile,
        max: u32,
        bytes: bool,
    },
    Write {
        file: NativeFile,
        data: Vec<u8>,
    },
    #[cfg(test)]
    Test(Box<dyn FnOnce() -> Reply + Send>),
}

pub(super) enum Reply {
    Open(Result<NativeFile, Failure>),
    Read {
        file: NativeFile,
        data: Result<Vec<u8>, Failure>,
        bytes: bool,
    },
    Write {
        file: NativeFile,
        result: Result<(), Failure>,
    },
    Panicked,
}

impl Task {
    fn run(self) -> Reply {
        match self {
            Self::Open { path, mode } => Reply::Open(host_files::open(&path, &mode)),
            Self::Read {
                mut file,
                max,
                bytes,
            } => {
                let data = host_files::read(&mut file, max, JOB_BYTES);
                Reply::Read { file, data, bytes }
            }
            Self::Write { mut file, data } => {
                let result = host_files::write(&mut file, &data, JOB_BYTES);
                Reply::Write { file, result }
            }
            #[cfg(test)]
            Self::Test(run) => run(),
        }
    }
}

#[derive(Default)]
struct Budget {
    jobs: AtomicUsize,
    bytes: AtomicUsize,
}

struct Lease {
    budget: Arc<Budget>,
    bytes: usize,
}

impl Budget {
    fn reserve(self: &Arc<Self>, bytes: usize) -> Result<Lease, KernelError> {
        self.jobs
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < ARENA_LIMIT).then_some(count + 1)
            })
            .map_err(|_error| KernelError::new("native host job budget exhausted"))?;
        if self
            .bytes
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |used| {
                used.checked_add(bytes).filter(|sum| *sum <= RETAINED_BYTES)
            })
            .is_err()
        {
            self.jobs.fetch_sub(1, Ordering::AcqRel);
            return Err(KernelError::new("native host buffer budget exhausted"));
        }
        Ok(Lease {
            budget: Arc::clone(self),
            bytes,
        })
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.budget.bytes.fetch_sub(self.bytes, Ordering::AcqRel);
        self.budget.jobs.fetch_sub(1, Ordering::AcqRel);
    }
}

struct Job {
    id: u64,
    task: Task,
    sender: mpsc::Sender<Finished>,
    cancelled: Arc<AtomicBool>,
    lease: Lease,
    notifier: Arc<Notifier>,
}

#[derive(Default)]
struct Notifier {
    sender: Mutex<Option<Arc<WakeSender>>>,
}

impl Notifier {
    fn notify(&self) {
        if let Ok(sender) = self.sender.lock()
            && let Some(sender) = sender.as_ref()
        {
            sender.notify();
        }
    }
}

struct Finished {
    id: u64,
    reply: Reply,
    lease: Lease,
}

#[derive(Default)]
struct Queue {
    jobs: VecDeque<Job>,
    workers: usize,
    busy: usize,
}

#[derive(Default)]
struct Pool {
    queue: Mutex<Queue>,
    wake: Condvar,
    budget: Arc<Budget>,
}

static POOL: OnceLock<Arc<Pool>> = OnceLock::new();

fn pool() -> &'static Arc<Pool> {
    POOL.get_or_init(|| Arc::new(Pool::default()))
}

impl Pool {
    fn submit(self: &Arc<Self>, job: Job) -> Result<(), KernelError> {
        let mut queue = self
            .queue
            .lock()
            .map_err(|_error| KernelError::new("native host queue poisoned"))?;
        if queue.workers < WORKERS && queue.jobs.len() + queue.busy >= queue.workers {
            let worker = Arc::clone(self);
            std::thread::Builder::new()
                .name("bend-io".into())
                .spawn(move || worker.run())
                .map_err(|error| {
                    KernelError::new(format!("cannot start native host worker: {error}"))
                })?;
            queue.workers += 1;
        }
        queue.jobs.push_back(job);
        self.wake.notify_one();
        Ok(())
    }

    fn run(&self) {
        loop {
            let job = {
                let Ok(mut queue) = self.queue.lock() else {
                    return;
                };
                while queue.jobs.is_empty() {
                    let Ok(waited) = self.wake.wait(queue) else {
                        return;
                    };
                    queue = waited;
                }
                let Some(job) = queue.jobs.pop_front() else {
                    continue;
                };
                queue.busy += 1;
                job
            };
            if !job.cancelled.load(Ordering::Acquire) {
                let reply = std::panic::catch_unwind(AssertUnwindSafe(|| job.task.run()))
                    .unwrap_or(Reply::Panicked);
                // On cancellation, dropping this answer closes any returned
                // file. A worker never accesses a VM arena or resumes a task.
                if !job.cancelled.load(Ordering::Acquire)
                    && job
                        .sender
                        .send(Finished {
                            id: job.id,
                            reply,
                            lease: job.lease,
                        })
                        .is_ok()
                {
                    job.notifier.notify();
                }
            }
            let Ok(mut queue) = self.queue.lock() else {
                return;
            };
            queue.busy -= 1;
        }
    }

    fn cancel(&self, cancelled: &Arc<AtomicBool>) {
        if let Ok(mut queue) = self.queue.lock() {
            queue
                .jobs
                .retain(|job| !Arc::ptr_eq(&job.cancelled, cancelled));
        }
    }
}

struct Portal {
    sender: mpsc::Sender<Finished>,
    receiver: mpsc::Receiver<Finished>,
    cancelled: Arc<AtomicBool>,
    notifier: Arc<Notifier>,
    wake: Option<WakePair>,
}

impl Default for Portal {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            cancelled: Arc::new(AtomicBool::new(false)),
            notifier: Arc::new(Notifier::default()),
            wake: None,
        }
    }
}

struct Pending {
    continuation: ThunkId,
    handle: Option<Handle>,
}

pub(super) struct Completion {
    pub(super) continuation: ThunkId,
    pub(super) handle: Option<Handle>,
    pub(super) reply: Reply,
    // Hold the reservation until the VM finishes packing or dropping the reply.
    _lease: Lease,
}

#[derive(Default)]
pub(super) struct State {
    portal: Option<Portal>,
    pending: BTreeMap<u64, Pending>,
    ready: VecDeque<Finished>,
    next: u64,
}

impl State {
    pub(super) fn submit(
        &mut self,
        task: Task,
        continuation: ThunkId,
        handle: Option<Handle>,
        bytes: usize,
    ) -> Result<(), KernelError> {
        if self.pending.len() >= ARENA_LIMIT {
            return Err(KernelError::new("native pending host job budget exhausted"));
        }
        let id = self.next;
        self.next = id
            .checked_add(1)
            .ok_or_else(|| KernelError::new("native host job identity exhausted"))?;
        let pool = pool();
        let lease = pool.budget.reserve(bytes)?;
        let portal = self.portal.get_or_insert_with(Portal::default);
        pool.submit(Job {
            id,
            task,
            sender: portal.sender.clone(),
            cancelled: Arc::clone(&portal.cancelled),
            lease,
            notifier: Arc::clone(&portal.notifier),
        })?;
        self.pending.insert(
            id,
            Pending {
                continuation,
                handle,
            },
        );
        Ok(())
    }

    pub(super) fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    pub(super) fn completed(&mut self) -> Result<Option<Completion>, KernelError> {
        let Some(portal) = &self.portal else {
            return Ok(None);
        };
        let finished = if let Some(finished) = self.ready.pop_front() {
            finished
        } else {
            match portal.receiver.try_recv() {
                Ok(finished) => finished,
                Err(mpsc::TryRecvError::Empty) => return Ok(None),
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(KernelError::new(
                        "native host completion channel disconnected",
                    ));
                }
            }
        };
        let pending = self
            .pending
            .remove(&finished.id)
            .ok_or_else(|| KernelError::new("unexpected native host completion"))?;
        Ok(Some(Completion {
            continuation: pending.continuation,
            handle: pending.handle,
            reply: finished.reply,
            _lease: finished.lease,
        }))
    }

    pub(super) fn wait(&mut self, nanoseconds: u64) -> Result<(), KernelError> {
        if !self.ready.is_empty() {
            return Ok(());
        }
        let Some(portal) = &self.portal else {
            return Err(KernelError::new("native host wait without pending work"));
        };
        match portal
            .receiver
            .recv_timeout(Duration::from_nanos(nanoseconds))
        {
            Ok(finished) => self.ready.push_back(finished),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(KernelError::new(
                    "native host completion channel disconnected",
                ));
            }
        }
        Ok(())
    }

    /// Install before collecting completed jobs and entering the descriptor
    /// poller. Already-running workers share this notifier, so no wake is lost.
    pub(super) fn enable_network_wake(&mut self) -> Result<(), KernelError> {
        let Some(portal) = &mut self.portal else {
            return Ok(());
        };
        if portal.wake.is_none() {
            let wake = WakePair::new().map_err(super::network::host_error)?;
            *portal
                .notifier
                .sender
                .lock()
                .map_err(|_error| KernelError::new("native worker notifier poisoned"))? =
                Some(Arc::clone(&wake.sender));
            portal.wake = Some(wake);
        }
        Ok(())
    }

    pub(super) fn wake_source(&self) -> Option<&NativeSocket> {
        self.portal.as_ref()?.wake.as_ref().map(|wake| &wake.reader)
    }

    pub(super) fn drain_network_wake(&self) -> Result<(), KernelError> {
        if let Some(wake) = self.portal.as_ref().and_then(|portal| portal.wake.as_ref()) {
            wake.drain().map_err(super::network::host_error)?;
        }
        Ok(())
    }

    pub(super) fn visit_roots(
        &self,
        mut visit: impl FnMut(ThunkId) -> Result<(), KernelError>,
    ) -> Result<(), KernelError> {
        for pending in self.pending.values() {
            visit(pending.continuation)?;
        }
        Ok(())
    }
}

impl Drop for State {
    fn drop(&mut self) {
        if let Some(portal) = &self.portal {
            portal.cancelled.store(true, Ordering::Release);
            if let Some(pool) = POOL.get() {
                pool.cancel(&portal.cancelled);
            }
        }
    }
}

#[cfg(test)]
mod tests;
