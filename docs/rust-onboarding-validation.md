# Rust contribution-flow retest

This is a staged migration test. The existing fork PR targets
`codex/rust-coordination` until its migration PR is reviewed and merged. It must
be retargeted to main after cutover. No live-main Rust cutover is claimed yet.

## Reproduce

1. Clone the contributor fork into an empty directory.
2. Fetch upstream's Rust migration and merge it into the existing pilot branch.
3. Run `cargo test --locked` and `cargo run --locked -- check`: 33 tests and the
   complete DNS evidence plan pass without Node/npm.
4. Omit the DNS-006 check from `evidence/dns-message.json`. Run the same checker:
   it reports `uncovered requirement DNS-006`.
5. Push the negative revision to the existing fork PR against the Rust base.
   The base-commit Rust gate rejects it for that exact missing requirement.
6. Restore the DNS-006 check. Run the same commands; the tests and check pass.
   Push the correction and verify both GitHub jobs.

## Provenance

[Machine-readable record](../validation/rust-onboarding.json) contains the exact
Rust base, negative candidate, run links and document hashes. Local
[negative](../validation/rust-onboarding-negative.json) and
[corrected](../validation/rust-onboarding-positive.json) results are retained.

The original `onboarding-validation.md` records the historical pre-Rust bootstrap;
its commands are not current contributor prerequisites. All runtime DNS checks
remain proposed, not-run. This retest validates contribution structure and workflow.

The migration task separately tests two live agent sessions on one GitHub account:
one is confirmed and the second is refused. A real Rust `/submitted` command moves
the migration PR into review. Those dispatches fetch authoritative GitHub comments;
a dispatch input cannot supply a fake author or arbitrary command.
