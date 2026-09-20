// SPDX-License-Identifier: MPL-2.0
//! Monotonic nanosecond clocks. IO.now keeps the operating system's origin;
//! starting a Bend invocation never resets the clock or narrows it to U32.

use crate::kernel::KernelError;
use std::time::Duration;

pub(super) const NANOS_PER_MILLI: u64 = 1_000_000;
pub(super) const MAX_WAIT_NANOS: u64 = 100_000_000;

pub(super) trait Clock {
    fn now(&mut self) -> Result<u64, KernelError>;
    fn wait(&mut self, nanoseconds: u64) -> Result<(), KernelError>;
}

pub(super) struct SystemClock;

impl Clock for SystemClock {
    fn now(&mut self) -> Result<u64, KernelError> {
        monotonic_nanoseconds()
    }

    fn wait(&mut self, nanoseconds: u64) -> Result<(), KernelError> {
        // The driver checks cancellation before and after each bounded wait.
        std::thread::sleep(Duration::from_nanos(nanoseconds.min(MAX_WAIT_NANOS)));
        Ok(())
    }
}

#[cfg(windows)]
fn monotonic_nanoseconds() -> Result<u64, KernelError> {
    use windows::Win32::System::Performance::QueryPerformanceCounter;
    use windows::Win32::System::Performance::QueryPerformanceFrequency;

    let mut count = 0;
    let mut frequency = 0;
    // SAFETY: each call writes to its own live, aligned i64 output slot.
    unsafe { QueryPerformanceCounter(&raw mut count) }
        .map_err(|error| KernelError::new(format!("cannot read monotonic clock: {error}")))?;
    // SAFETY: frequency is a live, aligned i64 output slot.
    unsafe { QueryPerformanceFrequency(&raw mut frequency) }
        .map_err(|error| KernelError::new(format!("cannot read monotonic frequency: {error}")))?;
    ticks_to_nanoseconds(count, frequency)
}

#[cfg(any(windows, test))]
fn ticks_to_nanoseconds(count: i64, frequency: i64) -> Result<u64, KernelError> {
    let count = u128::try_from(count)
        .map_err(|_error| KernelError::new("monotonic clock returned a negative count"))?;
    let frequency = u128::try_from(frequency)
        .ok()
        .filter(|frequency| *frequency > 0)
        .ok_or_else(|| KernelError::new("monotonic clock returned an invalid frequency"))?;
    u64::try_from(count * 1_000_000_000 / frequency)
        .map_err(|_error| KernelError::new("monotonic clock exceeds the native nanosecond range"))
}

#[cfg(unix)]
fn monotonic_nanoseconds() -> Result<u64, KernelError> {
    let mut stamp = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: clock_gettime initializes the live timespec output on success.
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &raw mut stamp) } != 0 {
        return Err(KernelError::new(format!(
            "cannot read monotonic clock: {}",
            std::io::Error::last_os_error()
        )));
    }
    let seconds = u64::try_from(stamp.tv_sec)
        .map_err(|_error| KernelError::new("monotonic clock returned negative seconds"))?;
    let nanos = u64::try_from(stamp.tv_nsec)
        .ok()
        .filter(|nanos| *nanos < 1_000_000_000)
        .ok_or_else(|| KernelError::new("monotonic clock returned invalid nanoseconds"))?;
    seconds
        .checked_mul(1_000_000_000)
        .and_then(|value| value.checked_add(nanos))
        .ok_or_else(|| KernelError::new("monotonic clock exceeds the native nanosecond range"))
}

#[cfg(not(any(windows, unix)))]
fn monotonic_nanoseconds() -> Result<u64, KernelError> {
    Err(KernelError::new(
        "native monotonic clock is unavailable on this platform",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_scaling_preserves_large_values_without_rebasing_or_rounding_up() {
        assert_eq!(ticks_to_nanoseconds(1, 3).unwrap(), 333_333_333);
        assert_eq!(
            ticks_to_nanoseconds(50_000_000_000_007, 10_000_000).unwrap(),
            5_000_000_000_000_700
        );
        ticks_to_nanoseconds(-1, 1).expect_err("negative counter");
        ticks_to_nanoseconds(1, 0).expect_err("zero frequency");
        ticks_to_nanoseconds(1, -1).expect_err("negative frequency");
        ticks_to_nanoseconds(i64::MAX, 1).expect_err("nanosecond overflow");
    }

    #[test]
    fn system_clock_is_monotonic_across_separate_adapters() {
        let first = SystemClock.now().unwrap();
        let second = SystemClock.now().unwrap();
        assert!(second >= first);
    }
}
