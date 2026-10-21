//! Deadline decisions are independent of filesystem and thread scheduling speed.
use std::time::{Duration, Instant, SystemTime};

pub(super) struct LeaseDeadline {
    started: Instant,
    idle_since: Instant,
    last_change: Option<SystemTime>,
    hard_limit: Duration,
    idle_limit: Duration,
}

impl LeaseDeadline {
    pub(super) fn new(
        started: Instant,
        last_change: Option<SystemTime>,
        hard_limit: Duration,
        idle_limit: Duration,
    ) -> Self {
        Self {
            started,
            idle_since: started,
            last_change,
            hard_limit,
            idle_limit,
        }
    }

    /// Return the next bounded poll delay, or the deadline that expired.
    pub(super) fn next_poll(
        &mut self,
        now: Instant,
        changed: Option<SystemTime>,
    ) -> Result<Duration, Duration> {
        if changed.is_some() && changed != self.last_change {
            self.last_change = changed;
            self.idle_since = now;
        }
        let hard_remaining = self.hard_limit.saturating_sub(now - self.started);
        let idle_remaining = self.idle_limit.saturating_sub(now - self.idle_since);
        if hard_remaining.is_zero() {
            return Err(self.hard_limit);
        }
        if idle_remaining.is_zero() {
            return Err(self.idle_limit);
        }
        Ok(Duration::from_millis(10)
            .min(hard_remaining)
            .min(idle_remaining))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    #[test]
    fn idle_boundary_ignores_unchanged_and_unavailable_observations() {
        let started = Instant::now();
        let note = SystemTime::UNIX_EPOCH;
        let mut deadline = LeaseDeadline::new(started, Some(note), ms(250), ms(40));
        assert_eq!(deadline.next_poll(started, Some(note)), Ok(ms(10)));
        assert_eq!(deadline.next_poll(started + ms(30), None), Ok(ms(10)));
        assert_eq!(deadline.next_poll(started + ms(39), Some(note)), Ok(ms(1)));
        assert_eq!(deadline.next_poll(started + ms(40), None), Err(ms(40)));
    }

    #[test]
    fn renewed_idle_deadline_never_extends_hard_limit() {
        let started = Instant::now();
        let note = SystemTime::UNIX_EPOCH;
        let mut deadline = LeaseDeadline::new(started, None, ms(140), ms(40));
        for time in [30, 60, 90, 120] {
            assert_eq!(
                deadline.next_poll(started + ms(time), Some(note + ms(time))),
                Ok(ms(10)),
            );
        }
        assert_eq!(
            deadline.next_poll(started + ms(139), Some(note + ms(120))),
            Ok(ms(1)),
        );
        assert_eq!(
            deadline.next_poll(started + ms(140), Some(note + ms(140))),
            Err(ms(140)),
        );
    }

    #[test]
    fn hard_deadline_wins_when_both_expire_and_renewal_restores_only_idle() {
        let started = Instant::now();
        let note = SystemTime::UNIX_EPOCH;
        let mut both = LeaseDeadline::new(started, None, ms(140), ms(40));
        assert_eq!(both.next_poll(started + ms(200), None), Err(ms(140)));
        let mut renewed = LeaseDeadline::new(started, None, ms(140), ms(40));
        assert_eq!(renewed.next_poll(started + ms(50), Some(note)), Ok(ms(10)),);
        assert_eq!(renewed.next_poll(started + ms(89), None), Ok(ms(1)));
        assert_eq!(renewed.next_poll(started + ms(90), Some(note)), Err(ms(40)),);
    }
}
