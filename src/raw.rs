use std::os::raw::c_long;
use uapi::c::{syscall, SYS_clone3};

/// Arguments to the clone3 system call as defined in `/usr/include/linux/sched.h`.
#[repr(C, align(8))]
#[derive(Debug, Default)]
pub struct CloneArgs {
    pub flags: u64,
    pub pidfd: u64,
    pub child_tid: u64,
    pub parent_tid: u64,
    pub exit_signal: u64,
    pub stack: u64,
    pub stack_size: u64,
    pub tls: u64,
    #[cfg(feature = "linux_5-5")]
    pub set_tid: u64,
    #[cfg(feature = "linux_5-5")]
    pub set_tid_size: u64,
    #[cfg(feature = "linux_5-7")]
    pub cgroup: u64,
}

/// The raw clone3 system call.
pub unsafe fn clone3_system_call(cl_args: &CloneArgs) -> c_long {
    syscall(
        SYS_clone3,
        cl_args as *const CloneArgs,
        core::mem::size_of::<CloneArgs>(),
    )
}
