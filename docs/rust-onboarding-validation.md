# Rust contribution-flow retest

The initial negative/positive test used the staged Rust migration branch.
Migration PR #13 is now merged; this pilot targets main. The normal main-branch
Rust gate and default claim handler have also been exercised successfully.
Exact immutable run references are recorded below.

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

## Main-branch cutover verification

- Main merge: `f6d084187e7652b110536b9b978078ca05a5c42e`.
- [Main CI](https://github.com/reefyai/rewrite-internet/actions/runs/46047852012): both jobs passed.
- [Pilot against main](https://github.com/reefyai/rewrite-internet/actions/runs/46048109886): both jobs passed at the corrected specimen revision recorded in the machine report.
- [Normal claim](https://github.com/reefyai/rewrite-internet/actions/runs/46048167799) and [release](https://github.com/reefyai/rewrite-internet/actions/runs/46048340899): the temporary reservation returned to ready with no assignee.
- [Pilot session claim](https://github.com/reefyai/rewrite-internet/actions/runs/46048259133) and [submission](https://github.com/reefyai/rewrite-internet/actions/runs/46048458911): the existing PR returned to review with no lease expiry.

The website's published prompt matched main's JOIN.txt byte-for-byte. Original
branch protections were restored and compared unchanged immediately after the
owner-approved one-time merge override. The auto-closed-task cleanup edge case
was reconciled and recorded in task #7; it is not claimed fixed by this port.

This post-cutover commit updates history/provenance only. The Rust implementation
and DNS specification/evidence bytes are unchanged from the tested revisions.
