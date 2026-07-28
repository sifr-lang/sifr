use std::collections::VecDeque;

use sifr_runtime::interop::{
    CallbackBackpressure, CallbackOverflow, CallbackShutdown, ThreadsafeCallbackPolicy,
};

use super::support::{EventCallback, SubscriptionError};

pub struct CallbackQueue {
    events: VecDeque<String>,
    capacity: Option<usize>,
    overflow: CallbackOverflow,
    shutdown: CallbackShutdown,
}

impl CallbackQueue {
    pub fn from_policy(policy: ThreadsafeCallbackPolicy) -> Result<Self, SubscriptionError> {
        let capacity = match policy.backpressure {
            CallbackBackpressure::Direct => Some(0),
            CallbackBackpressure::Bounded(capacity) if capacity > 0 => Some(capacity),
            CallbackBackpressure::Bounded(_) => {
                return Err(SubscriptionError::new(
                    "bounded callback queue requires positive capacity",
                ));
            }
            CallbackBackpressure::Unbounded => None,
        };
        Ok(Self {
            events: VecDeque::new(),
            capacity,
            overflow: policy.overflow,
            shutdown: policy.shutdown,
        })
    }

    pub fn enqueue(&mut self, event: &str) -> Result<(), SubscriptionError> {
        let full = self
            .capacity
            .is_some_and(|capacity| self.events.len() >= capacity);
        if !full {
            self.events.push_back(event.to_string());
            return Ok(());
        }
        match self.overflow {
            CallbackOverflow::Error => {
                Err(SubscriptionError::new("bounded callback queue overflow"))
            }
            CallbackOverflow::DropOldest => {
                self.events.pop_front();
                self.events.push_back(event.to_string());
                Ok(())
            }
            CallbackOverflow::DropNewest => Ok(()),
        }
    }

    pub fn pending(&self) -> usize {
        self.events.len()
    }

    pub fn shutdown(
        mut self,
        callback: &EventCallback,
    ) -> Result<(usize, &'static str), SubscriptionError> {
        match self.shutdown {
            CallbackShutdown::Drain => {
                let mut drained = 0;
                while let Some(event) = self.events.pop_front() {
                    invoke_expected_success(callback, event)?;
                    drained += 1;
                }
                Ok((drained, "drain"))
            }
            CallbackShutdown::Cancel => {
                self.events.clear();
                Ok((0, "cancel"))
            }
            CallbackShutdown::DetachForbidden => {
                if self.events.is_empty() {
                    Ok((0, "detach-forbidden"))
                } else {
                    Err(SubscriptionError::new(
                        "detach-forbidden callback queue retained pending events",
                    ))
                }
            }
        }
    }
}

fn invoke_expected_success(
    callback: &EventCallback,
    event: String,
) -> Result<(), SubscriptionError> {
    match callback.call((event,)) {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) => Err(SubscriptionError::new(format!(
            "drain callback queue: {error}"
        ))),
        Err(error) => Err(SubscriptionError::context("drain callback queue", error)),
    }
}
