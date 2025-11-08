use crate::state::StateManager;
use lazy_static::lazy_static;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::path::PathBuf;
use std::sync::Mutex;

lazy_static! {
    static ref STATE_MANAGER: Mutex<Option<StateManager>> = Mutex::new(None);
}

fn init_state_manager() {
    let mut manager = STATE_MANAGER.lock().unwrap();
    if manager.is_none() {
        let state_file = std::env::var("MINSUKI_STATE")
            .unwrap_or_else(|_| "/tmp/minsuki.state".to_string());
        *manager = StateManager::new(&state_file).ok();
    }
}

fn get_state_manager() -> Option<StateManager> {
    init_state_manager();
    STATE_MANAGER.lock().unwrap().clone()
}

// Helper function to convert C string to PathBuf
unsafe fn cstr_to_pathbuf(path: *const c_char) -> Option<PathBuf> {
    if path.is_null() {
        return None;
    }
    CStr::from_ptr(path)
        .to_str()
        .ok()
        .map(|s| PathBuf::from(s))
}

/// Intercept chown system call
#[no_mangle]
pub unsafe extern "C" fn chown(path: *const c_char, uid: libc::uid_t, gid: libc::gid_t) -> c_int {
    log::debug!("Intercepted chown: uid={}, gid={}", uid, gid);
    
    if let Some(pathbuf) = cstr_to_pathbuf(path) {
        if let Some(manager) = get_state_manager() {
            if manager.chown(pathbuf, uid, gid).is_ok() {
                log::info!("Emulated chown successfully");
                return 0; // Success
            }
        }
    }
    
    // Fall back to real chown (will likely fail without root)
    libc::chown(path, uid, gid)
}

/// Intercept lchown system call
#[no_mangle]
pub unsafe extern "C" fn lchown(path: *const c_char, uid: libc::uid_t, gid: libc::gid_t) -> c_int {
    log::debug!("Intercepted lchown: uid={}, gid={}", uid, gid);
    
    if let Some(pathbuf) = cstr_to_pathbuf(path) {
        if let Some(manager) = get_state_manager() {
            if manager.chown(pathbuf, uid, gid).is_ok() {
                return 0;
            }
        }
    }
    
    libc::lchown(path, uid, gid)
}

/// Intercept fchown system call
#[no_mangle]
pub unsafe extern "C" fn fchown(fd: c_int, uid: libc::uid_t, gid: libc::gid_t) -> c_int {
    log::debug!("Intercepted fchown: fd={}, uid={}, gid={}", fd, uid, gid);
    
    // For file descriptors, we'd need to track fd->path mapping
    // For now, just pretend it succeeded
    if let Some(manager) = get_state_manager() {
        let state = manager.get_state();
        let state = state.lock().unwrap();
        if state.is_root() {
            return 0; // Fake success
        }
    }
    
    libc::fchown(fd, uid, gid)
}

/// Intercept chmod system call
#[no_mangle]
pub unsafe extern "C" fn chmod(path: *const c_char, mode: libc::mode_t) -> c_int {
    log::debug!("Intercepted chmod: mode={:o}", mode);
    
    if let Some(pathbuf) = cstr_to_pathbuf(path) {
        if let Some(manager) = get_state_manager() {
            if manager.chmod(pathbuf, mode).is_ok() {
                return 0;
            }
        }
    }
    
    libc::chmod(path, mode)
}

/// Intercept fchmod system call
#[no_mangle]
pub unsafe extern "C" fn fchmod(fd: c_int, mode: libc::mode_t) -> c_int {
    log::debug!("Intercepted fchmod: fd={}, mode={:o}", fd, mode);
    
    // Fake success if we're emulating root
    if let Some(manager) = get_state_manager() {
        let state = manager.get_state();
        let state = state.lock().unwrap();
        if state.is_root() {
            return 0;
        }
    }
    
    libc::fchmod(fd, mode)
}

/// Intercept setuid system call
#[no_mangle]
pub unsafe extern "C" fn setuid(uid: libc::uid_t) -> c_int {
    log::debug!("Intercepted setuid: uid={}", uid);
    
    if let Some(manager) = get_state_manager() {
        if manager.setuid(uid).is_ok() {
            return 0;
        }
    }
    
    libc::setuid(uid)
}

/// Intercept setgid system call
#[no_mangle]
pub unsafe extern "C" fn setgid(gid: libc::gid_t) -> c_int {
    log::debug!("Intercepted setgid: gid={}", gid);
    
    if let Some(manager) = get_state_manager() {
        if manager.setgid(gid).is_ok() {
            return 0;
        }
    }
    
    libc::setgid(gid)
}

/// Intercept geteuid to return fake root
#[no_mangle]
pub unsafe extern "C" fn geteuid() -> libc::uid_t {
    if let Some(manager) = get_state_manager() {
        let state = manager.get_state();
        let state = state.lock().unwrap();
        return state.effective_uid;
    }
    
    libc::geteuid()
}

/// Intercept getuid to return fake root
#[no_mangle]
pub unsafe extern "C" fn getuid() -> libc::uid_t {
    if let Some(manager) = get_state_manager() {
        let state = manager.get_state();
        let state = state.lock().unwrap();
        return state.current_uid;
    }
    
    libc::getuid()
}

/// Intercept getegid to return fake root group
#[no_mangle]
pub unsafe extern "C" fn getegid() -> libc::gid_t {
    if let Some(manager) = get_state_manager() {
        let state = manager.get_state();
        let state = state.lock().unwrap();
        return state.effective_gid;
    }
    
    libc::getegid()
}

/// Intercept getgid to return fake root group
#[no_mangle]
pub unsafe extern "C" fn getgid() -> libc::gid_t {
    if let Some(manager) = get_state_manager() {
        let state = manager.get_state();
        let state = state.lock().unwrap();
        return state.current_gid;
    }
    
    libc::getgid()
}