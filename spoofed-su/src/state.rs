use crate::types::{FakeState, MinSukiError, Result};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Manages persistent state for the fake root environment
#[derive(Clone)]
pub struct StateManager {
    state: Arc<Mutex<FakeState>>,
    state_file: String,
}

impl StateManager {
    pub fn new(state_file: &str) -> Result<Self> {
        let state = if Path::new(state_file).exists() {
            Self::load_from_file(state_file)?
        } else {
            FakeState::new()
        };
        
        Ok(Self {
            state: Arc::new(Mutex::new(state)),
            state_file: state_file.to_string(),
        })
    }
    
    fn load_from_file(path: &str) -> Result<FakeState> {
        let mut file = File::open(path)?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        
        bincode::deserialize(&contents)
            .map_err(|e| MinSukiError::Serialization(e.to_string()))
    }
    
    pub fn save(&self) -> Result<()> {
        let state = self.state.lock().unwrap();
        let encoded = bincode::serialize(&*state)
            .map_err(|e| MinSukiError::Serialization(e.to_string()))?;
        
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.state_file)?;
        
        file.write_all(&encoded)?;
        Ok(())
    }
    
    pub fn get_state(&self) -> Arc<Mutex<FakeState>> {
        Arc::clone(&self.state)
    }
    
    pub fn chown(&self, path: std::path::PathBuf, uid: u32, gid: u32) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.chown(path, uid, gid);
        drop(state);
        self.save()
    }
    
    pub fn chmod(&self, path: std::path::PathBuf, mode: u32) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.chmod(path, mode);
        drop(state);
        self.save()
    }
    
    pub fn setuid(&self, uid: u32) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.effective_uid = uid;
        state.current_uid = uid;
        drop(state);
        self.save()
    }
    
    pub fn setgid(&self, gid: u32) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.effective_gid = gid;
        state.current_gid = gid;
        drop(state);
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_state_persistence() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();
        
        {
            let manager = StateManager::new(path).unwrap();
            manager.chown("/test/file".into(), 1000, 1000).unwrap();
        }
        
        {
            let manager = StateManager::new(path).unwrap();
            let state = manager.get_state();
            let state = state.lock().unwrap();
            let metadata = state.get_metadata(&"/test/file".into()).unwrap();
            assert_eq!(metadata.uid, 1000);
            assert_eq!(metadata.gid, 1000);
        }
    }
}