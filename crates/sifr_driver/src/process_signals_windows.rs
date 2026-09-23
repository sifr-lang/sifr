//! Scoped console cancellation shared by concurrent compiler subprocesses.
use std::io;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, Ordering};
use windows_sys::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler};

static CANCELLED: AtomicI32 = AtomicI32::new(0);
static USERS: Mutex<usize> = Mutex::new(0);

pub(crate) struct Guard;

unsafe extern "system" fn cancel(event: u32) -> i32 {
    match event {
        CTRL_C_EVENT => CANCELLED.store(libc::SIGINT, Ordering::Relaxed),
        CTRL_BREAK_EVENT => CANCELLED.store(libc::SIGTERM, Ordering::Relaxed),
        _ => return 0,
    }
    1
}

pub(crate) fn cancelled() -> i32 {
    CANCELLED.load(Ordering::Relaxed)
}

#[allow(unsafe_code)]
pub(crate) fn acquire() -> io::Result<Guard> {
    let mut users = USERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if *users == 0 {
        CANCELLED.store(0, Ordering::Relaxed);
        // SAFETY: the handler only stores an integer atomically and remains
        // installed while at least one guarded subprocess is live.
        if unsafe { SetConsoleCtrlHandler(Some(cancel), 1) } == 0 {
            return Err(io::Error::last_os_error());
        }
    }
    *users += 1;
    Ok(Guard)
}

impl Drop for Guard {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        let mut users = USERS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *users -= 1;
        if *users == 0 {
            // SAFETY: this is the exact handler installed by the first guard.
            unsafe { SetConsoleCtrlHandler(Some(cancel), 0) };
            CANCELLED.store(0, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
pub(crate) fn test_cancel() {
    CANCELLED.store(libc::SIGINT, Ordering::Relaxed);
}

#[cfg(test)]
pub(crate) fn test_active() -> bool {
    *USERS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        > 0
}
