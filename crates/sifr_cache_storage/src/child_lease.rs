//! Pass a lease only to the command that consumes the protected storage.
use std::fs::File;
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Retain an owned descriptor until command disposal and clear CLOEXEC only in
/// the forked child. Other threads and unrelated commands never inherit it.
/// The caller must drop the Command after use to release its retained reference.
#[allow(unsafe_code)]
pub fn inherit_lease_for_command(command: &mut Command, lease: &File) -> io::Result<()> {
    let owned = lease.try_clone()?;
    // SAFETY: the closure owns the live descriptor. After fork it uses only
    // async-signal-safe fcntl and errno access; it neither locks nor allocates.
    unsafe {
        command.pre_exec(move || {
            let flags = libc::fcntl(owned.as_raw_fd(), libc::F_GETFD);
            if flags < 0
                || libc::fcntl(owned.as_raw_fd(), libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    Ok(())
}

/// Give an owned cache producer private creation permissions without changing
/// the parent process or unrelated children. Runtime consumers keep their
/// caller's normal umask and use inherit_lease_for_command instead.
#[allow(unsafe_code)]
pub fn inherit_lease_for_private_command(command: &mut Command, lease: &File) -> io::Result<()> {
    inherit_lease_for_command(command, lease)?;
    // SAFETY: umask is async-signal-safe and runs only in the forked child.
    // No parent-wide umask mutation, allocation, or synchronization occurs.
    unsafe {
        command.pre_exec(|| {
            libc::umask(0o077);
            Ok(())
        });
    }
    Ok(())
}

#[cfg(test)]
#[path = "child_lease_tests.rs"]
mod tests;
