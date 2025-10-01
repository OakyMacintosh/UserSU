#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/fs.h>
#include <linux/device.h>
#include <linux/cdev.h>
#include <linux/uaccess.h>
#include <linux/slab.h>
#include <linux/cred.h>
#include <linux/uidgid.h>
#include <linux/sched.h>
#include <linux/version.h>
#include <linux/security.h>
#include <linux/capability.h>
#include <linux/selinux.h>

#define DEVICE_NAME "usersu"
#define CLASS_NAME "usersu"
#define USERSU_MAGIC 0x55535500

/* IOCTL Commands */
#define USERSU_IOCTL_GRANT_ROOT     _IOW(USERSU_MAGIC, 1, int)
#define USERSU_IOCTL_DROP_ROOT      _IO(USERSU_MAGIC, 2)
#define USERSU_IOCTL_CHECK_ROOT     _IOR(USERSU_MAGIC, 3, int)
#define USERSU_IOCTL_SET_UID        _IOW(USERSU_MAGIC, 4, int)
#define USERSU_IOCTL_SET_GID        _IOW(USERSU_MAGIC, 5, int)
#define USERSU_IOCTL_GET_VERSION    _IOR(USERSU_MAGIC, 6, int)
#define USERSU_IOCTL_SET_CONTEXT    _IOW(USERSU_MAGIC, 7, char*)
#define USERSU_IOCTL_ADD_SUPP_GID   _IOW(USERSU_MAGIC, 8, int)

#define USERSU_VERSION 1

/* Android-specific UIDs */
#define AID_ROOT          0    /* traditional unix root user */
#define AID_SYSTEM     1000    /* system server */
#define AID_RADIO      1001    /* telephony subsystem */
#define AID_BLUETOOTH  1002    /* bluetooth subsystem */
#define AID_GRAPHICS   1003    /* graphics devices */
#define AID_SHELL      2000    /* adb and debug shell */
#define AID_CACHE      2001    /* cache access */
#define AID_DIAG       2002    /* access to diagnostic resources */
#define AID_MEDIA      1013    /* mediaserver process */
#define AID_SDCARD_RW  1015    /* external storage write access */
#define AID_WIFI       1010    /* wifi subsystem */

MODULE_LICENSE("Unlicense");
MODULE_AUTHOR("OakyMacintosh/Miguel V. Mesquita");
MODULE_DESCRIPTION("A user-based root solution for Android devices.");
MODULE_VERSION("0.1.0a");

static int major_number;
static struct class *usersu_class = NULL;
static struct device *usersu_device = NULL;
static struct cdev usersu_cdev;

/* Track if SELinux is present */
static bool selinux_present = false;

/* Device operations forward declarations */
static int dev_open(struct inode *, struct file *);
static int dev_release(struct inode *, struct file *);
static long dev_ioctl(struct file *, unsigned int, unsigned long);

static struct file_operations fops = {
    .owner = THIS_MODULE,
    .open = dev_open,
    .release = dev_release,
    .unlocked_ioctl = dev_ioctl,
#ifdef CONFIG_COMPAT
    .compat_ioctl = dev_ioctl,
#endif
};

/*
 * Check if SELinux is enabled
 */
static bool usersu_selinux_enabled(void)
{
#ifdef CONFIG_SECURITY_SELINUX
    return selinux_is_enabled();
#else
    return false;
#endif
}

/*
 * Set SELinux context (Android-specific)
 * In production, this needs proper SELinux policy integration
 */
static int usersu_set_selinux_context(const char *context)
{
#ifdef CONFIG_SECURITY_SELINUX
    /* This is a placeholder - actual implementation would need
     * to properly integrate with SELinux subsystem.
     * On Android, you typically need to:
     * 1. Define proper SELinux policy
     * 2. Use security_transition_sid() or similar
     * 3. Call security_task_setprocattr()
     */
    pr_info("UserSU: SELinux context change requested: %s\n", context);
    pr_warn("UserSU: SELinux context switching requires proper policy\n");
    return 0;
#else
    pr_info("UserSU: SELinux not compiled in kernel\n");
    return -ENOSYS;
#endif
}

/*
 * Check if the calling process has permission to use UserSU
 * Android-specific permission model
 */
static bool usersu_check_permission(void)
{
    uid_t uid = current_uid().val;
    
    /* Allow if already root */
    if (uid == AID_ROOT)
        return true;
    
    /* Allow shell user (adb) */
    if (uid == AID_SHELL)
        return true;
    
    /* Allow system server */
    if (uid == AID_SYSTEM)
        return true;
    
    /* Check for CAP_SYS_ADMIN */
    if (capable(CAP_SYS_ADMIN))
        return true;
    
    /* In production, implement:
     * - Check against authorized apps list (by UID)
     * - Verify app signatures
     * - Check for specific permission in Android manifest
     * - Implement rate limiting
     * - Log attempts for security monitoring
     */
    
    return false;
}

/*
 * Add supplementary group (Android uses many supplementary groups)
 */
static int usersu_add_supplementary_gid(gid_t gid)
{
    struct cred *new_cred;
    struct group_info *new_groups, *old_groups;
    int i;
    
    if (!capable(CAP_SETGID))
        return -EPERM;
    
    new_cred = prepare_creds();
    if (!new_cred)
        return -ENOMEM;
    
    old_groups = new_cred->group_info;
    
    /* Allocate new group_info with one more slot */
    new_groups = groups_alloc(old_groups->ngroups + 1);
    if (!new_groups) {
        abort_creds(new_cred);
        return -ENOMEM;
    }
    
    /* Copy existing groups */
    for (i = 0; i < old_groups->ngroups; i++) {
        new_groups->gid[i] = old_groups->gid[i];
    }
    
    /* Add new group */
    new_groups->gid[old_groups->ngroups] = make_kgid(current_user_ns(), gid);
    
    /* Replace group_info */
    new_cred->group_info = new_groups;
    
    commit_creds(new_cred);
    put_group_info(old_groups);
    
    pr_info("UserSU: Added supplementary GID %d to PID %d\n", gid, current->pid);
    
    return 0;
}

/*
 * Grant root privileges to the calling process
 * Android-aware version
 */
static int usersu_grant_root(void)
{
    struct cred *new_cred;
    struct group_info *groups;
    int i;
    
    if (!usersu_check_permission()) {
        pr_warn("UserSU: Permission denied for PID %d (UID %d, comm=%s)\n",
                current->pid, current_uid().val, current->comm);
        return -EACCES;
    }
    
    /* Prepare new credentials */
    new_cred = prepare_creds();
    if (!new_cred)
        return -ENOMEM;
    
    /* Set all UIDs and GIDs to root */
    new_cred->uid = new_cred->euid = new_cred->suid = new_cred->fsuid = GLOBAL_ROOT_UID;
    new_cred->gid = new_cred->egid = new_cred->sgid = new_cred->fsgid = GLOBAL_ROOT_GID;
    
    /* Grant full capabilities */
    for (i = 0; i < CAP_LAST_CAP; i++) {
        cap_raise(new_cred->cap_effective, i);
        cap_raise(new_cred->cap_permitted, i);
        cap_raise(new_cred->cap_inheritable, i);
    }
    
    /* Set bounding set */
    new_cred->cap_bset = CAP_FULL_SET;
    new_cred->cap_ambient = CAP_FULL_SET;
    
    /* Setup common Android supplementary groups for root */
    groups = groups_alloc(10);
    if (groups) {
        groups->gid[0] = make_kgid(current_user_ns(), AID_ROOT);
        groups->gid[1] = make_kgid(current_user_ns(), AID_SHELL);
        groups->gid[2] = make_kgid(current_user_ns(), AID_CACHE);
        groups->gid[3] = make_kgid(current_user_ns(), AID_DIAG);
        groups->gid[4] = make_kgid(current_user_ns(), AID_GRAPHICS);
        groups->gid[5] = make_kgid(current_user_ns(), AID_SDCARD_RW);
        groups->gid[6] = make_kgid(current_user_ns(), AID_MEDIA);
        groups->gid[7] = make_kgid(current_user_ns(), AID_WIFI);
        groups->gid[8] = make_kgid(current_user_ns(), 3003); /* inet */
        groups->gid[9] = make_kgid(current_user_ns(), 3002); /* net_bt_admin */
        groups->ngroups = 10;
        
        put_group_info(new_cred->group_info);
        new_cred->group_info = groups;
    }
    
    /* Commit the new credentials */
    commit_creds(new_cred);
    
    pr_info("UserSU: Granted root to PID %d, comm=%s (original UID %d)\n",
            current->pid, current->comm, current_uid().val);
    
    return 0;
}

/*
 * Drop root privileges
 */
static int usersu_drop_root(uid_t target_uid, gid_t target_gid)
{
    struct cred *new_cred;
    
    new_cred = prepare_creds();
    if (!new_cred)
        return -ENOMEM;
    
    /* Set UIDs and GIDs to target values */
    new_cred->uid = new_cred->euid = new_cred->suid = new_cred->fsuid = 
        make_kuid(current_user_ns(), target_uid);
    new_cred->gid = new_cred->egid = new_cred->sgid = new_cred->fsgid = 
        make_kgid(current_user_ns(), target_gid);
    
    /* Drop capabilities */
    cap_clear(new_cred->cap_effective);
    cap_clear(new_cred->cap_permitted);
    cap_clear(new_cred->cap_inheritable);
    cap_clear(new_cred->cap_bset);
    cap_clear(new_cred->cap_ambient);
    
    commit_creds(new_cred);
    
    pr_info("UserSU: Dropped privileges for PID %d to UID %d, GID %d\n",
            current->pid, target_uid, target_gid);
    
    return 0;
}

/*
 * Set specific UID
 */
static int usersu_set_uid(uid_t uid)
{
    struct cred *new_cred;
    
    if (!capable(CAP_SETUID))
        return -EPERM;
    
    new_cred = prepare_creds();
    if (!new_cred)
        return -ENOMEM;
    
    new_cred->uid = new_cred->euid = new_cred->suid = new_cred->fsuid = 
        make_kuid(current_user_ns(), uid);
    
    commit_creds(new_cred);
    
    pr_info("UserSU: Set UID to %d for PID %d\n", uid, current->pid);
    
    return 0;
}

/*
 * Set specific GID
 */
static int usersu_set_gid(gid_t gid)
{
    struct cred *new_cred;
    
    if (!capable(CAP_SETGID))
        return -EPERM;
    
    new_cred = prepare_creds();
    if (!new_cred)
        return -ENOMEM;
    
    new_cred->gid = new_cred->egid = new_cred->sgid = new_cred->fsgid = 
        make_kgid(current_user_ns(), gid);
    
    commit_creds(new_cred);
    
    pr_info("UserSU: Set GID to %d for PID %d\n", gid, current->pid);
    
    return 0;
}

/*
 * Device open
 */
static int dev_open(struct inode *inodep, struct file *filep)
{
    pr_debug("UserSU: Device opened by PID %d (UID %d, comm=%s)\n", 
             current->pid, current_uid().val, current->comm);
    return 0;
}

/*
 * Device release
 */
static int dev_release(struct inode *inodep, struct file *filep)
{
    pr_debug("UserSU: Device closed by PID %d\n", current->pid);
    return 0;
}

/*
 * Device IOCTL handler
 */
static long dev_ioctl(struct file *filep, unsigned int cmd, unsigned long arg)
{
    int ret = 0;
    int value;
    uid_t uid;
    gid_t gid;
    char context[256];
    
    switch (cmd) {
    case USERSU_IOCTL_GRANT_ROOT:
        ret = usersu_grant_root();
        break;
        
    case USERSU_IOCTL_DROP_ROOT:
        if (copy_from_user(&uid, (uid_t __user *)arg, sizeof(uid_t))) {
            ret = -EFAULT;
            break;
        }
        ret = usersu_drop_root(uid, uid);
        break;
        
    case USERSU_IOCTL_CHECK_ROOT:
        value = (current_uid().val == 0) ? 1 : 0;
        if (copy_to_user((int __user *)arg, &value, sizeof(int)))
            ret = -EFAULT;
        break;
        
    case USERSU_IOCTL_SET_UID:
        if (copy_from_user(&uid, (uid_t __user *)arg, sizeof(uid_t))) {
            ret = -EFAULT;
            break;
        }
        ret = usersu_set_uid(uid);
        break;
        
    case USERSU_IOCTL_SET_GID:
        if (copy_from_user(&gid, (gid_t __user *)arg, sizeof(gid_t))) {
            ret = -EFAULT;
            break;
        }
        ret = usersu_set_gid(gid);
        break;
        
    case USERSU_IOCTL_GET_VERSION:
        value = USERSU_VERSION;
        if (copy_to_user((int __user *)arg, &value, sizeof(int)))
            ret = -EFAULT;
        break;
        
    case USERSU_IOCTL_SET_CONTEXT:
        if (copy_from_user(context, (char __user *)arg, sizeof(context))) {
            ret = -EFAULT;
            break;
        }
        context[sizeof(context) - 1] = '\0';
        ret = usersu_set_selinux_context(context);
        break;
        
    case USERSU_IOCTL_ADD_SUPP_GID:
        if (copy_from_user(&gid, (gid_t __user *)arg, sizeof(gid_t))) {
            ret = -EFAULT;
            break;
        }
        ret = usersu_add_supplementary_gid(gid);
        break;
        
    default:
        pr_warn("UserSU: Invalid IOCTL command: 0x%x\n", cmd);
        ret = -EINVAL;
        break;
    }
    
    return ret;
}

/*
 * Module initialization
 */
static int __init usersu_init(void)
{
    dev_t dev;
    int ret;
    
    pr_info("UserSU: Initializing Android-compatible module\n");
    pr_info("UserSU: Kernel version %s\n", UTS_RELEASE);
    
    /* Check SELinux status */
    selinux_present = usersu_selinux_enabled();
    if (selinux_present) {
        pr_info("UserSU: SELinux is enabled\n");
        pr_warn("UserSU: Proper SELinux policy integration required\n");
    } else {
        pr_info("UserSU: SELinux is disabled or not present\n");
    }
    
    /* Allocate device number */
    ret = alloc_chrdev_region(&dev, 0, 1, DEVICE_NAME);
    if (ret < 0) {
        pr_err("UserSU: Failed to allocate device number\n");
        return ret;
    }
    major_number = MAJOR(dev);
    pr_info("UserSU: Registered with major number %d\n", major_number);
    
    /* Initialize cdev */
    cdev_init(&usersu_cdev, &fops);
    usersu_cdev.owner = THIS_MODULE;
    
    ret = cdev_add(&usersu_cdev, dev, 1);
    if (ret < 0) {
        unregister_chrdev_region(dev, 1);
        pr_err("UserSU: Failed to add cdev\n");
        return ret;
    }
    
    /* Create device class */
    usersu_class = class_create(THIS_MODULE, CLASS_NAME);
    if (IS_ERR(usersu_class)) {
        cdev_del(&usersu_cdev);
        unregister_chrdev_region(dev, 1);
        pr_err("UserSU: Failed to create device class\n");
        return PTR_ERR(usersu_class);
    }
    
    /* Create device */
    usersu_device = device_create(usersu_class, NULL, dev, NULL, DEVICE_NAME);
    if (IS_ERR(usersu_device)) {
        class_destroy(usersu_class);
        cdev_del(&usersu_cdev);
        unregister_chrdev_region(dev, 1);
        pr_err("UserSU: Failed to create device\n");
        return PTR_ERR(usersu_device);
    }
    
    pr_info("UserSU: Device created successfully at /dev/%s\n", DEVICE_NAME);
    pr_info("UserSU: Ready for Android use\n");
    
    return 0;
}

/*
 * Module cleanup
 */
static void __exit usersu_exit(void)
{
    dev_t dev = MKDEV(major_number, 0);
    
    device_destroy(usersu_class, dev);
    class_destroy(usersu_class);
    cdev_del(&usersu_cdev);
    unregister_chrdev_region(dev, 1);
    
    pr_info("UserSU: Module unloaded\n");
}

module_init(usersu_init);
module_exit(usersu_exit);