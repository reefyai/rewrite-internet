# First contribution exercise

This draft exercises F008 using the published contributor instructions and a
fresh fork checkout. It does not implement or certify a DNS parser.

## Reproduction

1. Fork `reefyai/rewrite-internet` and clone the fork into an empty directory.
2. Run `npm ci --ignore-scripts` and `npm test` on the unchanged checkout.
3. Read `npm run tasks`, comment `/claim` on task #8, and wait for assignment.
4. Add the DNS specimen and proposed evidence plan. Omit DNS-006 coverage.
5. Run `node scripts/check.mjs`: it rejects the missing requirement.
6. Open a draft fork PR. The base-commit GitHub check rejects the same omission.
7. Restore the planned DNS-006 check. Run the same commands, including the
   upstream checkout's checker against this candidate: all structural checks pass.
8. Push the corrected plan and verify GitHub CI. Submit the PR URL on the task
   and preserve an ignored local handoff. No recurring agent job is installed.

## Provenance and observed results

- Bootstrap base: `6153dc974fb9868ac6ae333949c316087db9bb94`.
- Negative candidate: `5bc4a12d37b932739e1f0fb27c44079a6521453a`.
- [Negative CI run](https://github.com/reefyai/rewrite-internet/actions/runs/46038636652)
  fails specifically with `uncovered requirement DNS-006`.
- [Local negative result](../validation/onboarding-negative.json) and
  [local corrected result](../validation/onboarding-positive.json) are retained.
- [Machine-readable provenance](../validation/onboarding.json) records actual
  revisions, document hashes and commands. Final remote results are recorded in
  the PR validation section after GitHub finishes.

The final branch retains the negative revision and the correction so this gate
can be replayed. The normal suite also has permanent missing-coverage, stale
version, false-measurement and unsafe-path regression cases.

## Limits

This tests one authenticated contributor fork, not an unrelated user's account
or a high-volume arrival burst. Claim contention/renewal policy has unit tests;
API failures and queue behavior remain F007 work. No runtime DNS tests, security
scan, benchmark, QEMU/hardware check or expert approval is claimed.
