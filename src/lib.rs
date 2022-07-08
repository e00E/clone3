//! Bindings to the `clone3` linux system call.
//!
//! See the [clone3 man page](https://man7.org/linux/man-pages/man2/clone3.2.html) for more
//! information.
//!
//! This is a complex and generally unsafe operation. Users must understand the documentation to
//! use it safely and correctly.
//!
//! # Examples
//!
//! ```
//! use clone3::Clone3;
//!
//! let mut pidfd = -1;
//! let mut clone3 = Clone3::default();
//! clone3.flag_pidfd(&mut pidfd);
//!
//! match unsafe { clone3.call() }.unwrap() {
//!     0 => println!("i am the child"),
//!     child => println!("i am the parent, my child has pid {} and my pidfd is {}", child, pidfd),
//! }
//! ```
//!
//! # Features
//!
//! The clone3 api can cahnge in a backward compatible manner between Linux kernel versions. This
//! crates supports picking the target api through the Cargo features `linux_5-5`, `linux_5-7`.
//! The default is the newest kernel version. Specifying no `linux` feature corresponds to the
//! initial clone3 api.

#![doc(html_root_url = "https://docs.rs/clone3/0.2.2")]
#![allow(clippy::missing_safety_doc)]

mod raw;
mod wrapper;

pub use crate::wrapper::*;
pub use raw::*;

// The libc crate does not include some of the newer constants so define all of them.
bitflags::bitflags! {
    /// Flags for the clone3 system call as defined in `/usr/include/linux/sched.h`.
    #[derive(Default)]
    pub struct Flags: u64 {
        const CHILD_CLEARTID = 0x00200000;
        const CHILD_SETTID = 0x01000000;
        #[cfg(feature = "linux_5-5")]
        const CLEAR_SIGHAND = 0x100000000;
        const FILES = 0x00000400;
        const FS = 0x00000200;
        #[cfg(feature = "linux_5-7")]
        const INTO_CGROUP = 0x200000000;
        const IO = 0x80000000;
        const NEWCGROUP = 0x02000000;
        const NEWIPC = 0x08000000;
        const NEWNET = 0x40000000;
        const NEWNS = 0x00020000;
        const NEWPID = 0x20000000;
        const NEWTIME = 0x00000080;
        const NEWUSER = 0x10000000;
        const NEWUTS = 0x04000000;
        const PARENT = 0x00008000;
        const PARENT_SETTID = 0x00100000;
        const PIDFD = 0x00001000;
        const PTRACE = 0x00002000;
        const SETTLS = 0x00080000;
        const SIGHAND = 0x00000800;
        const SYSVSEM = 0x00040000;
        const THREAD = 0x00010000;
        const UNTRACED = 0x00800000;
        const VFORK = 0x00004000;
        const VM = 0x00000100;
    }
}
