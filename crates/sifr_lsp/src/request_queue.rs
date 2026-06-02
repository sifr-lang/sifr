use crate::scheduler::{Scheduler, WorkLane};
use lsp_server::RequestId;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

const FAIRNESS_INTERVAL: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScheduledRequest {
    id: RequestId,
    key: String,
    method: String,
    lane: WorkLane,
}

impl ScheduledRequest {
    pub(crate) fn id(&self) -> &RequestId {
        &self.id
    }

    pub(crate) fn key(&self) -> &str {
        &self.key
    }

    pub(crate) fn method(&self) -> &str {
        &self.method
    }

    pub(crate) fn lane(&self) -> WorkLane {
        self.lane
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct QueuedRequest {
    id: RequestId,
    key: String,
    method: String,
    lane: WorkLane,
    sequence: u64,
}

#[derive(Default)]
pub(crate) struct RequestQueue {
    queued: BTreeMap<WorkLane, VecDeque<QueuedRequest>>,
    in_flight: BTreeMap<String, ScheduledRequest>,
    queued_keys: BTreeSet<String>,
    shutdown_requested: bool,
    next_sequence: u64,
    priority_dispatches_since_fairness: usize,
    fairness_cursor: usize,
}

impl RequestQueue {
    pub(crate) fn enqueue(
        &mut self,
        id: &RequestId,
        method: &str,
        lane: WorkLane,
    ) -> Result<(), &'static str> {
        if self.shutdown_requested {
            return Err("server is shutting down");
        }
        let key = request_key(id);
        let request = QueuedRequest {
            id: id.clone(),
            key: key.clone(),
            method: method.to_string(),
            lane,
            sequence: self.next_sequence,
        };
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.queued.entry(lane).or_default().push_back(request);
        self.queued_keys.insert(key);
        Ok(())
    }

    pub(crate) fn start_next(&mut self) -> Option<ScheduledRequest> {
        let lane = self.select_next_lane()?;
        let request = self.queued.get_mut(&lane)?.pop_front()?;
        if self.queued.get(&lane).is_some_and(VecDeque::is_empty) {
            self.queued.remove(&lane);
        }
        self.queued_keys.remove(&request.key);
        let scheduled = ScheduledRequest {
            id: request.id,
            key: request.key,
            method: request.method,
            lane: request.lane,
        };
        self.in_flight
            .insert(scheduled.key.clone(), scheduled.clone());
        Some(scheduled)
    }

    pub(crate) fn finish(&mut self, id: &RequestId) {
        self.in_flight.remove(&request_key(id));
    }

    pub(crate) fn remove_pending(&mut self, id: &RequestId) -> bool {
        let key = request_key(id);
        for lane_queue in self.queued.values_mut() {
            if let Some(index) = lane_queue.iter().position(|request| request.key == key) {
                lane_queue.remove(index);
                self.queued_keys.remove(&key);
                return true;
            }
        }
        self.in_flight.remove(&key).is_some()
    }

    pub(crate) fn begin_shutdown(&mut self) {
        self.shutdown_requested = true;
        self.queued.clear();
        self.queued_keys.clear();
        self.in_flight.clear();
    }

    fn select_next_lane(&mut self) -> Option<WorkLane> {
        if self.priority_dispatches_since_fairness >= FAIRNESS_INTERVAL {
            if let Some(lane) = self.next_fair_lane() {
                self.priority_dispatches_since_fairness = 0;
                return Some(lane);
            }
        }
        let lane = Scheduler::LANES
            .into_iter()
            .find(|lane| self.queued.get(lane).is_some_and(|queue| !queue.is_empty()))?;
        self.priority_dispatches_since_fairness =
            self.priority_dispatches_since_fairness.saturating_add(1);
        Some(lane)
    }

    fn next_fair_lane(&mut self) -> Option<WorkLane> {
        for offset in 1..=Scheduler::LANES.len() {
            let index = (self.fairness_cursor + offset) % Scheduler::LANES.len();
            let lane = Scheduler::LANES[index];
            if self
                .queued
                .get(&lane)
                .is_some_and(|queue| !queue.is_empty())
            {
                self.fairness_cursor = index;
                return Some(lane);
            }
        }
        None
    }
}

pub(crate) fn request_key(id: &RequestId) -> String {
    format!("{id:?}")
}

#[cfg(test)]
mod tests {
    use super::RequestQueue;
    use crate::scheduler::WorkLane;
    use lsp_server::RequestId;

    #[test]
    fn scheduler_prefers_latency_but_eventually_services_background() {
        let mut queue = RequestQueue::default();
        for id in 0..12 {
            queue
                .enqueue(
                    &RequestId::from(id),
                    "textDocument/completion",
                    WorkLane::LatencySensitive,
                )
                .expect("latency request should enqueue");
        }
        queue
            .enqueue(
                &RequestId::from(100),
                "sifr/backgroundIndex",
                WorkLane::Background,
            )
            .expect("background request should enqueue");

        let scheduled = (0..6)
            .map(|_| queue.start_next().expect("request should schedule").lane())
            .collect::<Vec<_>>();

        assert_eq!(
            scheduled,
            vec![
                WorkLane::LatencySensitive,
                WorkLane::LatencySensitive,
                WorkLane::LatencySensitive,
                WorkLane::LatencySensitive,
                WorkLane::Background,
                WorkLane::LatencySensitive
            ]
        );
    }

    #[test]
    fn scheduler_rotates_fairness_lane_across_nonempty_queues() {
        let mut queue = RequestQueue::default();
        for id in 0..10 {
            queue
                .enqueue(
                    &RequestId::from(id),
                    "textDocument/hover",
                    WorkLane::LatencySensitive,
                )
                .expect("latency request should enqueue");
        }
        queue
            .enqueue(
                &RequestId::from(100),
                "workspace/diagnostic",
                WorkLane::Workspace,
            )
            .expect("workspace request should enqueue");
        queue
            .enqueue(
                &RequestId::from(101),
                "sifr/backgroundIndex",
                WorkLane::Background,
            )
            .expect("background request should enqueue");

        let scheduled = (0..10)
            .map(|_| queue.start_next().expect("request should schedule").lane())
            .collect::<Vec<_>>();

        assert_eq!(scheduled[4], WorkLane::Workspace);
        assert_eq!(scheduled[9], WorkLane::Background);
    }

    #[test]
    fn cancellation_removes_queued_request_before_dispatch() {
        let mut queue = RequestQueue::default();
        let id = RequestId::from(42);
        queue
            .enqueue(&id, "workspace/diagnostic", WorkLane::Workspace)
            .expect("request should enqueue");

        assert!(queue.remove_pending(&id));
        assert!(queue.start_next().is_none());
    }
}
