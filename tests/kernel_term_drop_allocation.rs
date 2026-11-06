// SPDX-License-Identifier: MPL-2.0
//! Isolated allocation-failure tests for automatic term cleanup.
#![cfg(target_os = "linux")]
use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::alloc::System;
use std::os::unix::process::ExitStatusExt;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use teamy_bend::kernel::Term;
use teamy_bend::kernel::term;

struct FaultAllocator;
static FAIL_AT: AtomicUsize = AtomicUsize::new(0);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

fn fail_this_allocation() -> bool {
    let target = FAIL_AT.load(Ordering::SeqCst);
    target != 0 && ALLOCATIONS.fetch_add(1, Ordering::SeqCst) + 1 == target
}

// SAFETY: non-injected operations forward their original arguments to System;
// injected allocation failures return null without accessing those arguments.
unsafe impl GlobalAlloc for FaultAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if fail_this_allocation() {
            std::ptr::null_mut()
        } else {
            // SAFETY: the layout is the one received from the caller.
            unsafe { System.alloc(layout) }
        }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: the pointer and layout are the ones received from the caller.
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if fail_this_allocation() {
            std::ptr::null_mut()
        } else {
            // SAFETY: arguments are forwarded unchanged from the caller.
            unsafe { System.realloc(ptr, layout, new_size) }
        }
    }
}

#[global_allocator]
static ALLOCATOR: FaultAllocator = FaultAllocator;

struct ArmOnUnwind(usize);
impl Drop for ArmOnUnwind {
    fn drop(&mut self) {
        FAIL_AT.store(self.0, Ordering::SeqCst);
    }
}

#[test]
fn allocation_failure_terminates_even_during_unwind() {
    if let Ok(mode) = std::env::var("TEAMY_BEND_DROP_ALLOC_FAIL") {
        let no_core = libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        // SAFETY: `no_core` is a valid rlimit for this child process.
        let result = unsafe { libc::setrlimit(libc::RLIMIT_CORE, &raw const no_core) };
        assert_eq!(result, 0, "disable core dumps in the fault child");
        let (index, unwind) = match mode.as_str() {
            "placeholder" => (1, false),
            "worklist" => (2, false),
            "unwind-placeholder" => (1, true),
            "unwind-worklist" => (2, true),
            _ => panic!("invalid fault mode"),
        };
        std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(move || {
                let root = (0..10_000).fold(term(Term::Rfl), |child, _| term(Term::Typ(child)));
                eprintln!("fault armed: {mode}");
                if unwind {
                    let _root = root;
                    let _arm = ArmOnUnwind(index);
                    panic!("start unwind before arming allocation failure");
                }
                FAIL_AT.store(index, Ordering::SeqCst);
                drop(root);
            })
            .unwrap()
            .join()
            .unwrap();
        panic!("allocation failure did not terminate the child");
    }
    for mode in [
        "placeholder",
        "worklist",
        "unwind-placeholder",
        "unwind-worklist",
    ] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "allocation_failure_terminates_even_during_unwind",
                "--nocapture",
            ])
            .env("TEAMY_BEND_DROP_ALLOC_FAIL", mode)
            .output()
            .unwrap();
        assert_eq!(output.status.signal(), Some(6), "{mode}: {output:?}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("fault armed: {mode}")));
        if mode.starts_with("unwind-") {
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains("start unwind before arming allocation failure")
            );
        }
    }
}
