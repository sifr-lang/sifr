//! Scoped signal ownership shared by concurrent compiler subprocesses.
use std::io;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI32, Ordering};

static CANCELLED: AtomicI32 = AtomicI32::new(0);
static STATE: Mutex<Option<State>> = Mutex::new(None);
struct State {
    users: usize,
    interrupt: libc::sigaction,
    terminate: libc::sigaction,
}
pub(crate) struct Guard;

extern "C" fn cancel(signal: libc::c_int) {
    CANCELLED.store(signal, Ordering::Relaxed);
}

pub(crate) fn cancelled() -> i32 {
    CANCELLED.load(Ordering::Relaxed)
}

#[allow(unsafe_code)]
pub(crate) fn acquire() -> io::Result<Guard> {
    let mut state = STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(state) = state.as_mut() {
        state.users += 1;
        return Ok(Guard);
    }
    CANCELLED.store(0, Ordering::Relaxed);
    // SAFETY: all sigaction storage is valid, the handler only stores to an
    // atomic, and the original handlers are restored after the last live owner.
    unsafe {
        let mut handler: libc::sigaction = std::mem::zeroed();
        handler.sa_sigaction = cancel as *const () as libc::sighandler_t;
        libc::sigemptyset(&raw mut handler.sa_mask);
        let mut interrupt = std::mem::MaybeUninit::uninit();
        let mut terminate = std::mem::MaybeUninit::uninit();
        if libc::sigaction(libc::SIGINT, &raw const handler, interrupt.as_mut_ptr()) != 0 {
            return Err(io::Error::last_os_error());
        }
        let interrupt = interrupt.assume_init();
        if libc::sigaction(libc::SIGTERM, &raw const handler, terminate.as_mut_ptr()) != 0 {
            let error = io::Error::last_os_error();
            libc::sigaction(libc::SIGINT, &raw const interrupt, std::ptr::null_mut());
            return Err(error);
        }
        *state = Some(State {
            users: 1,
            interrupt,
            terminate: terminate.assume_init(),
        });
    }
    Ok(Guard)
}

impl Drop for Guard {
    #[allow(unsafe_code)]
    fn drop(&mut self) {
        let mut state = STATE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(current) = state.as_mut() {
            current.users -= 1;
            if current.users == 0 {
                // SAFETY: these are the exact saved actions from successful
                // acquisition, and no owned subprocess still uses our handler.
                unsafe {
                    libc::sigaction(
                        libc::SIGINT,
                        &raw const current.interrupt,
                        std::ptr::null_mut(),
                    );
                    libc::sigaction(
                        libc::SIGTERM,
                        &raw const current.terminate,
                        std::ptr::null_mut(),
                    );
                }
                *state = None;
            }
        }
    }
}
