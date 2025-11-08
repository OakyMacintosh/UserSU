use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MinSukiError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("System call error: {0}")]
    Syscall(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Permission denied")]
    PermissionDenied,
    
    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),
    
    #[error("Process tracing error: {0}")]
    Ptrace(String),
}

pub type Result<T> = std::result::Result<T, MinSukiError>;

/// Represents fake file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FakeMetadata {
    pub uid: u32,
    pub gid: u32,
    pub mode: u32,
    pub capabilities: Vec<String>,
}

impl Default for FakeMetadata {
    fn default() -> Self {
        Self {
            uid: 0,  // fake root
            gid: 0,  // fake root group
            mode: 0o755,
            capabilities: Vec::new(),
        }
    }
}

/// The main state database that tracks emulated privileges
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FakeState {
    /// Map of file paths to their fake metadata
    pub files: HashMap<PathBuf, FakeMetadata>,
    
    /// Current fake UID
    pub current_uid: u32,
    
    /// Current fake GID
    pub current_gid: u32,
    
    /// Fake effective UID
    pub effective_uid: u32,
    
    /// Fake effective GID
    pub effective_gid: u32,
    
    /// Process capabilities
    pub capabilities: Vec<String>,
}

impl FakeState {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            current_uid: unsafe { libc::getuid() },
            current_gid: unsafe { libc::getgid() },
            effective_uid: 0, // Emulate root
            effective_gid: 0,
            capabilities: vec![
                "CAP_CHOWN".to_string(),
                "CAP_DAC_OVERRIDE".to_string(),
                "CAP_FOWNER".to_string(),
                "CAP_NET_BIND_SERVICE".to_string(),
            ],
        }
    }
    
    pub fn get_metadata(&self, path: &PathBuf) -> Option<&FakeMetadata> {
        self.files.get(path)
    }
    
    pub fn set_metadata(&mut self, path: PathBuf, metadata: FakeMetadata) {
        self.files.insert(path, metadata);
    }
    
    pub fn chown(&mut self, path: PathBuf, uid: u32, gid: u32) {
        let metadata = self.files.entry(path).or_insert_with(FakeMetadata::default);
        metadata.uid = uid;
        metadata.gid = gid;
    }
    
    pub fn chmod(&mut self, path: PathBuf, mode: u32) {
        let metadata = self.files.entry(path).or_insert_with(FakeMetadata::default);
        metadata.mode = mode;
    }
    
    pub fn is_root(&self) -> bool {
        self.effective_uid == 0
    }
}

/// System call interception mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterceptionMode {
    /// Use LD_PRELOAD to intercept libc calls
    LdPreload,
    
    /// Use ptrace to intercept syscalls
    Ptrace,
    
    /// Use seccomp-bpf with user notification
    Seccomp,
}

/// Configuration for MinSuki
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mode: String,  // "preload", "ptrace", "seccomp"
    pub state_file: PathBuf,
    pub log_level: String,
    pub allowed_paths: Vec<PathBuf>,
    pub denied_paths: Vec<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: "preload".to_string(),
            state_file: PathBuf::from("/tmp/minsuki.state"),
            log_level: "info".to_string(),
            allowed_paths: vec![PathBuf::from("/tmp"), PathBuf::from("/home")],
            denied_paths: vec![PathBuf::from("/etc/shadow")],
        }
    }
}