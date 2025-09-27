use std::io::{Read, Write};
use std::ffi:CStr;
use std::mem;

struct UserSu {
    UserUID: enum,
    UserShell: String
}

pub fn get_os_type() -> &'static str {
    if cfg!(target_os = "linux") {
        "Linux"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "freebsd") {
        "FreeBSD"
    } else if cfg!(target_os = "openbsd") {
        "OpenBSD"
    } else if cfg!(target_os = "netbsd") {
        "NetBSD"
    } else if cfg!(target_os = "dragonfly") {
        "DragonFly BSD"
    } else if cfg!(target_os = "solaris") {
        "Solaris"
    } else if cfg!(target_os = "android") {
        "Android"
    } else if cfg!(target_family = "unix") {
        "Unknown Unix-like"
    } else {
        "Non-Unix"
    }
}
