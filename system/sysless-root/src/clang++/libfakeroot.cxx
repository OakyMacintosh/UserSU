#define _GNU_SOURCE
#include <dlfcn.h>
#include <fcntl.h>
#include <stdarg.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <cerrno>
#include <mutex>
#include <functional>
#include <limits.h>
#include <sys/syscall.h>

static std::string get_prefix() {
    const char* p = getenv("FAKE_ROOT");
    if (!p || p[0] == '\0') return std::string();
    return std::string(p);
}

static bool rewrite_path_in(const char* path, std::string &out) {
    if (!path) return false;
    if (path[0] != '/') { out = path; return true; } // relative -> unchanged
    std::string prefix = get_prefix();
    if (prefix.empty()) { out = path; return true; }
    if (prefix.back() == '/')
        out = prefix + (path + 1);
    else
        out = prefix + path;
    return true;
}

template<typename T>
static T load_sym(const char* name) {
    static std::mutex m;
    std::lock_guard<std::mutex> lk(m);
    void* p = dlsym(RTLD_NEXT, name);
    return reinterpret_cast<T>(p);
}

/* ---------- open ---------- */
using open_fn_t = int(*)(const char*, int, ...);
extern "C" int open(const char* pathname, int flags, ...) {
    static open_fn_t real_open = nullptr;
    if (!real_open) real_open = load_sym<open_fn_t>("open");

    std::string newpath;
    if (!rewrite_path_in(pathname, newpath)) { errno = EFAULT; return -1; }

    if (flags & O_CREAT) {
        va_list ap;
        va_start(ap, flags);
        mode_t mode = (mode_t)va_arg(ap, int);
        va_end(ap);
        if (!real_open) { errno = ENOSYS; return -1; }
        return real_open(newpath.c_str(), flags, mode);
    } else {
        if (!real_open) { errno = ENOSYS; return -1; }
        return real_open(newpath.c_str(), flags);
    }
}

/* ---------- openat ---------- */
using openat_fn_t = int(*)(int, const char*, int, ...);
extern "C" int openat(int dirfd, const char* pathname, int flags, ...) {
    static openat_fn_t real_openat = nullptr;
    if (!real_openat) real_openat = load_sym<openat_fn_t>("openat");

    if (!pathname) { errno = EFAULT; return -1; }

    if (pathname[0] == '/') {
        std::string newpath;
        rewrite_path_in(pathname, newpath);
        if (flags & O_CREAT) {
            va_list ap;
            va_start(ap, flags);
            mode_t mode = (mode_t)va_arg(ap, int);
            va_end(ap);
            if (!real_openat) { errno = ENOSYS; return -1; }
            return real_openat(AT_FDCWD, newpath.c_str(), flags, mode);
        } else {
            if (!real_openat) { errno = ENOSYS; return -1; }
            return real_openat(AT_FDCWD, newpath.c_str(), flags);
        }
    } else {
        if (flags & O_CREAT) {
            va_list ap;
            va_start(ap, flags);
            mode_t mode = (mode_t)va_arg(ap, int);
            va_end(ap);
            if (!real_openat) { errno = ENOSYS; return -1; }
            return real_openat(dirfd, pathname, flags, mode);
        } else {
            if (!real_openat) { errno = ENOSYS; return -1; }
            return real_openat(dirfd, pathname, flags);
        }
    }
}

/* ---------- fopen ---------- */
using fopen_fn_t = FILE*(*)(const char*, const char*);
extern "C" FILE* fopen(const char* path, const char* mode) {
    static fopen_fn_t real_fopen = nullptr;
    if (!real_fopen) real_fopen = load_sym<fopen_fn_t>("fopen");
    std::string newpath;
    if (!rewrite_path_in(path, newpath)) { errno = EFAULT; return nullptr; }
    if (!real_fopen) { errno = ENOSYS; return nullptr; }
    return real_fopen(newpath.c_str(), mode);
}

/* ---------- stat / lstat / fstat / __xstat ---------- */
using stat_fn_t = int(*)(const char*, struct stat*);
extern "C" int stat(const char* path, struct stat* buf) {
    static stat_fn_t real_stat = nullptr;
    if (!real_stat) real_stat = load_sym<stat_fn_t>("stat");
    std::string newpath;
    if (!rewrite_path_in(path, newpath)) { errno = EFAULT; return -1; }
    if (!real_stat) { errno = ENOSYS; return -1; }
    return real_stat(newpath.c_str(), buf);
}

using lstat_fn_t = int(*)(const char*, struct stat*);
extern "C" int lstat(const char* path, struct stat* buf) {
    static lstat_fn_t real_lstat = nullptr;
    if (!real_lstat) real_lstat = load_sym<lstat_fn_t>("lstat");
    std::string newpath;
    if (!rewrite_path_in(path, newpath)) { errno = EFAULT; return -1; }
    if (!real_lstat) { errno = ENOSYS; return -1; }
    return real_lstat(newpath.c_str(), buf);
}

using fstat_fn_t = int(*)(int, struct stat*);
extern "C" int fstat(int fd, struct stat* buf) {
    static fstat_fn_t real_fstat = nullptr;
    if (!real_fstat) real_fstat = load_sym<fstat_fn_t>("fstat");
    if (!real_fstat) { errno = ENOSYS; return -1; }
    return real_fstat(fd, buf);
}

#if defined(__GLIBC__)
extern "C" int __xstat(int ver, const char* path, struct stat* buf) {
    typedef int (*xstat_fn_t)(int, const char*, struct stat*);
    static xstat_fn_t real_xstat = nullptr;
    if (!real_xstat) real_xstat = load_sym<xstat_fn_t>("__xstat");
    std::string newpath;
    if (!rewrite_path_in(path, newpath)) { errno = EFAULT; return -1; }
    if (!real_xstat) { errno = ENOSYS; return -1; }
    return real_xstat(ver, newpath.c_str(), buf);
}
extern "C" int __lxstat(int ver, const char* path, struct stat* buf) {
    typedef int (*lxstat_fn_t)(int, const char*, struct stat*);
    static lxstat_fn_t real_lxstat = nullptr;
    if (!real_lxstat) real_lxstat = load_sym<lxstat_fn_t>("__lxstat");
    std::string newpath;
    if (!rewrite_path_in(path, newpath)) { errno = EFAULT; return -1; }
    if (!real_lxstat) { errno = ENOSYS; return -1; }
    return real_lxstat(ver, newpath.c_str(), buf);
}
#endif

/* ---------- access ---------- */
using access_fn_t = int(*)(const char*, int);
extern "C" int access(const char* path, int mode) {
    static access_fn_t real_access = nullptr;
    if (!real_access) real_access = load_sym<access_fn_t>("access");
    std::string newpath;
    if (!rewrite_path_in(path, newpath)) { errno = EFAULT; return -1; }
    if (!real_access) { errno = ENOSYS; return -1; }
    return real_access(newpath.c_str(), mode);
}

/* ---------- unlink / unlinkat ---------- */
using unlink_fn_t = int(*)(const char*);
extern "C" int unlink(const char* path) {
    static unlink_fn_t real_unlink = nullptr;
    if (!real_unlink) real_unlink = load_sym<unlink_fn_t>("unlink");
    std::string newpath;
    if (!rewrite_path_in(path, newpath)) { errno = EFAULT; return -1; }
    if (!real_unlink) { errno = ENOSYS; return -1; }
    return real_unlink(newpath.c_str());
}

using unlinkat_fn_t = int(*)(int, const char*, int);
extern "C" int unlinkat(int dirfd, const char* path, int flags) {
    static unlinkat_fn_t real_unlinkat = nullptr;
    if (!real_unlinkat) real_unlinkat = load_sym<unlinkat_fn_t>("unlinkat");
    if (!path) { errno = EFAULT; return -1; }
    if (path[0] == '/') {
        std::string newpath;
        rewrite_path_in(path, newpath);
        if (!real_unlinkat) { errno = ENOSYS; return -1; }
        return real_unlinkat(AT_FDCWD, newpath.c_str(), flags);
    } else {
        if (!real_unlinkat) { errno = ENOSYS; return -1; }
        return real_unlinkat(dirfd, path, flags);
    }
}

/* ---------- rename / renameat ---------- */
using rename_fn_t = int(*)(const char*, const char*);
extern "C" int rename(const char* oldpath, const char* newpath) {
    static rename_fn_t real_rename = nullptr;
    if (!real_rename) real_rename = load_sym<rename_fn_t>("rename");
    std::string o, n;
    if (!rewrite_path_in(oldpath, o)) { errno = EFAULT; return -1; }
    if (!rewrite_path_in(newpath, n)) { errno = EFAULT; return -1; }
    if (!real_rename) { errno = ENOSYS; return -1; }
    return real_rename(o.c_str(), n.c_str());
}

using renameat_fn_t = int(*)(int, const char*, int, const char*);
extern "C" int renameat(int olddirfd, const char* oldpath, int newdirfd, const char* newpath) {
    static renameat_fn_t real_renameat = nullptr;
    if (!real_renameat) real_renameat = load_sym<renameat_fn_t>("renameat");
    // Simplify: if either path is absolute rewrite it, and use AT_FDCWD
    std::string o, n;
    if (!rewrite_path_in(oldpath, o)) { errno = EFAULT; return -1; }
    if (!rewrite_path_in(newpath, n)) { errno = EFAULT; return -1; }
    if (!real_renameat) { errno = ENOSYS; return -1; }
    return real_renameat(AT_FDCWD, o.c_str(), AT_FDCWD, n.c_str());
}

/* ---------- mkdir / mkdirat / rmdir ---------- */
using mkdir_fn_t = int(*)(const char*, mode_t);
extern "C" int mkdir(const char* path, mode_t mode) {
    static mkdir_fn_t real_mkdir = nullptr;
    if (!real_mkdir) real_mkdir = load_sym<mkdir_fn_t>("mkdir");
    std::string newpath;
    if (!rewrite_path_in(path, newpath)) { errno = EFAULT; return -1; }
    if (!real_mkdir) { errno = ENOSYS; return -1; }
    return real_mkdir(newpath.c_str(), mode);
}

using mkdirat_fn_t = int(*)(int, const char*, mode_t);
extern "C" int mkdirat(int dirfd, const char* path, mode_t mode) {
    static mkdirat_fn_t real_mkdirat = nullptr;
    if (!real_mkdirat) real_mkdirat = load_sym<mkdirat_fn_t>("mkdirat");
    if (!path) { errno = EFAULT; return -1; }
    if (path[0] == '/') {
        std::string newpath;
        rewrite_path_in(path, newpath);
        if (!real_mkdirat) { errno = ENOSYS; return -1; }
        return real_mkdirat(AT_FDCWD, newpath.c_str(), mode);
    } else {
        if (!real_mkdirat) { errno = ENOSYS; return -1; }
        return real_mkdirat(dirfd, path, mode);
    }
}

using rmdir_fn_t = int(*)(const char*);
extern "C" int rmdir(const char* path) {
    static rmdir_fn_t real_rmdir = nullptr;
    if (!real_rmdir) real_rmdir = load_sym<rmdir_fn_t>("rmdir");
    std::string newpath;
    if (!rewrite_path_in(path, newpath)) { errno = EFAULT; return -1; }
    if (!real_rmdir) { errno = ENOSYS; return -1; }
    return real_rmdir(newpath.c_str());
}

/* ---------- chmod / chown ---------- */
using chmod_fn_t = int(*)(const char*, mode_t);
extern "C" int chmod(const char* path, mode_t mode) {
    static chmod_fn_t real_chmod = nullptr;
    if (!real_chmod) real_chmod = load_sym<chmod_fn_t>("chmod");
    std::string newpath;
    if (!rewrite_path_in(path, newpath)) { errno = EFAULT; return -1; }
    if (!real_chmod) { errno = ENOSYS; return -1; }
    return real_chmod(newpath.c_str(), mode);
}

using chown_fn_t = int(*)(const char*, uid_t, gid_t);
extern "C" int chown(const char* path, uid_t owner, gid_t group) {
    static chown_fn_t real_chown = nullptr;
    if (!real_chown) real_chown = load_sym<chown_fn_t>("chown");
    std::string newpath;
    if (!rewrite_path_in(path, newpath)) { errno = EFAULT; return -1; }
    if (!real_chown) { errno = ENOSYS; return -1; }
    return real_chown(newpath.c_str(), owner, group);
}

/* ---------- execve (rewrite absolute filename) ---------- */
using execve_fn_t = int(*)(const char*, char* const[], char* const[]);
extern "C" int execve(const char* filename, char* const argv[], char* const envp[]) {
    static execve_fn_t real_execve = nullptr;
    if (!real_execve) real_execve = load_sym<execve_fn_t>("execve");
    if (!filename) { errno = EFAULT; return -1; }
    if (filename[0] == '/') {
        std::string newpath;
        rewrite_path_in(filename, newpath);
        if (!real_execve) { errno = ENOSYS; return -1; }
        return real_execve(newpath.c_str(), argv, envp);
    } else {
        if (!real_execve) { errno = ENOSYS; return -1; }
        return real_execve(filename, argv, envp);
    }
}

/* ---------- getuid / geteuid (fake root) ---------- */
using uid_t_fn_t = uid_t(*)(void);
extern "C" uid_t getuid(void) {
    static uid_t_fn_t real_getuid = nullptr;
    if (!real_getuid) real_getuid = load_sym<uid_t_fn_t>("getuid");
    if (getenv("FAKE_ROOT_UID")) return 0;
    if (!real_getuid) return (uid_t)-1;
    return real_getuid();
}
extern "C" uid_t geteuid(void) {
    static uid_t_fn_t real_geteuid = nullptr;
    if (!real_geteuid) real_geteuid = load_sym<uid_t_fn_t>("geteuid");
    if (getenv("FAKE_ROOT_UID")) return 0;
    if (!real_geteuid) return (uid_t)-1;
    return real_geteuid();
}
