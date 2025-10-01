#!/system/bin/sh

set -e

# check if running through adb

# this script needs to be runned through adb, with root permissions, since adb
# has permissions to run a shell with root privileges.

# adb root

# Method 1: Check parent process name
check_adb_via_parent() {
    parent_pid=$(ps -o ppid= -p $$ | tr -d ' ')
    parent_name=$(ps -o comm= -p $parent_pid 2>/dev/null)
    
    if echo "$parent_name" | grep -q "adbd"; then
        return 0  # Running via ADB
    else
        return 1  # Not running via ADB
    fi
}

# Method 2: Check for ADB-specific environment variables
check_adb_via_env() {
    if [ -n "$ADB_VENDOR_KEYS" ] || [ "$SHLVL" = "1" ]; then
        return 0  # Likely running via ADB
    else
        return 1
    fi
}

# Method 3: Check TTY (ADB typically uses pts)
check_adb_via_tty() {
    tty_info=$(tty 2>/dev/null)
    if echo "$tty_info" | grep -q "pts"; then
        return 0  # Running in pseudo-terminal (typical for ADB)
    else
        return 1
    fi
}

# Main check combining methods
if check_adb_via_parent; then
    echo "✓ Script is running through ADB (detected via parent process)"
    exit 0
elif check_adb_via_tty; then
    echo "✓ Script is likely running through ADB (detected via TTY)"
    exit 0
else
    echo "✗ Script is NOT running through ADB"
    exit 1
fi

