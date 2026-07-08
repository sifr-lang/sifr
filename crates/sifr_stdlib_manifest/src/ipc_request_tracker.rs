use crate::{IpcEnvelope, IpcShutdownMode};
use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IpcRequestTrackerState {
    Open,
    Draining,
    Closed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IpcRequestTrackerError {
    DuplicateRequestId { request_id: u64 },
    UnknownRequestId { request_id: u64 },
    BackpressureFull { max_in_flight: u32 },
    Closing,
    Closed,
}

impl Display for IpcRequestTrackerError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateRequestId { request_id } => {
                write!(formatter, "duplicate IPC request id {request_id}")
            }
            Self::UnknownRequestId { request_id } => {
                write!(formatter, "unknown IPC request id {request_id}")
            }
            Self::BackpressureFull { max_in_flight } => {
                write!(
                    formatter,
                    "IPC in-flight request window is full at {max_in_flight}"
                )
            }
            Self::Closing => formatter.write_str("IPC connection is draining"),
            Self::Closed => formatter.write_str("IPC connection is closed"),
        }
    }
}

impl std::error::Error for IpcRequestTrackerError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IpcRequestTracker {
    max_in_flight: u32,
    state: IpcRequestTrackerState,
    in_flight: BTreeSet<u64>,
}

impl IpcRequestTracker {
    #[must_use]
    pub const fn new(max_in_flight: u32) -> Self {
        Self {
            max_in_flight,
            state: IpcRequestTrackerState::Open,
            in_flight: BTreeSet::new(),
        }
    }

    #[must_use]
    pub const fn max_in_flight(&self) -> u32 {
        self.max_in_flight
    }

    #[must_use]
    pub const fn state(&self) -> IpcRequestTrackerState {
        self.state
    }

    #[must_use]
    pub fn in_flight_len(&self) -> usize {
        self.in_flight.len()
    }

    #[must_use]
    pub fn is_in_flight(&self, request_id: u64) -> bool {
        self.in_flight.contains(&request_id)
    }

    pub fn apply_frame(&mut self, frame: &IpcEnvelope) -> Result<(), IpcRequestTrackerError> {
        match frame {
            IpcEnvelope::Run { request_id, .. } => self.begin_run(*request_id),
            IpcEnvelope::Started { request_id } | IpcEnvelope::Cancel { request_id } => {
                self.require_in_flight(*request_id)
            }
            IpcEnvelope::Completed { request_id, .. } | IpcEnvelope::Failed { request_id, .. } => {
                self.finish_request(*request_id)
            }
            IpcEnvelope::Shutdown { mode } => {
                self.begin_shutdown(*mode);
                Ok(())
            }
            IpcEnvelope::Terminating { .. } => {
                self.close();
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn begin_run(&mut self, request_id: u64) -> Result<(), IpcRequestTrackerError> {
        self.ensure_accepting_new_runs()?;
        if self.in_flight.contains(&request_id) {
            return Err(IpcRequestTrackerError::DuplicateRequestId { request_id });
        }
        if self.in_flight.len() >= self.max_in_flight as usize {
            return Err(IpcRequestTrackerError::BackpressureFull {
                max_in_flight: self.max_in_flight,
            });
        }
        self.in_flight.insert(request_id);
        Ok(())
    }

    pub fn finish_request(&mut self, request_id: u64) -> Result<(), IpcRequestTrackerError> {
        if self.in_flight.remove(&request_id) {
            return Ok(());
        }
        Err(IpcRequestTrackerError::UnknownRequestId { request_id })
    }

    pub fn require_in_flight(&self, request_id: u64) -> Result<(), IpcRequestTrackerError> {
        if self.in_flight.contains(&request_id) {
            return Ok(());
        }
        Err(IpcRequestTrackerError::UnknownRequestId { request_id })
    }

    pub fn begin_shutdown(&mut self, mode: IpcShutdownMode) {
        if self.state == IpcRequestTrackerState::Closed {
            return;
        }
        self.state = IpcRequestTrackerState::Draining;
        if mode == IpcShutdownMode::CancelInFlight {
            self.in_flight.clear();
        }
    }

    pub fn close(&mut self) {
        self.state = IpcRequestTrackerState::Closed;
        self.in_flight.clear();
    }

    fn ensure_accepting_new_runs(&self) -> Result<(), IpcRequestTrackerError> {
        match self.state {
            IpcRequestTrackerState::Open => Ok(()),
            IpcRequestTrackerState::Draining => Err(IpcRequestTrackerError::Closing),
            IpcRequestTrackerState::Closed => Err(IpcRequestTrackerError::Closed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IpcRequestTracker, IpcRequestTrackerError, IpcRequestTrackerState};
    use crate::{IpcEnvelope, IpcShutdownMode, IpcTerminationReason};

    #[test]
    fn run_frames_reserve_in_flight_capacity() {
        let mut tracker = IpcRequestTracker::new(2);

        assert_eq!(tracker.begin_run(1), Ok(()));
        assert_eq!(
            tracker.apply_frame(&IpcEnvelope::Run {
                request_id: 2,
                payload: vec![1, 2],
            }),
            Ok(())
        );
        assert_eq!(tracker.in_flight_len(), 2);
        assert!(tracker.is_in_flight(1));
        assert!(tracker.is_in_flight(2));
    }

    #[test]
    fn duplicate_request_id_is_typed_malformed_evidence() {
        let mut tracker = IpcRequestTracker::new(2);

        assert_eq!(tracker.begin_run(7), Ok(()));
        assert_eq!(
            tracker.begin_run(7),
            Err(IpcRequestTrackerError::DuplicateRequestId { request_id: 7 })
        );
    }

    #[test]
    fn full_window_rejects_new_run_without_losing_existing_requests() {
        let mut tracker = IpcRequestTracker::new(1);

        assert_eq!(tracker.begin_run(1), Ok(()));
        assert_eq!(
            tracker.begin_run(2),
            Err(IpcRequestTrackerError::BackpressureFull { max_in_flight: 1 })
        );
        assert_eq!(tracker.in_flight_len(), 1);
        assert!(tracker.is_in_flight(1));
    }

    #[test]
    fn completed_or_failed_frames_release_capacity() {
        let mut tracker = IpcRequestTracker::new(1);

        assert_eq!(tracker.begin_run(1), Ok(()));
        assert_eq!(
            tracker.apply_frame(&IpcEnvelope::Completed {
                request_id: 1,
                payload: vec![1],
            }),
            Ok(())
        );
        assert_eq!(tracker.begin_run(2), Ok(()));
        assert_eq!(
            tracker.apply_frame(&IpcEnvelope::Failed {
                request_id: 2,
                error: vec![2],
            }),
            Ok(())
        );
        assert_eq!(tracker.in_flight_len(), 0);
    }

    #[test]
    fn unknown_terminal_or_cancel_request_is_typed_malformed_evidence() {
        let mut tracker = IpcRequestTracker::new(2);

        assert_eq!(
            tracker.apply_frame(&IpcEnvelope::Completed {
                request_id: 99,
                payload: Vec::new(),
            }),
            Err(IpcRequestTrackerError::UnknownRequestId { request_id: 99 })
        );
        assert_eq!(
            tracker.apply_frame(&IpcEnvelope::Cancel { request_id: 100 }),
            Err(IpcRequestTrackerError::UnknownRequestId { request_id: 100 })
        );
    }

    #[test]
    fn started_and_cancel_frames_keep_request_in_flight() {
        let mut tracker = IpcRequestTracker::new(2);

        assert_eq!(tracker.begin_run(9), Ok(()));
        assert_eq!(
            tracker.apply_frame(&IpcEnvelope::Started { request_id: 9 }),
            Ok(())
        );
        assert_eq!(
            tracker.apply_frame(&IpcEnvelope::Cancel { request_id: 9 }),
            Ok(())
        );
        assert!(tracker.is_in_flight(9));
    }

    #[test]
    fn drain_shutdown_rejects_new_runs_but_keeps_existing_work() {
        let mut tracker = IpcRequestTracker::new(2);

        assert_eq!(tracker.begin_run(1), Ok(()));
        assert_eq!(
            tracker.apply_frame(&IpcEnvelope::Shutdown {
                mode: IpcShutdownMode::Drain,
            }),
            Ok(())
        );
        assert_eq!(tracker.state(), IpcRequestTrackerState::Draining);
        assert!(tracker.is_in_flight(1));
        assert_eq!(tracker.begin_run(2), Err(IpcRequestTrackerError::Closing));
    }

    #[test]
    fn cancel_in_flight_shutdown_clears_outstanding_work() {
        let mut tracker = IpcRequestTracker::new(2);

        assert_eq!(tracker.begin_run(1), Ok(()));
        tracker.begin_shutdown(IpcShutdownMode::CancelInFlight);

        assert_eq!(tracker.state(), IpcRequestTrackerState::Draining);
        assert_eq!(tracker.in_flight_len(), 0);
    }

    #[test]
    fn terminating_frame_closes_and_clears_tracker() {
        let mut tracker = IpcRequestTracker::new(2);

        assert_eq!(tracker.begin_run(1), Ok(()));
        assert_eq!(
            tracker.apply_frame(&IpcEnvelope::Terminating {
                reason: IpcTerminationReason::Shutdown,
            }),
            Ok(())
        );

        assert_eq!(tracker.state(), IpcRequestTrackerState::Closed);
        assert_eq!(tracker.in_flight_len(), 0);
        assert_eq!(tracker.begin_run(2), Err(IpcRequestTrackerError::Closed));
    }

    #[test]
    fn shutdown_after_terminating_keeps_tracker_closed() {
        let mut tracker = IpcRequestTracker::new(2);

        tracker.close();
        tracker.begin_shutdown(IpcShutdownMode::Drain);

        assert_eq!(tracker.state(), IpcRequestTrackerState::Closed);
        assert_eq!(tracker.begin_run(1), Err(IpcRequestTrackerError::Closed));
    }

    #[test]
    fn non_request_frames_do_not_mutate_tracker_state() {
        let mut tracker = IpcRequestTracker::new(2);

        assert_eq!(tracker.begin_run(1), Ok(()));
        assert_eq!(
            tracker.apply_frame(&IpcEnvelope::Heartbeat { sequence: 7 }),
            Ok(())
        );

        assert_eq!(tracker.state(), IpcRequestTrackerState::Open);
        assert_eq!(tracker.in_flight_len(), 1);
    }

    #[test]
    fn tracker_errors_do_not_render_payload_bytes() {
        assert_eq!(
            IpcRequestTrackerError::BackpressureFull { max_in_flight: 64 }.to_string(),
            "IPC in-flight request window is full at 64"
        );
    }
}
