#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdarg.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>
#include <fcntl.h>
#include <stdlib.h>
#include <errno.h>

static char *get_prefix() {
    char *p = getenv("FAKE_ROOT");
    return p ? p : "";
}

static int path_rewrite(const char *path, char *out, size_t outlen) {
    if (!path) return -1;
    if (path[0] != '/') { // relative path — leave alone
        strncpy(out, path, outlen-1);
        out[outlen-1]=0;
        return 0;
    }
    char *prefix = get_prefix();
    if (prefix[0] == 0) {
        strncpy(out, path, outlen-1);
        out[outlen-1]=0;
        return 0;
    }
    // combine prefix + path (avoid duplicate slashes)
    size_t n = snprintf(out, outlen, "%s%s", prefix, path);
    return (n < outlen) ? 0 : -1;
}

/* Intercept open */
typedef int (*open_t)(const char *, int, ...);
int open(const char *pathname, int flags, ...) {
    static open_t real_open = NULL;
    if (!real_open) real_open = (open_t)dlsym(RTLD_NEXT, "open");

    char newpath[4096];
    if (path_rewrite(pathname, newpath, sizeof(newpath)) == 0) {
        if (flags & O_CREAT) {
            va_list ap;
            va_start(ap, flags);
            mode_t mode = va_arg(ap, int);
            va_end(ap);
            return real_open(newpath, flags, mode);
        } else {
            return real_open(newpath, flags);
        }
    }
    errno = EFAULT;
    return -1;
}

/* Intercept fopen */
typedef FILE *(*fopen_t)(const char *, const char *);
FILE *fopen(const char *path, const char *mode) {
    static fopen_t real_fopen = NULL;
    if (!real_fopen) real_fopen = (fopen_t)dlsym(RTLD_NEXT, "fopen");
    char newpath[4096];
    if (path_rewrite(path, newpath, sizeof(newpath)) == 0) {
        return real_fopen(newpath, mode);
    }
    return NULL;
}

/* Intercept stat */
typedef int (*stat_t)(const char *, struct stat *);
int stat(const char *path, struct stat *buf) {
    static stat_t real_stat = NULL;
    if (!real_stat) real_stat = (stat_t)dlsym(RTLD_NEXT, "stat");
    char newpath[4096];
    if (path_rewrite(path, newpath, sizeof(newpath)) == 0) {
        return real_stat(newpath, buf);
    }
    return -1;
}

/* Intercept access */
typedef int (*access_t)(const char *, int);
int access(const char *path, int mode) {
    static access_t real_access = NULL;
    if (!real_access) real_access = (access_t)dlsym(RTLD_NEXT, "access");
    char newpath[4096];
    if (path_rewrite(path, newpath, sizeof(newpath)) == 0) {
        return real_access(newpath, mode);
    }
    return -1;
}

/* Fake uid/euid if FAKE_ROOT_UID=1 */
typedef uid_t (*getuid_t)(void);
uid_t getuid(void) {
    static getuid_t real_getuid = NULL;
    if (!real_getuid) real_getuid = (getuid_t)dlsym(RTLD_NEXT, "getuid");
    if (getenv("FAKE_ROOT_UID")) return 0;
    return real_getuid();
}

typedef uid_t (*geteuid_t)(void);
uid_t geteuid(void) {
    static geteuid_t real_geteuid = NULL;
    if (!real_geteuid) real_geteuid = (geteuid_t)dlsym(RTLD_NEXT, "geteuid");
    if (getenv("FAKE_ROOT_UID")) return 0;
    return real_geteuid();
}
