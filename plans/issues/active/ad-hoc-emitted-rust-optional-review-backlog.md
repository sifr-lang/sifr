# Optional emitted-Rust review suggestions — separate backlog

Status: optional, uncommitted scope; NOT phase acceptance requirements.
Owner: performance / issue3776. No implementation authorized by this record.

The user's fixed-scope direction keeps new optional review suggestions outside
the remaining emitted-Rust phase packages. They must not block B52–B54, integration
or closure unless a separately demonstrated violation of an existing criterion
is recorded. Genuinely new requirements need user direction.

Source: [B52–B54 grouped review](https://github.com/sifr-lang/sifr/issues/3776#issuecomment-5591731875),
candidate `6a59be12ec4076e2801652ffbf99250e67e5a258`, SATISFIED, no blockers.

- Optionally document the role of the public fresh-session
  `generate_stdlib_module_body` API, whose present internal callers are tests.
  No compatibility requirement or code-removal obligation follows.
- Optionally link the SQL missing-host error guard's comment to the prepared
  registry demand predicate. The structured error and lifecycle invariant already
  exist and were approved; this is comment clarity only.
- For future dependency review, note that removing proc-macro2 `span-locations`
  makes the gated byte-range API fail compilation. The review's suggestion also
  mentioned silent degradation, but its own API analysis establishes compile
  failure, not silent syntax acceptance. No present defect or new test requirement.

No suggestion was implemented or promoted into a phase item in this batch.
