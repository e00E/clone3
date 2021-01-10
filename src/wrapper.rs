use crate::{CloneArgs, Flags};
use std::{
    convert::TryInto,
    os::{raw::c_long, unix::io::RawFd},
};
use uapi::{c::pid_t, Errno};

/// Higher level wrapper around the clone3 system call.
#[derive(Debug, Default)]
pub struct Clone3<'a> {
    pub flags: Flags,
    pub pidfd: Option<&'a mut RawFd>,
    pub child_tid: Option<&'a mut pid_t>,
    pub parent_tid: Option<&'a mut pid_t>,
    pub exit_signal: u64,
    pub stack: Option<&'a mut [u8]>,
    pub tls: u64,
    pub set_tid: Option<&'a [pid_t]>,
    pub cgroup: RawFd,
}

impl<'a> Clone3<'a> {
    /// Perform the system call.
    ///
    /// # Errors
    ///
    /// Errors if the system call returns -1.
    ///
    /// # Panics
    ///
    /// Panics if the following conditions are not met:
    /// * PIDFD must be set iff pidfd is set
    /// * at least one of CHILD_CLEARTID and CHILD_SETTID must be set iff child_tid is set
    /// * PARENT_SETTID must be set iff parent_tid is set
    /// * if VM is set stack must be set
    pub unsafe fn call(self) -> Result<pid_t, Errno> {
        self.assert_consistency();
        let cl_args = self.into_clone_args();
        let return_value = crate::clone3_system_call(&cl_args);
        Self::handle_return_value(return_value)
    }

    /// Assert that the set flags match the other parameters.
    fn assert_consistency(&self) {
        assert_eq!(
            self.flags.contains(Flags::PIDFD),
            self.pidfd.is_some(),
            "PIDFD must be set iff pidfd is set"
        );
        assert_eq!(
            self.flags
                .intersects(Flags::CHILD_CLEARTID | Flags::CHILD_SETTID),
            self.child_tid.is_some(),
            "at least one of CHILD_CLEARTID and CHILD_SETTID must be set iff child_tid is set"
        );
        assert_eq!(
            self.flags.contains(Flags::PARENT_SETTID),
            self.set_tid.is_some(),
            "PARENT_SETTID must be set iff parent_tid is set"
        );
        if self.flags.contains(Flags::VM) {
            assert!(self.stack.is_some(), "if VM flag is set stack must be set");
        }
    }

    pub fn into_clone_args(mut self) -> CloneArgs {
        CloneArgs {
            flags: self.flags.bits(),
            pidfd: option_as_mut_ptr(&mut self.pidfd) as u64,
            child_tid: option_as_mut_ptr(&mut self.child_tid) as u64,
            parent_tid: option_as_mut_ptr(&mut self.parent_tid) as u64,
            exit_signal: self.exit_signal,
            stack: option_slice_as_mut_ptr(&mut self.stack) as u64,
            stack_size: self.stack.as_ref().map(|stack| stack.len()).unwrap_or(0) as u64,
            tls: self.tls,
            set_tid: option_slice_as_ptr(&self.set_tid) as u64,
            set_tid_size: self.set_tid.map(|set_tid| set_tid.len()).unwrap_or(0) as u64,
            cgroup: self.cgroup as u64,
        }
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
}

fn option_as_mut_ptr<T>(o: &mut Option<&mut T>) -> *mut T {
    match o {
        Some(inner) => *inner as *mut T,
        None => std::ptr::null_mut(),
    }
}

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
    fn panics_when_inconsistent() {
        let clone3 = Clone3 {
            flags: Flags::PIDFD,
            ..Default::default()
        };
        unsafe {
            let _ = clone3.call();
        }
    }

    #[test]
    fn wait_for_child() {
        let mut pidfd = 0;
        let clone3 = Clone3 {
            flags: Flags::PIDFD,
            pidfd: Some(&mut pidfd),
            ..Default::default()
        };
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
