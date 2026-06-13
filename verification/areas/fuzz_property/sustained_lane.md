# Sustained Fuzzing Lane (Non-blocking)

Purpose:
- run broader/longer fuzz workloads outside local blocking flow
- produce compatibility and crash signal
- feed issue triage and regression corpus updates

Status contract:
- non-blocking for merge decisions in phase 29
- outputs are informational and backlog-oriented

Operational note:
- use the same seed corpus foundations as smoke gates, but larger iteration/time budgets
- every actionable finding must follow the triage/minimization workflow in `verification/policy/fuzz_property.md`
