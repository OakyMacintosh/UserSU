use std::io::{Read, Write};
use std::ffi:CStr;
use std::mem;

struct UserSu {
    UserUID: enum,
    UserShell: String
}

#[repr(C)]
struct UtsName {
    sysname: [libc::c_char; 65],
    nodename: [libc::c_char; 65],
    release: [libc::c_char; 65],
    version: [libc::c_char; 65],
    machine: [libc::c_char; 65],
    domainname: [libc::c_char; 65],
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

fn get_system_info() -> Result<(), Box<dyn std::error::Error>> {
    let mut uts: UtsName = unsafe { mem::zeroed() };
    
    let result = unsafe { libc::uname(&mut uts as *mut UtsName as *mut libc::utsname) };
    
    if result == 0 {
        let sysname = unsafe { CStr::from_ptr(uts.sysname.as_ptr()) };
        let release = unsafe { CStr::from_ptr(uts.release.as_ptr()) };
        let machine = unsafe { CStr::from_ptr(uts.machine.as_ptr()) };
        
        println!("System: {}", sysname.to_str()?);
        println!("Release: {}", release.to_str()?);
        println!("Machine: {}", machine.to_str()?);
    }
    
    Ok(())
}

fn printHelp() -> bool {
    println!("UserSU help\n");
    println!("usage: usud [command] <args>\n");
    println!("STILL IN DEV BRO\n");
}

fn rootCheck() {
    // Quick check if currently running as root
    if is_root() {
        println!("Root access is already available!");
    }

    // Comprehensive check
    let access = SuperUserAccess::check();
    if access.has_any_access() {
        println!("Some form of root access is available");
    }

    // Check if you need to request elevation
    if !access.has_current_root() && access.can_escalate {
        println!("Need to use sudo for root operations");
    }
}

fn main() {
    
}