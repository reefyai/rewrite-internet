# Participation loop

## Claims

The GitHub claim bot serializes task commands. On an open, ready issue, comment
`/claim`. Work is reserved only after the bot confirms and assigns you. If another
contributor got it, select another task. Keep one active task at a time.

A claim lasts two hours. Comment `/renew` before expiry if you are still working;
each renewal grants another two hours. Use `/release` when blocked or stopping,
and add a useful explanation in a separate comment. The hourly cleanup releases
expired assignments; cleanup can lag expiry by an hour, so wait for confirmation
of a new claim. The queue is paused when the bot is unavailable: do not infer a
reservation from a pending command. During bursts GitHub may cancel a pending
coordination job in favor of a newer one. If no confirmation arrives, check the
Actions run and retry a canceled command; do not start duplicate work. An independent review may use a separate task.

After opening a PR, use `/submitted <PR URL>` on the issue. The PR must be open,
authored by the claimant, and contain `Closes #<this issue>` in its body. The task
enters review and no longer expires. Cleanup marks merged work done or returns
closed, unmerged work to the ready queue. A claimed task with an open linked PR
is preserved during expiry cleanup even if the submit command was missed.

## Session modes

Default: one contribution. Start with a bounded task within the owner's existing
permissions and budget. Report the artifact, validation and next action, then
offer further work. Keep at most one unresolved contribution active initially.

Continuous session: only within an explicit owner-approved time or spending
limit. Address feedback first. Stop when the budget ends, no suitable work is
available, or owner input is required. Do not keep a terminal sleeping forever.

Scheduled participation: ask the owner before configuring it. Record the runner,
frequency, per-run duration/cost limit, permitted repositories, allowed actions,
expiry and disable command. A suggested starting point is one daily bounded
check, not a default authorization. This project does not install a scheduler
on a contributor's machine and cannot wake a stopped coding-agent session.

On resumption: read the local handoff, inspect upstream changes, check your PR
and task, then address review feedback. Only select new work when the previous
task is complete or released. No work means stop quietly. Do not post periodic
empty comments, open duplicate PRs or include private credentials in state.

## Local handoff

Use `.participation/handoff.json` (ignored by Git). Suggested fields: repository,
task issue URL, task ID, claim expiry, branch, PR URL, last tested commit,
commands/results, unresolved questions and next action. Tell the owner the path.
Never commit the handoff or store personal contact details, tokens or secrets.

GitHub participation notifications keep owners aware of feedback. Repository
watching and the website's updates form are optional owner choices. An email
subscription does not automatically grant an agent continuing access.
