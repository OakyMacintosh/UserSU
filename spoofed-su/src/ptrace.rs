// Android-compatible ptrace implementation
// Replace the problematic sections in src/ptrace.rs

use nix::sys::ptrace;
use nix::unistd::Pid;
use std::io;

// Architecture-specific register handling
#[cfg(target_arch = "aarch64")]
pub mod regs {
    use libc::{c_void, user_regs_struct};
    use nix::sys::ptrace::Request;
    use nix::unistd::Pid;
    use std::io;

    pub fn getregs(pid: Pid) -> io::Result<user_regs_struct> {
        let mut regs: user_regs_struct = unsafe { std::mem::zeroed() };
        unsafe {
            if libc::ptrace(
                Request::PTRACE_GETREGSET as i32,
                pid.as_raw(),
                libc::NT_PRSTATUS,
                &mut regs as *mut _ as *mut c_void,
            ) == -1
            {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(regs)
    }

    pub fn setregs(pid: Pid, regs: user_regs_struct) -> io::Result<()> {
        unsafe {
            if libc::ptrace(
                Request::PTRACE_SETREGSET as i32,
                pid.as_raw(),
                libc::NT_PRSTATUS,
                &regs as *const _ as *const c_void,
            ) == -1
            {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }
}

#[cfg(target_arch = "arm")]
pub mod regs {
    use libc::{c_void, user_regs_struct};
    use nix::sys::ptrace::Request;
    use nix::unistd::Pid;
    use std::io;

    pub fn getregs(pid: Pid) -> io::Result<user_regs_struct> {
        let mut regs: user_regs_struct = unsafe { std::mem::zeroed() };
        unsafe {
            if libc::ptrace(
                Request::PTRACE_GETREGS as i32,
                pid.as_raw(),
                0,
                &mut regs as *mut _ as *mut c_void,
            ) == -1
            {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(regs)
    }

    pub fn setregs(pid: Pid, regs: user_regs_struct) -> io::Result<()> {
        unsafe {
            if libc::ptrace(
                Request::PTRACE_SETREGS as i32,
                pid.as_raw(),
                0,
                &regs as *const _ as *const c_void,
            ) == -1
            {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(())
    }
}

#[cfg(target_arch = "x86_64")]
pub mod regs {
    use libc::user_regs_struct;
    use nix::sys::ptrace;
    use nix::unistd::Pid;
    use std::io;

    pub fn getregs(pid: Pid) -> io::Result<user_regs_struct> {
        ptrace::getregs(pid).map_err(|e| io::Error::from_raw_os_error(e as i32))
    }

    pub fn setregs(pid: Pid, regs: user_regs_struct) -> io::Result<()> {
        ptrace::setregs(pid, regs).map_err(|e| io::Error::from_raw_os_error(e as i32))
    }
}

#[cfg(target_arch = "x86")]
pub mod regs {
    use libc::user_regs_struct;
    use nix::sys::ptrace;
    use nix::unistd::Pid;
    use std::io;

    pub fn getregs(pid: Pid) -> io::Result<user_regs_struct> {
        ptrace::getregs(pid).map_err(|e| io::Error::from_raw_os_error(e as i32))
    }

    pub fn setregs(pid: Pid, regs: user_regs_struct) -> io::Result<()> {
        ptrace::setregs(pid, regs).map_err(|e| io::Error::from_raw_os_error(e as i32))
    }
}

// In your original ptrace.rs, replace:
//   ptrace::getregs(pid)
// with:
//   regs::getregs(pid)
//
// and replace:
//   ptrace::setregs(pid, regs)
// with:
//   regs::setregs(pid, regs)
