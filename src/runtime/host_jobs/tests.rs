// SPDX-License-Identifier: MPL-2.0
use super::host_files::Error;
use super::*;

fn missing() -> Reply {
    Reply::Open(Err(Failure::Io(Error {
        code: 2,
        message: b"missing".to_vec(),
    })))
}

#[test]
fn live_worker_retains_continuation_until_completion_is_consumed() {
    let (started, start) = mpsc::channel();
    let (release, wait) = mpsc::channel();
    let mut jobs = State::default();
    jobs.submit(
        Task::Test(Box::new(move || {
            started.send(()).unwrap();
            wait.recv().unwrap();
            missing()
        })),
        42,
        None,
        7,
    )
    .unwrap();
    start.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(jobs.has_pending());
    assert!(jobs.completed().unwrap().is_none());
    let mut roots = Vec::new();
    jobs.visit_roots(|id| {
        roots.push(id);
        Ok(())
    })
    .unwrap();
    assert_eq!(roots, [42]);
    assert!(
        jobs.visit_roots(|_| Err(KernelError::new("cancelled")))
            .is_err()
    );
    release.send(()).unwrap();
    jobs.wait(5_000_000_000).unwrap();
    let completion = jobs.completed().unwrap().unwrap();
    assert_eq!(completion.continuation, 42);
    assert!(matches!(completion.reply, Reply::Open(Err(Failure::Io(_)))));
    assert!(!jobs.has_pending());
}

#[test]
fn dropping_an_invocation_does_not_wait_for_an_already_running_host_call() {
    let (started, start) = mpsc::channel();
    let (release, wait) = mpsc::channel();
    let (finished, finish) = mpsc::channel();
    let mut jobs = State::default();
    jobs.submit(
        Task::Test(Box::new(move || {
            started.send(()).unwrap();
            wait.recv().unwrap();
            finished.send(()).unwrap();
            missing()
        })),
        42,
        None,
        0,
    )
    .unwrap();
    start.recv_timeout(Duration::from_secs(5)).unwrap();
    drop(jobs);
    // Reaching this send demonstrates that teardown did not join the blocked
    // worker. Its owned reply is discarded after the call finishes.
    release.send(()).unwrap();
    finish.recv_timeout(Duration::from_secs(5)).unwrap();
}

#[test]
fn cancellation_removes_only_its_queued_jobs_and_releases_reservations() {
    let pool = Pool::default();
    let cancelled = Arc::new(AtomicBool::new(true));
    let other = Arc::new(AtomicBool::new(false));
    let (sender, _receiver) = mpsc::channel();
    for token in [&cancelled, &other] {
        pool.queue.lock().unwrap().jobs.push_back(Job {
            id: 0,
            task: Task::Test(Box::new(missing)),
            sender: sender.clone(),
            cancelled: Arc::clone(token),
            lease: pool.budget.reserve(10).unwrap(),
            notifier: Arc::new(Notifier::default()),
        });
    }
    assert_eq!(pool.budget.jobs.load(Ordering::Acquire), 2);
    pool.cancel(&cancelled);
    assert_eq!(pool.queue.lock().unwrap().jobs.len(), 1);
    assert_eq!(pool.budget.jobs.load(Ordering::Acquire), 1);
    assert_eq!(pool.budget.bytes.load(Ordering::Acquire), 10);
    pool.cancel(&other);
    assert_eq!(pool.budget.jobs.load(Ordering::Acquire), 0);
    assert_eq!(pool.budget.bytes.load(Ordering::Acquire), 0);
}

#[test]
fn aggregate_memory_and_job_limits_are_independent_and_reusable() {
    let budget = Arc::new(Budget::default());
    let lease = budget.reserve(RETAINED_BYTES).unwrap();
    assert!(budget.reserve(1).is_err());
    assert_eq!(budget.jobs.load(Ordering::Acquire), 1);
    let empty = budget.reserve(0).unwrap();
    drop(lease);
    assert_eq!(budget.bytes.load(Ordering::Acquire), 0);
    drop(empty);
    budget.jobs.store(ARENA_LIMIT, Ordering::Release);
    assert!(budget.reserve(0).is_err());
    budget.jobs.store(0, Ordering::Release);
    let lease = budget.reserve(1).unwrap();
    drop(lease);
    assert_eq!(budget.jobs.load(Ordering::Acquire), 0);
}
