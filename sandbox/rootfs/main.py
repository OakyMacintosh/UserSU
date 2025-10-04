#!/usr/bin/env python3
# UserSU sandbox rootfs generator
# Run this script in Termux to copy Android system binaries

import os
import sys
import subprocess
import shutil
from pathlib import Path
import typer
from typing import Optional

app = typer.Typer(help="UserSU: Create and manage PRoot-based Android sandbox environments")

# Android-like directory structure
rootfs_dirs = [
    "system/bin",
    "system/xbin", 
    "system/lib",
    "system/lib64",
    "system/etc",
    "system/usr",
    "sys",
    "data/data",
    "data/local/tmp",
    "misc",
    "boot",
    "recovery",
    "dev",
    "proc",
    "sdcard",
    "storage",
    "mnt",
    "tmp",
    "root",
    "etc",
    "bin",
    "sbin",
    "usr/bin",
    "usr/sbin",
    "var/log",
    "cache"
]

# Symlinks to create (Android compatibility)
symlinks = {
    "bin": "/system/bin",
    "sbin": "/system/xbin",
    "lib": "/system/lib",
    "lib64": "/system/lib64",
    "etc": "/system/etc",
    "usr": "/system/usr"
}

# Android system paths to check
ANDROID_BIN_PATHS = [
    "/system/bin",
    "/system/xbin",
    Path.home() / "../usr/bin",  # Termux
]

ANDROID_LIB_PATHS = [
    "/system/lib",
    "/system/lib64",
    Path.home() / "../usr/lib",  # Termux
]


def is_termux():
    """Check if running in Termux environment"""
    return "com.termux" in str(Path.home()) or os.getenv("TERMUX_VERSION") is not None


def find_binaries():
    """Find available binaries from Android/Termux"""
    binaries = []
    for bin_path in ANDROID_BIN_PATHS:
        bin_path = Path(bin_path)
        if bin_path.exists():
            binaries.extend([f for f in bin_path.iterdir() if f.is_file() and os.access(f, os.X_OK)])
    return binaries


@app.command()
def create(
    path: Path = typer.Argument(..., help="Path where rootfs will be created"),
    minimal: bool = typer.Option(False, "--minimal", "-m", help="Create minimal structure only"),
    copy_bins: bool = typer.Option(True, "--copy-bins/--no-copy-bins", help="Copy system binaries"),
    verbose: bool = typer.Option(False, "--verbose", "-v", help="Verbose output")
):
    """Create a new Android-like rootfs structure for PRoot sandbox"""
    
    if not is_termux():
        typer.secho("Warning: Not running in Termux. Some features may not work correctly.", 
                   fg=typer.colors.YELLOW)
        typer.echo("For best results, run this script in Termux on Android.\n")
    
    if path.exists() and any(path.iterdir()):
        typer.confirm(f"Directory {path} already exists and is not empty. Continue?", abort=True)
    
    path.mkdir(parents=True, exist_ok=True)
    
    typer.echo(f"Creating rootfs at: {path.absolute()}")
    
    # Create directory structure
    dirs_to_create = rootfs_dirs[:6] if minimal else rootfs_dirs
    
    for directory in dirs_to_create:
        dir_path = path / directory
        dir_path.mkdir(parents=True, exist_ok=True)
        if verbose:
            typer.echo(f"  Created: {directory}")
    
    # Create symlinks
    if not minimal:
        for link_name, target in symlinks.items():
            link_path = path / link_name
            if link_path.exists() or link_path.is_symlink():
                link_path.unlink()
            link_path.symlink_to(target)
            if verbose:
                typer.echo(f"  Symlinked: {link_name} -> {target}")
    
    # Create basic files
    (path / "system/etc/hosts").write_text("127.0.0.1 localhost\n::1 localhost\n")
    
    # Create build.prop (Android-like)
    build_prop = path / "system/build.prop"
    build_prop.write_text("""ro.build.version.sdk=33
ro.product.model=UserSU_Sandbox
ro.build.type=user
""")
    
    # Copy binaries from Android/Termux
    if copy_bins:
        typer.echo("\nCopying system binaries...")
        binaries = find_binaries()
        
        if not binaries:
            typer.secho("Warning: No system binaries found to copy", fg=typer.colors.YELLOW)
        else:
            dest_bin = path / "system/bin"
            copied = 0
            
            with typer.progressbar(binaries, label="Copying binaries") as progress:
                for binary in progress:
                    try:
                        dest_file = dest_bin / binary.name
                        if not dest_file.exists():
                            shutil.copy2(binary, dest_file)
                            dest_file.chmod(0o755)
                            copied += 1
                    except (PermissionError, OSError) as e:
                        if verbose:
                            typer.echo(f"  Skipped {binary.name}: {e}")
            
            typer.secho(f"✓ Copied {copied} binaries", fg=typer.colors.GREEN)
            
            # Copy essential libraries
            typer.echo("Copying essential libraries...")
            lib_copied = 0
            
            for lib_path in ANDROID_LIB_PATHS:
                lib_path = Path(lib_path)
                if not lib_path.exists():
                    continue
                
                # Determine destination (lib or lib64)
                dest_lib = path / "system/lib64" if "lib64" in str(lib_path) else path / "system/lib"
                
                # Copy .so files
                for lib_file in lib_path.glob("*.so*"):
                    try:
                        dest_file = dest_lib / lib_file.name
                        if not dest_file.exists():
                            shutil.copy2(lib_file, dest_file)
                            lib_copied += 1
                    except (PermissionError, OSError):
                        pass
            
            if lib_copied > 0:
                typer.secho(f"✓ Copied {lib_copied} libraries", fg=typer.colors.GREEN)
    
    typer.secho(f"\n✓ Rootfs created successfully at {path.absolute()}", fg=typer.colors.GREEN)
    typer.echo(f"\nTo enter the sandbox, run:")
    typer.echo(f"  python generateRootfs.py enter {path.absolute()}")


@app.command()
def enter(
    path: Path = typer.Argument(..., help="Path to rootfs"),
    command: Optional[str] = typer.Option(None, "--command", "-c", help="Command to execute"),
    bind: list[str] = typer.Option([], "--bind", "-b", help="Bind mount directories (format: src:dest)"),
    link2symlink: bool = typer.Option(True, "--link2symlink/--no-link2symlink", help="Enable link2symlink")
):
    """Enter the PRoot sandbox environment"""
    
    if not path.exists():
        typer.secho(f"Error: Rootfs path {path} does not exist", fg=typer.colors.RED, err=True)
        raise typer.Exit(1)
    
    # Build PRoot command
    proot_cmd = ["proot"]
    
    # Set root directory
    proot_cmd.extend(["-r", str(path.absolute())])
    
    # Bind mounts
    default_binds = [
        "/dev",
        "/proc", 
        "/sys"
    ]
    
    for bind_dir in default_binds:
        proot_cmd.extend(["-b", bind_dir])
    
    # User-specified binds
    for bind_spec in bind:
        proot_cmd.extend(["-b", bind_spec])
    
    # Link2symlink support
    if link2symlink:
        proot_cmd.append("-L")
    
    # Set working directory
    proot_cmd.extend(["-w", "/root"])
    
    # Set environment
    proot_cmd.extend(["-0"])  # Fake root user (uid=0)
    
    # Set PATH
    proot_cmd.extend(["--env", "PATH=/system/bin:/system/xbin:/bin:/sbin:/usr/bin:/usr/sbin"])
    
    # Command to execute
    if command:
        proot_cmd.extend(["/system/bin/sh", "-c", command])
    else:
        # Try to find a shell
        shells = ["/system/bin/sh", "/bin/sh", "/system/bin/bash", "/bin/bash"]
        shell_found = None
        
        for shell in shells:
            if (path / shell.lstrip('/')).exists():
                shell_found = shell
                break
        
        if shell_found:
            proot_cmd.append(shell_found)
        else:
            typer.secho("Warning: No shell found in rootfs", fg=typer.colors.YELLOW)
            proot_cmd.append("/system/bin/sh")
    
    typer.echo(f"Entering sandbox at {path.absolute()}...")
    
    try:
        subprocess.run(proot_cmd)
    except FileNotFoundError:
        typer.secho("Error: PRoot is not installed", fg=typer.colors.RED, err=True)
        typer.echo("\nInstall PRoot in Termux with:")
        typer.echo("  pkg install proot")
        raise typer.Exit(1)
    except KeyboardInterrupt:
        typer.echo("\nExited sandbox")


@app.command()
def info(path: Path = typer.Argument(..., help="Path to rootfs")):
    """Show information about a rootfs"""
    
    if not path.exists():
        typer.secho(f"Error: Path {path} does not exist", fg=typer.colors.RED, err=True)
        raise typer.Exit(1)
    
    typer.echo(f"Rootfs: {path.absolute()}\n")
    
    # Check directory structure
    typer.echo("Directory structure:")
    for directory in rootfs_dirs[:10]:
        dir_path = path / directory
        exists = "✓" if dir_path.exists() else "✗"
        color = typer.colors.GREEN if dir_path.exists() else typer.colors.RED
        typer.secho(f"  {exists} {directory}", fg=color)
    
    # Check for build.prop
    build_prop = path / "system/build.prop"
    if build_prop.exists():
        typer.echo(f"\nBuild info:")
        for line in build_prop.read_text().strip().split('\n'):
            typer.echo(f"  {line}")
    
    # Count binaries
    bin_dir = path / "system/bin"
    if bin_dir.exists():
        binaries = list(bin_dir.iterdir())
        typer.echo(f"\nBinaries: {len(binaries)} files in /system/bin")
    
    # Count libraries
    lib_count = 0
    for lib_dir in ["system/lib", "system/lib64"]:
        lib_path = path / lib_dir
        if lib_path.exists():
            lib_count += len(list(lib_path.glob("*.so*")))
    
    if lib_count > 0:
        typer.echo(f"Libraries: {lib_count} shared objects")
    
    # Estimate size
    total_size = sum(f.stat().st_size for f in path.rglob('*') if f.is_file())
    typer.echo(f"\nTotal size: {total_size / 1024 / 1024:.2f} MB")


@app.command()
def update_binaries(
    path: Path = typer.Argument(..., help="Path to rootfs"),
    verbose: bool = typer.Option(False, "--verbose", "-v", help="Verbose output")
):
    """Update/add binaries from the current Android/Termux system"""
    
    if not path.exists():
        typer.secho(f"Error: Rootfs {path} does not exist", fg=typer.colors.RED, err=True)
        raise typer.Exit(1)
    
    if not is_termux():
        typer.secho("Warning: Not running in Termux. Limited binaries available.", 
                   fg=typer.colors.YELLOW)
    
    typer.echo("Updating binaries from system...")
    binaries = find_binaries()
    
    if not binaries:
        typer.secho("Error: No system binaries found", fg=typer.colors.RED, err=True)
        raise typer.Exit(1)
    
    dest_bin = path / "system/bin"
    dest_bin.mkdir(parents=True, exist_ok=True)
    
    copied = 0
    updated = 0
    
    with typer.progressbar(binaries, label="Copying binaries") as progress:
        for binary in progress:
            try:
                dest_file = dest_bin / binary.name
                if dest_file.exists():
                    # Update if source is newer
                    if binary.stat().st_mtime > dest_file.stat().st_mtime:
                        shutil.copy2(binary, dest_file)
                        dest_file.chmod(0o755)
                        updated += 1
                else:
                    shutil.copy2(binary, dest_file)
                    dest_file.chmod(0o755)
                    copied += 1
            except (PermissionError, OSError) as e:
                if verbose:
                    typer.echo(f"  Skipped {binary.name}: {e}")
    
    typer.secho(f"\n✓ Added {copied} new binaries", fg=typer.colors.GREEN)
    typer.secho(f"✓ Updated {updated} existing binaries", fg=typer.colors.GREEN)


if __name__ == "__main__":
    app()