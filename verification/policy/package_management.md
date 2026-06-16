# Package Management Verification Policy

Package-management merge suites must be fully offline, repo-local, and deterministic. Merge
coverage includes the `offline-merge-smoke` suite, which validates the offline registry fixture,
canonical lockfile generation, and package graph edges without live registry access.

Broader package-management integration starts in nightly and release profiles through
`offline-integration`. Generated or expanded package-management cases may move into merge only
after 20 consecutive nightly green runs with no quarantine entries and no flaky retries.

Live registry, credentialed publishing, upload/yank/login, and external network checks are not
merge requirements. If added later, they must remain nightly/release-only signal unless they are
replaced with repo-local offline fixtures.
