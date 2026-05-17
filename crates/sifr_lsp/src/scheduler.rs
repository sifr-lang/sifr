#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WorkLane {
    LatencySensitive,
    Formatting,
    Workspace,
}

#[derive(Default)]
pub(crate) struct Scheduler;

impl Scheduler {
    pub(crate) fn lane_for_method(method: &str) -> WorkLane {
        match method {
            "textDocument/formatting" | "textDocument/rangeFormatting" => WorkLane::Formatting,
            "workspace/symbol" | "workspace/diagnostic" => WorkLane::Workspace,
            _ => WorkLane::LatencySensitive,
        }
    }
}
