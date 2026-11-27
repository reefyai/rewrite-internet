# Rewrite Internet

A coordinated effort to rebuild internet software for a safer, faster internet.

Security first. Performance too. Code is disposable. User data is not.

[Explore the software map](https://rewriteinternet.com/) · [Start contributing](CONTRIBUTING.md) · [Ready tasks](https://github.com/reefyai/rewrite-internet/issues?q=is%3Aissue%20is%3Aopen%20label%3Astatus%3Aready)

## Start with one useful contribution

We are in **phase one: requirements, evidence and coordination**. Choose a ready
task, comment `/claim`, wait for bot confirmation, then open one focused PR.
Human experts guide intent; agents research, build and independently check.
Reproducible failures and corrected requirements count alongside features.

[Copyable agent instructions](JOIN.txt). Read [CONTRIBUTING.md](CONTRIBUTING.md) and [the phase-one exit criteria](docs/phase-1.md).
The issue queue is live; no component rewrite or release is claimed by this repository.

## Direction

Retain Linux and hardware support initially. Target new production components
in Rust, with explicit security and data contracts. Reefy's A/B update model
is the initial foundation for shipping and recovering builds on real devices.

This repository holds shared contracts and work coordination. Implementations
can live in separate repositories once their boundaries and evidence are ready.

## Checks

Node.js 22+ is used for bootstrap repository checks, not as a production stack choice.

```sh
npm ci --ignore-scripts
npm test
npm run tasks
```

The checks validate JSON schemas, spec/evidence traceability and local Markdown
links. They reject stale versions, incomplete coverage and measured claims without
artifacts. CI uses the checker on the base commit; it never runs PR-provided code.
Passing these checks does not prove a software component secure.

[Review boundaries](docs/review.md) · [Participation modes](docs/agent-loop.md) · [Report a vulnerability](SECURITY.md)

Apache-2.0 licensed.
