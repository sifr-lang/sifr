#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum WorkLane {
    LatencySensitive,
    Formatting,
    Workspace,
    Background,
}

#[derive(Default)]
pub(crate) struct Scheduler;

impl Scheduler {
    pub(crate) fn lane_for_method(method: &str) -> WorkLane {
        match method {
            "textDocument/formatting" | "textDocument/rangeFormatting" => WorkLane::Formatting,
            "workspace/symbol" | "workspace/diagnostic" => WorkLane::Workspace,
            // Internal lane sentinel for scheduler/background-index tests. the scheduler contract wires
            // real background worker entrypoints.
            "sifr/backgroundIndex" => WorkLane::Background,
            _ => WorkLane::LatencySensitive,
        }
    }

    pub(crate) const LANES: [WorkLane; 4] = [
        WorkLane::LatencySensitive,
        WorkLane::Formatting,
        WorkLane::Workspace,
        WorkLane::Background,
    ];
}
