use crate::{CloneArgs, Flags};
#[cfg(feature = "linux_5-7")]
use std::os::unix::io::AsRawFd;
use std::{
    convert::TryInto,
    os::{raw::c_long, unix::io::RawFd},
};
use uapi::{c::pid_t, Errno};

/// Higher level wrapper around the clone3 system call.
///
/// Construct it with `Clone3::default()` which sets no flags and no exit signal. Use builder
/// methods to customize the underlying [`CloneArgs`](crate::CloneArgs). Perform the system call
/// with [`call`](Self::call).
#[derive(Default)]
pub struct Clone3<'a> {
    flags: Flags,
    pidfd: Option<&'a mut RawFd>,
    child_tid: Option<&'a mut pid_t>,
    parent_tid: Option<&'a mut pid_t>,
    exit_signal: u64,
    stack: Option<&'a mut [u8]>,
    tls: Option<u64>,
    #[cfg(feature = "linux_5-5")]
    set_tid: Option<&'a [pid_t]>,
    #[cfg(feature = "linux_5-7")]
    cgroup: Option<&'a dyn AsRawFd>,
}

impl<'a> Clone3<'a> {
    pub fn flag_child_cleartid(&mut self, child_tid: &'a mut pid_t) -> &mut Self {
        self.flags.set(Flags::CHILD_CLEARTID, true);
        self.child_tid = Some(child_tid);
        self
    }

    #[cfg(feature = "linux_5-5")]
    pub fn flag_child_settid(&mut self, child_tid: &'a mut pid_t) -> &mut Self {
        self.flags.set(Flags::CHILD_SETTID, true);
        self.child_tid = Some(child_tid);
        self
    }

    #[cfg(feature = "linux_5-5")]
    pub fn flag_clear_sighand(&mut self) -> &mut Self {
        self.flags.set(Flags::CLEAR_SIGHAND, true);
        self
    }

    pub fn flag_files(&mut self) -> &mut Self {
        self.flags.set(Flags::FILES, true);
        self
    }

    pub fn flag_fs(&mut self) -> &mut Self {
        self.flags.set(Flags::FS, true);
        self
    }

    #[cfg(feature = "Linux 5.7")]
    pub fn flag_into_cgroup(&mut self, cgroup: &'a dyn AsRawFd) -> &mut Self {
        self.flags.set(Flags::INTO_CGROUP, true);
        self.cgroup = Some(cgroup);
        self
    }

    pub fn flag_io(&mut self) -> &mut Self {
        self.flags.set(Flags::IO, true);
        self
    }

    pub fn flag_newcgroup(&mut self) -> &mut Self {
        self.flags.set(Flags::NEWCGROUP, true);
        self
    }

    pub fn flag_newipc(&mut self) -> &mut Self {
        self.flags.set(Flags::NEWIPC, true);
        self
    }

    pub fn flag_newnet(&mut self) -> &mut Self {
        self.flags.set(Flags::NEWNET, true);
        self
    }

    pub fn flag_newns(&mut self) -> &mut Self {
        self.flags.set(Flags::NEWNS, true);
        self
    }

    pub fn flag_newpid(&mut self) -> &mut Self {
        self.flags.set(Flags::NEWPID, true);
        self
    }

    pub fn flag_newtime(&mut self) -> &mut Self {
        self.flags.set(Flags::NEWTIME, true);
        self
    }

    pub fn flag_newuser(&mut self) -> &mut Self {
        self.flags.set(Flags::NEWUSER, true);
        self
    }

    pub fn flag_newuts(&mut self) -> &mut Self {
        self.flags.set(Flags::NEWUTS, true);
        self
    }

    pub fn flag_parent(&mut self) -> &mut Self {
        self.flags.set(Flags::PARENT, true);
        self
    }

    pub fn flag_parent_settid(&mut self, parent_tid: &'a mut pid_t) -> &mut Self {
        self.flags.set(Flags::PARENT_SETTID, true);
        self.parent_tid = Some(parent_tid);
        self
    }

    pub fn flag_pidfd(&mut self, pidfd: &'a mut RawFd) -> &mut Self {
        self.flags.set(Flags::PIDFD, true);
        self.pidfd = Some(pidfd);
        self
    }

    pub fn flag_ptrace(&mut self) -> &mut Self {
        self.flags.set(Flags::PTRACE, true);
        self
    }

    pub fn flag_settls(&mut self, tls: u64) -> &mut Self {
        self.flags.set(Flags::SETTLS, true);
        self.tls = Some(tls);
        self
    }

    pub fn flag_sighand(&mut self) -> &mut Self {
        self.flags.set(Flags::SIGHAND, true);
        self
    }

    pub fn flag_sysvsem(&mut self) -> &mut Self {
        self.flags.set(Flags::SYSVSEM, true);
        self
    }

    pub fn flag_thread(&mut self) -> &mut Self {
        self.flags.set(Flags::THREAD, true);
        self
    }

    pub fn flag_untraced(&mut self) -> &mut Self {
        self.flags.set(Flags::UNTRACED, true);
        self
    }

    pub fn flag_vfork(&mut self) -> &mut Self {
        self.flags.set(Flags::VFORK, true);
        self
    }

    pub fn flag_vm(&mut self, stack: &'a mut [u8]) -> &mut Self {
        self.flags.set(Flags::VM, true);
        self.stack(stack);
        self
    }

    pub fn exit_signal(&mut self, exit_signal: u64) -> &mut Self {
        self.exit_signal = exit_signal;
        self
    }

    pub fn stack(&mut self, stack: &'a mut [u8]) -> &mut Self {
        self.stack = Some(stack);
        self
    }

    #[cfg(feature = "linux_5-5")]
    pub fn set_tid(&mut self, set_tid: &'a [pid_t]) -> &mut Self {
        self.set_tid = Some(set_tid);
        self
    }

    /// Performs the system call.
    ///
    /// # Errors
    ///
    /// Errors if the system call returns -1.
    ///
    /// # Panics
    ///
    /// Panics if the set flags are incompatible which is a user error:
    /// * `CHILD_CLEARTID` and `CHILD_SETTID` must not be set together
    /// * `CLEAR_SIGHAND` and `SIGHAND` must not be set together
    /// * `INTO_CGROUP` and `NEWCGROUP` must not be set together
    /// * `NEWIPC` and `SYSVSEM` must not be set together
    /// * `FS` and `NEWNS` must not be set together
    /// * `THREAD` and `PIDFD` must not be set together
    /// * `NEWPID` must not be set with `PARENT` or `THREAD`
    /// * `NEWUSER` must not be set with `FS` or `PARENT` or `THREAD`
    /// * if `SIGHAND` is set then `VM` must be set
    /// * if `THREAD` is set then `SIGHAND` must be set
    //
    // For the last two conditions we could automatically set the other required but I prefer the
    // explicitness of forcing the user to set them.
    ///
    /// Panics if the system call retrurns a value that neither indicates failure nor is convertible
    /// to [`pid_t`](pid_t) which  could happen on overflow due to different type sizes. This is a
    /// bug in the Linux kernel or the libc bindings used by this crate.
    pub unsafe fn call(&mut self) -> Result<pid_t, Errno> {
        if let Some(reason) = find_incompatible_flags(self.flags) {
            panic!("flags {:?} are inconsistent: {}", self.flags, reason);
        }
        let return_value = self.call_unchecked();
        handle_return_value(return_value)
    }

    /// Performs the system call.
    ///
    /// Like [`call`](Self::call) but never errors or panics. Forwards the return value of the system
    /// call.
    pub unsafe fn call_unchecked(&mut self) -> c_long {
        let cl_args = self.as_clone_args();
        crate::clone3_system_call(&cl_args)
    }

    /// Returns the underlying [`CloneArgs`](crate::CloneArgs).
    ///
    /// Prefer using [`call`](Self::call) instead. If you manually perform the system call you must
    /// ensure that the referenced variables stay alive and the referenced mutable variables are not
    /// aliased.
    pub fn as_clone_args(&mut self) -> CloneArgs {
        CloneArgs {
            flags: self.flags.bits(),
            pidfd: option_as_mut_ptr(&mut self.pidfd) as u64,
            child_tid: option_as_mut_ptr(&mut self.child_tid) as u64,
            parent_tid: option_as_mut_ptr(&mut self.parent_tid) as u64,
            exit_signal: self.exit_signal,
            stack: option_slice_as_mut_ptr(&mut self.stack) as u64,
            stack_size: self.stack.as_ref().map(|stack| stack.len()).unwrap_or(0) as u64,
            tls: self.tls.unwrap_or(0),
            #[cfg(feature = "linux_5-5")]
            set_tid: option_slice_as_ptr(&self.set_tid) as u64,
            #[cfg(feature = "linux_5-5")]
            set_tid_size: self.set_tid.map(|set_tid| set_tid.len()).unwrap_or(0) as u64,
            #[cfg(feature = "linux_5-7")]
            cgroup: self.cgroup.map(AsRawFd::as_raw_fd).unwrap_or(0) as u64,
        }
    }
}

fn find_incompatible_flags(flags: Flags) -> Option<String> {
    use Flags as F;

    let mutually_exclusive = [
        (F::CHILD_CLEARTID, F::CHILD_SETTID),
        #[cfg(feature = "linux_5-5")]
        (F::CLEAR_SIGHAND, F::SIGHAND),
        #[cfg(feature = "linux_5-7")]
        (F::INTO_CGROUP, F::NEWCGROUP),
        (F::NEWIPC, F::SYSVSEM),
        (F::FS, F::NEWNS),
        (F::THREAD, F::PIDFD),
        (F::NEWPID, F::PARENT | F::THREAD),
        (F::NEWUSER, F::FS | F::PARENT | F::THREAD),
    ];
    for (left, right) in mutually_exclusive.as_ref() {
        if flags.contains(*left) && flags.intersects(*right) {
            return Some(format!("{:?} and any of {:?} is set", left, right));
        }
    }

    let implies = [(F::SIGHAND, F::VM), (F::THREAD, F::SIGHAND)];
    for (left, right) in implies.as_ref() {
        if flags.contains(*left) && !flags.contains(*right) {
            return Some(format!("{:?} is set without {:?}", left, right));
        }
    }

    None
}

fn handle_return_value(return_value: c_long) -> Result<pid_t, Errno> {
    if return_value == -1 {
        return Err(Errno::default());
    }
    Ok(return_value.try_into().unwrap_or_else(|err| {
        panic!(
            "could not convert successful clone3 system call result {} to pid_t: {:?}",
            return_value, err
        )
    }))
}

fn option_as_mut_ptr<T>(o: &mut Option<&mut T>) -> *mut T {
    match o {
        Some(inner) => *inner as *mut T,
        None => std::ptr::null_mut(),
    }
}

#[cfg(feature = "linux_5-5")]
fn option_slice_as_ptr<T>(o: &Option<&[T]>) -> *const T {
    match o {
        Some(inner) => inner.as_ptr(),
        None => std::ptr::null(),
    }
}

fn option_slice_as_mut_ptr<T>(o: &mut Option<&mut [T]>) -> *mut T {
    match o {
        Some(inner) => inner.as_mut_ptr(),
        None => std::ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{mem, time::Duration};
    use uapi::c::{siginfo_t, waitid, P_PIDFD, WEXITED, __WCLONE};

    #[test]
    #[should_panic]
    fn panics_when_incompatible() {
        let mut clone3 = Clone3::default();
        clone3.flag_thread();
        unsafe {
            let _ = clone3.call();
        }
    }

    #[test]
    fn wait_for_child() {
        let mut pidfd = -1;
        let mut clone3 = Clone3::default();
        clone3.flag_pidfd(&mut pidfd);
        let child_pid = match unsafe { clone3.call() }.unwrap() {
            0 => {
                let sleep_duration = Duration::from_secs_f32(0.1);
                println!("child: sleeping {}s", sleep_duration.as_secs_f32());
                std::thread::sleep(sleep_duration);
                println!("child: done");
                return;
            }
            child_pid => child_pid,
        };
        assert!(pidfd >= 0);

        println!(
            "parent: waiting for child pid {} to exit on pidfd {}",
            child_pid, pidfd
        );
        // vk: Not sure how else to construct a siginfo_t.
        let mut siginfo = [0u8; mem::size_of::<siginfo_t>()];
        let return_value = unsafe {
            // We need __WCLONE because we have not set exit_signal to SIGCHLD.
            waitid(
                P_PIDFD,
                pidfd as _,
                siginfo.as_mut_ptr() as *mut _,
                WEXITED | __WCLONE,
            )
        };
        assert!(return_value != -1, "waitid failed: {:?}", Errno::default());
        println!("parent: child has exited");
    }
}
