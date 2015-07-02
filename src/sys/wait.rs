use libc::{pid_t, c_int};
use errno::Errno;
use {Error, Result};

use sys::signal;

mod ffi {
    use libc::{pid_t, c_int};

    extern {
        pub fn waitpid(pid: pid_t, status: *mut c_int, options: c_int) -> pid_t;
    }
}

bitflags!(
    flags WaitPidFlag: c_int {
        const WNOHANG = 0x00000001,
    }
);

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum WaitStatus {
    Exited(pid_t, i8),
    Signaled(pid_t, signal::SigNum, bool),
    Stopped(pid_t, signal::SigNum),
    Continued(pid_t),
    StillAlive
}

#[cfg(target_os = "linux")]
mod status {
    use libc::pid_t;
    use super::WaitStatus;
    use sys::signal;

    fn exited(status: i32) -> bool {
        (status & 0x7F) == 0
    }

    fn exit_status(status: i32) -> i8 {
        ((status & 0xFF00) >> 8) as i8
    }

    fn signaled(status: i32) -> bool {
        ((((status & 0x7f) + 1) as i8) >> 1) > 0
    }

    fn term_signal(status: i32) -> signal::SigNum {
        (status & 0x7f) as signal::SigNum
    }

    fn dumped_core(status: i32) -> bool {
        (status & 0x80) != 0
    }

    fn stopped(status: i32) -> bool {
        (status & 0xff) == 0x7f
    }

    fn stop_signal(status: i32) -> signal::SigNum {
        ((status & 0xFF00) >> 8) as signal::SigNum
    }

    fn continued(status: i32) -> bool {
        status == 0xFFFF
    }

    pub fn decode(pid : pid_t, status: i32) -> WaitStatus {
        if exited(status) {
            WaitStatus::Exited(pid, exit_status(status))
        } else if signaled(status) {
            WaitStatus::Signaled(pid, term_signal(status), dumped_core(status))
        } else if stopped(status) {
            WaitStatus::Stopped(pid, stop_signal(status))
        } else {
            println!("status is {}", status);
            assert!(continued(status));
            WaitStatus::Continued(pid)
        }
    }
}

pub fn waitpid(pid: pid_t, options: Option<WaitPidFlag>) -> Result<WaitStatus> {
    use self::WaitStatus::*;

    let mut status: i32 = 0;

    let option_bits = match options {
        Some(bits) => bits.bits(),
        None => 0
    };

    let res = unsafe { ffi::waitpid(pid as pid_t, &mut status as *mut c_int, option_bits) };

    if res < 0 {
        Err(Error::Sys(Errno::last()))
    } else if res == 0 {
        Ok(StillAlive)
    } else {
        Ok(status::decode(res, status))
    }
}

pub fn wait() -> Result<WaitStatus> {
    waitpid(-1, None)
}
