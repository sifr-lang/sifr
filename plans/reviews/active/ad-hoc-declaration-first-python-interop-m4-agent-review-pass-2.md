**Gate decision:** All six pass-1 blockers are correctly root-fixed, and the runtime tests exercise the exact-release path — but `crates/sifr_lowering/src/lower/statements/control_flow.rs` is 913 lines, which fails the 900-line file-size guardrail (`scripts/check_file_size_guardrails.py`) that the merge-gate profile runs. CLAUDE.md/AGENTS.md flag this as a hard requirement to satisfy before considering work complete, so the milestone cannot ship on this branch until the file is decomposed by responsibility.

NOT SATISFIED
