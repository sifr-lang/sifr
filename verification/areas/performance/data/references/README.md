# Named performance reference profiles

A profile names one measured reference machine and execution configuration.
Select it explicitly with `SIFR_PERFORMANCE_REFERENCE` or `--reference-profile`.
Unknown or mismatched profiles fail qualification; there is no cross-host
default. Unprofiled measurements remain exploratory.

The first Linux reference is `linux-i7-4720hq-12gb-dev-v1`: Intel i7-4720HQ,
four physical/eight logical CPUs, 12 GB installed RAM, two Cargo jobs, Rust
1.98.1, dev compiler. Actual usable RAM, OS/kernel, CPU identity, concurrency,
toolchain, build environment, storage and benchmark input hashes are captured
from the machine, not inferred from the name. Its JSON exists only after an
approved full-corpus capture succeeds.

The existing `../baselines.json`, `../budgets.json`, `../work_budgets.json`
and `../trend/current.json` remain historical Mac evidence. They do not record
an exact CPU model and RAM capacity, so those details must not be retroactively
invented or used to qualify a new named Mac profile. A future Mac capture must
measure its own identity and use a distinct named profile.

Profiles are immutable captures. A new version requires explicit approved
capture from a merged compiler reference, with only benchmark-tooling changes
allowed relative to that compiler. The current candidate cannot become its own
reference. Each profile binds an approved full-manifest receipt, baseline,
shared-policy budgets and measured identity in one atomically published JSON.

When a verification profile selects this performance area, the named reference
owns its measurement Cargo concurrency. The area explicitly sets and reports
the reference's `cargo_jobs` for its producer and budget checks, even when the
outer profile uses a different native-build worker count. Other host, toolchain,
power and input comparisons remain enforced. Selecting a reference does not
change the machine's governor; establish its recorded policy before admission.
