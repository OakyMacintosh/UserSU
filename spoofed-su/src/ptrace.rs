use crate::state::StateManager;
use crate::types::{MinSukiError, Result};
use nix::sys::ptrace;
use nix::sys::signal::Signal;
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid};
use std::path::PathBuf;

/// Syscall numbers for x86_64
#[allow(dead_code)]
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
        // Allow parent to trace this process
        ptrace::traceme()
            .map_err(|e| MinSukiError::Ptrace(format!("traceme failed: {}", e)))?;
        
        // Execute the target command
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
        
        // Wait for child to stop after execve
        waitpid(child, None)
            .map_err(|e| MinSukiError::Ptrace(format!("Initial wait failed: {}", e)))?;
        
        // Set ptrace options
        ptrace::setoptions(
            child,
            ptrace::Options::PTRACE_O_TRACESYSGOOD | ptrace::Options::PTRACE_O_EXITKILL,
        ).map_err(|e| MinSukiError::Ptrace(format!("setoptions failed: {}", e)))?;
        
        let mut in_syscall = false;
        
        loop {
            // Continue execution until next syscall
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
                        // Entering syscall
                        if let Err(e) = self.handle_syscall_enter(child) {
                            log::error!("Error handling syscall enter: {}", e);
                        }
                    }
                    in_syscall = !in_syscall;
                }
                Ok(WaitStatus::Stopped(_, Signal::SIGTRAP)) => {
                    // Continue on trap
                }
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
        let regs = ptrace::getregs(pid)
            .map_err(|e| MinSukiError::Ptrace(format!("getregs failed: {}", e)))?;
        
        let syscall_num = regs.orig_rax as i64;
        
        match syscall_num {
            syscall::CHOWN | syscall::LCHOWN => {
                log::debug!("Intercepted chown/lchown syscall");
                // rdi = path, rsi = uid, rdx = gid
                self.handle_chown(pid, regs.rdi, regs.rsi as u32, regs.rdx as u32)?;
            }
            syscall::CHMOD => {
                log::debug!("Intercepted chmod syscall");
                // rdi = path, rsi = mode
                self.handle_chmod(pid, regs.rdi, regs.rsi as u32)?;
            }
            syscall::SETUID => {
                log::debug!("Intercepted setuid syscall");
                self.handle_setuid(regs.rdi as u32)?;
            }
            syscall::SETGID => {
                log::debug!("Intercepted setgid syscall");
                self.handle_setgid(regs.rdi as u32)?;
            }
            syscall::GETUID | syscall::GETEUID => {
                log::debug!("Intercepted getuid/geteuid syscall");
                self.handle_getuid(pid)?;
            }
            syscall::GETGID | syscall::GETEGID => {
                log::debug!("Intercepted getgid/getegid syscall");
                self.handle_getgid(pid)?;
            }
            _ => {
                // Let other syscalls pass through
            }
        }
        
        Ok(())
    }
    
    fn handle_chown(&self, pid: Pid, path_ptr: u64, uid: u32, gid: u32) -> Result<()> {
        // Read path string from child process memory
        let path = self.read_string(pid, path_ptr)?;
        
        self.state_manager.chown(PathBuf::from(path), uid, gid)?;
        
        // Modify return value to success
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
        let mut regs = ptrace::getregs(pid)
            .map_err(|e| MinSukiError::Ptrace(format!("getregs failed: {}", e)))?;
        
        regs.rax = value as u64;
        
        ptrace::setregs(pid, regs)
            .map_err(|e| MinSukiError::Ptrace(format!("setregs failed: {}", e)))?;
        
        Ok(())
    }
}