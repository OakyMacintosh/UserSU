// Android-compatible ptrace implementation
// Complete ptrace.rs file

use nix::sys::ptrace;
use nix::unistd::Pid;
use std::io;

// Architecture-specific register handling
#[cfg(target_arch = "aarch64")]
pub mod regs {
    use libc::c_void;
    use nix::unistd::Pid;
    use std::io;

    // ARM64 user_pt_regs structure
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct user_regs_struct {
        pub regs: [u64; 31],
        pub sp: u64,
        pub pc: u64,
        pub pstate: u64,
    }

    pub fn getregs(pid: Pid) -> io::Result<user_regs_struct> {
        let mut regs: user_regs_struct = unsafe { std::mem::zeroed() };
        let mut iov = libc::iovec {
            iov_base: &mut regs as *mut _ as *mut c_void,
            iov_len: std::mem::size_of::<user_regs_struct>(),
        };
        
        unsafe {
            if libc::ptrace(
                libc::PTRACE_GETREGSET,
                pid.as_raw(),
                libc::NT_PRSTATUS,
                &mut iov as *mut _ as *mut c_void,
            ) == -1
            {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(regs)
    }

    pub fn setregs(pid: Pid, regs: user_regs_struct) -> io::Result<()> {
        let mut iov = libc::iovec {
            iov_base: &regs as *const _ as *mut c_void,
            iov_len: std::mem::size_of::<user_regs_struct>(),
        };
        
        unsafe {
            if libc::ptrace(
                libc::PTRACE_SETREGSET,
                pid.as_raw(),
                libc::NT_PRSTATUS,
                &mut iov as *mut _ as *mut c_void,
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
    use libc::c_void;
    use nix::unistd::Pid;
    use std::io;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct user_regs_struct {
        pub uregs: [u32; 18],
    }

    pub fn getregs(pid: Pid) -> io::Result<user_regs_struct> {
        let mut regs: user_regs_struct = unsafe { std::mem::zeroed() };
        unsafe {
            if libc::ptrace(
                libc::PTRACE_GETREGS,
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
                libc::PTRACE_SETREGS,
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

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod regs {
    pub use libc::user_regs_struct;
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

use crate::state::StateManager;
use crate::types::{MinSukiError, Result};
use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};
use std::path::PathBuf;

/// Syscall numbers - architecture specific
#[cfg(target_arch = "aarch64")]
mod syscall {
    pub const CHOWN: i64 = 53;
    pub const FCHOWN: i64 = 55;
    pub const LCHOWN: i64 = 16;
    pub const CHMOD: i64 = 15;
    pub const FCHMOD: i64 = 52;
    pub const SETUID: i64 = 146;
    pub const SETGID: i64 = 144;
    pub const GETUID: i64 = 174;
    pub const GETEUID: i64 = 175;
    pub const GETGID: i64 = 176;
    pub const GETEGID: i64 = 177;
}

#[cfg(target_arch = "arm")]
mod syscall {
    pub const CHOWN: i64 = 182;
    pub const FCHOWN: i64 = 207;
    pub const LCHOWN: i64 = 16;
    pub const CHMOD: i64 = 15;
    pub const FCHMOD: i64 = 94;
    pub const SETUID: i64 = 213;
    pub const SETGID: i64 = 214;
    pub const GETUID: i64 = 199;
    pub const GETEUID: i64 = 201;
    pub const GETGID: i64 = 200;
    pub const GETEGID: i64 = 202;
}

#[cfg(target_arch = "x86_64")]
mod syscall {
    pub const CHOWN: i64 = 92;
    pub const FCHOWN: i64 = 93;
    pub const LCHOWN: i64 = 94;
    pub const CHMOD: i64 = 90;
    pub const FCHMOD: i64 = 91;
    pub const SETUID: i64 = 105;
    pub const SETGID: i64 = 106;
    pub const GETUID: i64 = 102;
    pub const GETEUID: i64 = 107;
    pub const GETGID: i64 = 104;
    pub const GETEGID: i64 = 108;
}

#[cfg(target_arch = "x86")]
mod syscall {
    pub const CHOWN: i64 = 182;
    pub const FCHOWN: i64 = 95;
    pub const LCHOWN: i64 = 16;
    pub const CHMOD: i64 = 15;
    pub const FCHMOD: i64 = 94;
    pub const SETUID: i64 = 213;
    pub const SETGID: i64 = 214;
    pub const GETUID: i64 = 199;
    pub const GETEUID: i64 = 201;
    pub const GETGID: i64 = 200;
    pub const GETEGID: i64 = 202;
}

pub struct PtraceInterceptor {
    state_manager: StateManager,
}

impl PtraceInterceptor {
    pub fn new(state_file: &str) -> Result<Self> {
        Ok(Self {
            state_manager: StateManager::new(state_file)?,
        })
    }
    
    pub fn run(&self, command: &[String]) -> Result<()> {
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                self.trace_child(child)
            }
            Ok(ForkResult::Child) => {
                self.setup_tracee(command)
            }
            Err(e) => Err(MinSukiError::Syscall(format!("Fork failed: {}", e))),
        }
    }
    
    fn setup_tracee(&self, command: &[String]) -> Result<()> {
        ptrace::traceme()
            .map_err(|e| MinSukiError::Ptrace(format!("traceme failed: {}", e)))?;
        
        let program = &command[0];
        let args: Vec<&str> = command.iter().map(|s| s.as_str()).collect();
        
        nix::unistd::execvp(
            &std::ffi::CString::new(program.as_str()).unwrap(),
            &args.iter().map(|s| std::ffi::CString::new(*s).unwrap()).collect::<Vec<_>>()
        ).map_err(|e| MinSukiError::Syscall(format!("execvp failed: {}", e)))?;
        
        unreachable!()
    }
    
    fn trace_child(&self, child: Pid) -> Result<()> {
        log::info!("Tracing child process: {}", child);
        
        waitpid(child, None)
            .map_err(|e| MinSukiError::Ptrace(format!("Initial wait failed: {}", e)))?;
        
        ptrace::setoptions(
            child,
            ptrace::Options::PTRACE_O_TRACESYSGOOD | ptrace::Options::PTRACE_O_EXITKILL,
        ).map_err(|e| MinSukiError::Ptrace(format!("setoptions failed: {}", e)))?;
        
        let mut in_syscall = false;
        
        loop {
            ptrace::syscall(child, None)
                .map_err(|e| MinSukiError::Ptrace(format!("syscall continue failed: {}", e)))?;
            
            match waitpid(child, None) {
                Ok(WaitStatus::Exited(_, code)) => {
                    log::info!("Child exited with code: {}", code);
                    return Ok(());
                }
                Ok(WaitStatus::Signaled(_, signal, _)) => {
                    log::info!("Child killed by signal: {:?}", signal);
                    return Ok(());
                }
                Ok(WaitStatus::PtraceSyscall(_)) => {
                    if !in_syscall {
                        if let Err(e) = self.handle_syscall_enter(child) {
                            log::error!("Error handling syscall enter: {}", e);
                        }
                    }
                    in_syscall = !in_syscall;
                }
                Ok(WaitStatus::Stopped(_, Signal::SIGTRAP)) => {}
                Ok(status) => {
                    log::debug!("Unexpected wait status: {:?}", status);
                }
                Err(e) => {
                    return Err(MinSukiError::Ptrace(format!("waitpid failed: {}", e)));
                }
            }
        }
    }
    
    fn handle_syscall_enter(&self, pid: Pid) -> Result<()> {
        let regs = regs::getregs(pid)
            .map_err(|e| MinSukiError::Ptrace(format!("getregs failed: {}", e)))?;
        
        // Get syscall number - architecture specific
        #[cfg(target_arch = "aarch64")]
        let syscall_num = regs.regs[8] as i64;
        
        #[cfg(target_arch = "arm")]
        let syscall_num = regs.uregs[7] as i64;
        
        #[cfg(target_arch = "x86_64")]
        let syscall_num = regs.orig_rax as i64;
        
        #[cfg(target_arch = "x86")]
        let syscall_num = regs.orig_eax as i64;
        
        match syscall_num {
            syscall::CHOWN | syscall::LCHOWN => {
                log::debug!("Intercepted chown/lchown syscall");
                #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
                self.handle_chown(pid, regs.regs[0], regs.regs[1] as u32, regs.regs[2] as u32)?;
                #[cfg(target_arch = "x86_64")]
                self.handle_chown(pid, regs.rdi, regs.rsi as u32, regs.rdx as u32)?;
                #[cfg(target_arch = "x86")]
                self.handle_chown(pid, regs.ebx as u64, regs.ecx as u32, regs.edx as u32)?;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    fn handle_chown(&self, pid: Pid, path_ptr: u64, uid: u32, gid: u32) -> Result<()> {
        let path = self.read_string(pid, path_ptr)?;
        self.state_manager.chown(PathBuf::from(path), uid, gid)?;
        self.set_syscall_return(pid, 0)?;
        Ok(())
    }
    
    fn handle_chmod(&self, pid: Pid, path_ptr: u64, mode: u32) -> Result<()> {
        let path = self.read_string(pid, path_ptr)?;
        self.state_manager.chmod(PathBuf::from(path), mode)?;
        self.set_syscall_return(pid, 0)?;
        Ok(())
    }
    
    fn handle_setuid(&self, uid: u32) -> Result<()> {
        self.state_manager.setuid(uid)
    }
    
    fn handle_setgid(&self, gid: u32) -> Result<()> {
        self.state_manager.setgid(gid)
    }
    
    fn handle_getuid(&self, pid: Pid) -> Result<()> {
        let state = self.state_manager.get_state();
        let state = state.lock().unwrap();
        let uid = state.current_uid;
        self.set_syscall_return(pid, uid as i64)?;
        Ok(())
    }
    
    fn handle_getgid(&self, pid: Pid) -> Result<()> {
        let state = self.state_manager.get_state();
        let state = state.lock().unwrap();
        let gid = state.current_gid;
        self.set_syscall_return(pid, gid as i64)?;
        Ok(())
    }
    
    fn read_string(&self, pid: Pid, addr: u64) -> Result<String> {
        let mut result = Vec::new();
        let mut offset = 0;
        
        loop {
            let word = ptrace::read(pid, (addr + offset) as *mut _)
                .map_err(|e| MinSukiError::Ptrace(format!("read failed: {}", e)))?;
            
            let bytes = word.to_ne_bytes();
            for &byte in &bytes {
                if byte == 0 {
                    return String::from_utf8(result)
                        .map_err(|_| MinSukiError::Ptrace("Invalid UTF-8".to_string()));
                }
                result.push(byte);
            }
            
            offset += 8;
            if offset > 4096 {
                return Err(MinSukiError::Ptrace("String too long".to_string()));
            }
        }
    }
    
    fn set_syscall_return(&self, pid: Pid, value: i64) -> Result<()> {
        let mut regs = regs::getregs(pid)
            .map_err(|e| MinSukiError::Ptrace(format!("getregs failed: {}", e)))?;
        
        // Set return value - architecture specific
        #[cfg(target_arch = "aarch64")]
        { regs.regs[0] = value as u64; }
        
        #[cfg(target_arch = "arm")]
        { regs.uregs[0] = value as u32; }
        
        #[cfg(target_arch = "x86_64")]
        { regs.rax = value as u64; }
        
        #[cfg(target_arch = "x86")]
        { regs.eax = value as u32; }
        
        regs::setregs(pid, regs)
            .map_err(|e| MinSukiError::Ptrace(format!("setregs failed: {}", e)))?;
        
        Ok(())
    }
}
