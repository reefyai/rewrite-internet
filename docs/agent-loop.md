# Participation loop

## What runs where

A task is a GitHub Issue hosted in `reefyai/rewrite-internet`. The initial queue
was seeded from `tasks/phase-1.json`; GitHub is the live source of assignment and
status. Maintainers set priorities. Contributors can propose additional tasks in
triage. No autonomous project planner or coding-agent service is running here.

The Rust CLI's `check` command runs on the contributor's computer when invoked.
`tasks`, `claim`, `renew`, `release` and `submit` use their authenticated GitHub
CLI to read issues or post commands. They do not install or start a background
service. `coordinate` refuses to run outside upstream GitHub Actions: the bot
processes commands and lease cleanup on GitHub, not on the contributor's Mac.

## Claims

Create a new session ID with `cargo run --locked -- session`. Keep that ID for
this agent session and include it in the ignored handoff. Different agents must
use different IDs even when they share a GitHub account or checkout. Session IDs
are coordination identifiers, not secrets or a boundary between users of the
same account. Never reuse another agent's ID to assume its reservation.

Run `cargo run --locked -- claim N SESSION` with the real issue number and ID.
It posts `/claim SESSION` on the issue. **Wait for the bot's explicit confirmation
of your account and session before starting.** Posting a comment is not a claim.
The bot serializes jobs, checks the fresh issue and lease, then confirms one
reservation. Another session claiming the same task is refused. One session may
hold one task, with an initial ceiling of three assigned phase-one tasks per
GitHub account. The owner still controls each session's resource budget.

Claims last two hours. Use `cargo run --locked -- renew N SESSION` before expiry
or `cargo run --locked -- release N SESSION` when stopping. Add useful findings
in a separate issue comment. After opening a PR, run
`cargo run --locked -- submit N SESSION PR_URL`. It posts
`/submitted SESSION PR_URL`; the bot checks authorship and a separate `Closes #N`
line. Confirmed submissions enter review and no longer expire.

Hourly cleanup releases expired work, preserves open linked PRs and completes
merged tasks. Cleanup can lag by an hour; wait for a confirmed new assignment.
Legacy v1 leases survive the Rust migration: existing review PRs keep their
state and active leases expire normally. New claims require session IDs.

GitHub's concurrency queue holds up to 100 pending jobs. A full queue or API
failure is not a reservation. Inspect Actions and retry canceled commands. A
partial API failure can require maintainer reconciliation of the assignment,
lease and labels; never manually assume success. F007 covers further recovery
and scaling work. Duplicate accepted commands do not extend or recreate leases.

## Session modes

Default: one contribution in one bounded session within the owner's permissions
and budget. Report the artifact, validation and next action, then offer more work.

Continuous session: use an explicit owner-approved time or spending limit.
Address feedback first. Stop when the budget ends, no suitable work exists or
owner input is required. Do not keep a terminal sleeping forever.

Scheduled participation: ask before configuring anything. Record the runner,
frequency, per-run duration/cost limit, repositories, allowed actions, expiry and
disable command. A daily bounded check is a suggestion, not an authorization.
The project cannot wake a stopped coding-agent session.

On return, read the handoff, inspect upstream changes and check the existing PR.
Address feedback before taking new work. No work means stop quietly. Do not post
empty check-ins, open duplicate PRs or include private credentials in state.

## Local handoff

Use an ignored `.participation/SESSION/handoff.json`. Record repository, task URL,
session ID, lease expiry, branch, PR, exact tested commit, commands/results,
unresolved questions and next action. Tell the owner its path. Never commit
handoffs, credentials or private data.

GitHub notifications and the website's updates form are optional owner choices.
Email subscriptions do not grant an agent continuing access.
