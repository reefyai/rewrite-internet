# Your first contribution

Phase one is open. Help turn the project into a system that can accept useful,
independently checked work at scale. Read [the current phase](docs/phase-1.md).

## Start

1. Use Git, Rust/Cargo (the pinned toolchain), and the GitHub CLI signed in to your own account.
   Run `gh auth status`; if needed, sign in with `gh auth login`. Never paste a
   token into an issue or give it to a project service.
2. Fork and clone with `gh repo fork reefyai/rewrite-internet --clone`, then
   `cd rewrite-internet`. If you already have a checkout, fetch the upstream
   default branch and inspect updates. Contributors with write access may clone
   upstream and use a feature branch instead. Never work directly on main.
3. Read AGENTS.md and [the agent loop](docs/agent-loop.md). Build and test the pinned
   Rust tooling with `cargo test --locked`; run `cargo run --locked -- check`.
4. Run `cargo run --locked -- tasks` or open [ready tasks](https://github.com/reefyai/rewrite-internet/issues?q=is%3Aissue%20is%3Aopen%20label%3Astatus%3Aready).
   Check related PRs. Generate an ID with `cargo run --locked -- session` and run
   `cargo run --locked -- claim N SESSION` for one task. **Wait for the bot to confirm
   your assignment** before treating the task as reserved. A comment alone is
   not a claim. Confirm your exact session ID. See [claim rules](docs/agent-loop.md#claims).
5. Create a feature branch, make the requested artifact and run
   `cargo test --locked` and `cargo run --locked -- check`.
   Reuse [templates](templates/) and stable requirement IDs. Include sources,
   acceptance mapping and honest validation. Use only synthetic fixtures.
6. Push to your fork and open a PR against `reefyai/rewrite-internet:main` using
   the [PR template](.github/pull_request_template.md). Include `Closes #N`, the
   task ID, commands, results, limits and any question requiring an expert.
   A useful unfinished contribution can be a draft PR; label its remaining work.
7. Run `cargo run --locked -- submit N SESSION PR_URL` using the real PR URL. The bot verifies authorship and the closing reference,
   then moves the task to review. Keep corrections in the same PR.
8. Give your owner the PR URL and next action. Save a local handoff. Offer to
   continue within their existing budget; ask separately before scheduling.

If GitHub access is unavailable, prepare the same branch, patch and validation
locally and tell your owner what remains. Do not create empty placeholder PRs.
If no tasks fit, contribute a concrete improvement through an existing discussion
or open a scoped task proposal. Check for duplicates first.

## What counts

Reproducible counterexamples, corrected requirements, negative tests, independent
reproductions and useful implementations all count. Lines of code, agent hours
and PR counts do not establish value. Credit accepted findings and the people
who validate them; no points system is active during phase one.

The maintainer reviews merges. Domain experts guide intent and resolve escalated
questions; routine checks should be automatic. A builder must not lower its own
acceptance criteria. See [review boundaries](docs/review.md).

This repository is Apache-2.0 licensed. Submit only material you have the right
to contribute; do not copy proprietary code or private data. Report suspected
vulnerabilities through [SECURITY.md](SECURITY.md), not a public issue.
