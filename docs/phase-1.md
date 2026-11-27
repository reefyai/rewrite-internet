# Phase one: build the contribution system

Goal: let a new contributor find bounded work, submit a useful artifact and
receive a clear verdict, without a human approving every intermediate step.

The [ready queue](https://github.com/reefyai/rewrite-internet/issues?q=is%3Aissue%20is%3Aopen%20label%3Astatus%3Aready)
is the live source of task status. `tasks/phase-1.json` records task definitions,
priorities and acceptance criteria; it is not a second assignment database.
`cargo run --locked -- tasks` reads live GitHub state. Work can be claimed with the Rust CLI and a distinct session ID.

P0 foundations: expert decision boundaries, spec structure, independent checking,
evidence and data preservation. P1 work covers performance, task coordination,
returning contributors and adversarial validation. Each issue has its own scope.

Templates and schemas start at v0.1.0 and remain proposals until reviewed.
Coordination tooling is implemented in Rust. Rust is also the direction for new
production components.
Linux, hardware firmware, drivers and vendor hardware support remain initially.
Reefy's A/B update model is the initial path to delivering builds on real devices.

## Exit criteria

- A fresh contributor completes a fork-to-PR path, including claim, validation,
  submission and a resumable handoff.
- A common spec structure and evidence contract have reviewed v0.1 versions.
- Two different worked examples exercise the contracts: one stateless protocol
  boundary and one stateful migration/restore boundary.
- Negative fixtures demonstrate rejection of incomplete or inconsistent evidence.
- Maintainer and expert responsibilities, escalation triggers and release gates
  are explicit. Claims expire and reviewed work has a recorded outcome.

Maintainers record the evidence and decision to enter phase two here through a
reviewed PR. Passing the bootstrap tests alone does not settle phase one. Avoid
unbounded architecture documents; every proposal needs an example and an
acceptance condition. Component implementation tasks follow reviewed contracts.
