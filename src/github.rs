use crate::{
    check,
    coordination::{self as c, Lease, State, Task, Verdict},
};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use std::{
    env, fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use uuid::Uuid;

pub const REPO: &str = "reefyai/rewrite-internet";
pub fn api(endpoint: &str, method: &str, body: Option<&Value>) -> Result<Value, String> {
    let mut cmd = Command::new("gh");
    cmd.args([
        "api",
        "--hostname",
        "github.com",
        "--method",
        method,
        "-H",
        "Accept: application/vnd.github+json",
        "-H",
        "X-GitHub-Api-Version: 2022-11-28",
        &format!("repos/{REPO}/{endpoint}"),
    ]);
    if body.is_some() {
        cmd.args(["--input", "-"]).stdin(Stdio::piped());
    }
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("GitHub CLI unavailable: {e}"))?;
    if let Some(body) = body {
        child
            .stdin
            .take()
            .ok_or("Missing API stdin")?
            .write_all(&serde_json::to_vec(body).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }
    let out = child.wait_with_output().map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(format!(
            "GitHub {method} {} failed ({}). Check gh auth status and workflow permissions; no state is assumed confirmed.",
            endpoint.split('?').next().unwrap_or(endpoint),
            out.status
        ));
    }
    if out.stdout.is_empty() {
        Ok(Value::Null)
    } else {
        serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())
    }
}
fn all(endpoint: &str) -> Result<Vec<Value>, String> {
    let mut out = Vec::new();
    for page in 1..=20 {
        let separator = if endpoint.contains('?') { '&' } else { '?' };
        let rows = api(
            &format!("{endpoint}{separator}per_page=100&page={page}"),
            "GET",
            None,
        )?;
        let rows = rows.as_array().ok_or("Expected GitHub array")?;
        out.extend(rows.iter().cloned());
        if rows.len() < 100 {
            return Ok(out);
        }
    }
    Err("Pagination limit reached; no assignment made".into())
}
fn labels(issue: &Value) -> Vec<String> {
    issue["labels"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|l| l["name"].as_str().map(str::to_owned))
        .collect()
}
fn assignees(issue: &Value) -> Vec<String> {
    issue["assignees"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|a| a["login"].as_str().map(str::to_owned))
        .collect()
}
fn number(issue: &Value) -> Result<u64, String> {
    issue["number"]
        .as_u64()
        .ok_or("Missing issue number".into())
}
fn say(n: u64, body: &str) -> Result<Value, String> {
    api(
        &format!("issues/{n}/comments"),
        "POST",
        Some(&json!({"body":body})),
    )
}
fn get_state(n: u64) -> Result<(Option<u64>, State), String> {
    for comment in all(&format!("issues/{n}/comments"))?.into_iter().rev() {
        if let Some(state) = c::parse_state(
            comment["body"].as_str().unwrap_or(""),
            c::trusted_comment(&comment),
        )? {
            return Ok((comment["id"].as_u64(), state));
        }
    }
    Ok((None, State::default()))
}
fn save(n: u64, id: Option<u64>, state: &State, message: &str) -> Result<(), String> {
    let body = format!(
        "{}\n{}\n\n{message}",
        c::MARKER,
        serde_json::to_string(state).map_err(|e| e.to_string())?
    );
    if let Some(id) = id {
        api(
            &format!("issues/comments/{id}"),
            "PATCH",
            Some(&json!({"body":body})),
        )?;
    } else {
        say(n, &body)?;
    }
    Ok(())
}
fn set_status(issue: &Value, status: &str) -> Result<(), String> {
    let mut labels: Vec<_> = labels(issue)
        .into_iter()
        .filter(|n| !n.starts_with("status:"))
        .collect();
    labels.push(format!("status:{status}"));
    api(
        &format!("issues/{}", number(issue)?),
        "PATCH",
        Some(&json!({"labels":labels})),
    )?;
    Ok(())
}
fn linked_prs(n: u64) -> Result<Vec<Value>, String> {
    Ok(all("pulls?state=open")?
        .into_iter()
        .filter(|p| c::closes_issue(p["body"].as_str().unwrap_or(""), n))
        .collect())
}
fn release(
    issue: &Value,
    record: Option<u64>,
    state: &State,
    message: &str,
    done: bool,
    command_id: u64,
) -> Result<(), String> {
    let n = number(issue)?;
    // Never publish 'ready' while the old assignment still exists.
    if let Some(lease) = &state.lease {
        api(
            &format!("issues/{n}/assignees"),
            "DELETE",
            Some(&json!({"assignees":[lease.user]})),
        )?;
    }
    save(
        n,
        record,
        &State {
            lease: None,
            last_command: command_id.max(state.last_command),
        },
        message,
    )?;
    set_status(issue, if done { "done" } else { "ready" })?;
    if done {
        api(
            &format!("issues/{n}"),
            "PATCH",
            Some(&json!({"state":"closed","state_reason":"completed"})),
        )?;
    }
    Ok(())
}
fn process_comment(comment_id: u64) -> Result<(), String> {
    // Fetch authoritative author, body and issue association; dispatch input cannot impersonate a user.
    let comment = api(&format!("issues/comments/{comment_id}"), "GET", None)?;
    if comment["user"]["type"] == "Bot" {
        return Ok(());
    }
    let body = comment["body"].as_str().unwrap_or("");
    let Some(request) = c::parse_command(body) else {
        return Ok(());
    };
    let actor = comment["user"]["login"]
        .as_str()
        .ok_or("Missing comment author")?;
    let prefix = format!("https://api.github.com/repos/{REPO}/issues/");
    let n = comment["issue_url"]
        .as_str()
        .and_then(|u| u.strip_prefix(&prefix))
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or("Wrong comment repository")?;
    let issue = api(&format!("issues/{n}"), "GET", None)?;
    if issue["state"] != "open" || !issue["pull_request"].is_null() {
        return Ok(());
    }
    let (record, state) = get_state(n)?;
    let active: Vec<_> = all(&format!(
        "issues?state=open&assignee={actor}&labels=phase%3A1"
    ))?
    .into_iter()
    .filter(|i| i["number"] != n)
    .collect();
    let mut other_active = active.len() >= 3;
    for other in &active {
        if get_state(number(other)?)?
            .1
            .lease
            .is_some_and(|l| l.session.as_deref() == Some(&request.session))
        {
            other_active = true;
        }
    }
    let task = Task {
        phase_one: labels(&issue).iter().any(|l| l == "phase:1"),
        ready: labels(&issue).iter().any(|l| l == "status:ready"),
        assignees: assignees(&issue),
    };
    let verdict = c::decide(
        &request,
        actor,
        &task,
        &state,
        other_active,
        comment_id,
        Utc::now(),
    );
    let mut updated = State {
        lease: state.lease.clone(),
        last_command: comment_id,
    };
    match verdict {
        Verdict::Claim => {
            api(
                &format!("issues/{n}/assignees"),
                "POST",
                Some(&json!({"assignees":[actor]})),
            )?;
            if !assignees(&api(&format!("issues/{n}"), "GET", None)?)
                .iter()
                .any(|a| a == actor)
            {
                return Err("Assignment was not confirmed".into());
            }
            set_status(&issue, "claimed")?;
            let expires = (Utc::now() + Duration::seconds(c::LEASE_SECONDS)).to_rfc3339();
            updated.lease = Some(Lease {
                user: actor.into(),
                session: Some(request.session.clone()),
                expires: Some(expires.clone()),
                pr: None,
            });
            save(
                n,
                record,
                &updated,
                &format!(
                    "Claim confirmed for @{actor}, session `{}`, until {expires}. Use the same session for renew, release and submit.",
                    request.session
                ),
            )?;
        }
        Verdict::Renew => {
            let lease = updated.lease.as_mut().unwrap();
            lease.expires = Some((Utc::now() + Duration::seconds(c::LEASE_SECONDS)).to_rfc3339());
            save(
                n,
                record,
                &updated,
                &format!("Renewed for @{actor}, session `{}`.", request.session),
            )?;
        }
        Verdict::Release => {
            if !linked_prs(n)?.is_empty() {
                say(
                    n,
                    "An open linked PR exists. Submit it to review before releasing the task.",
                )?;
            } else {
                release(
                    &issue,
                    record,
                    &state,
                    &format!("Released by @{actor}. Ready for a new claim."),
                    false,
                    comment_id,
                )?;
            }
        }
        Verdict::Submitted(pr_number) => {
            let pr = api(&format!("pulls/{pr_number}"), "GET", None)?;
            if pr["state"] != "open"
                || pr["user"]["login"] != actor
                || !c::closes_issue(pr["body"].as_str().unwrap_or(""), n)
            {
                say(
                    n,
                    &format!(
                        "Submission rejected: PR must be open, yours, and contain a separate `Closes #{n}` line."
                    ),
                )?;
                return Ok(());
            }
            let lease = updated.lease.as_mut().unwrap();
            lease.expires = None;
            lease.pr = Some(pr_number);
            save(
                n,
                record,
                &updated,
                &format!(
                    "Submitted by @{actor}: https://github.com/{REPO}/pull/{pr_number}. Task is in review; the claim no longer expires."
                ),
            )?;
            set_status(&issue, "review")?;
        }
        Verdict::Replayed => {}
        _ => {
            say(
                n,
                &format!(
                    "@{actor}: {}. No new reservation was made. See [claim rules](https://github.com/{REPO}/blob/main/docs/agent-loop.md#claims).",
                    verdict.message()
                ),
            )?;
        }
    }
    Ok(())
}
fn cleanup() -> Result<(), String> {
    for issue in all("issues?state=open&labels=phase%3A1")?
        .into_iter()
        .filter(|i| i["pull_request"].is_null())
    {
        let n = number(&issue)?;
        let (record, mut state) = get_state(n)?;
        let Some(lease) = state.lease.clone() else {
            continue;
        };
        if let Some(pr_number) = lease.pr {
            let pr = api(&format!("pulls/{pr_number}"), "GET", None)?;
            if pr["state"] == "closed" {
                let done = !pr["merged_at"].is_null();
                release(
                    &issue,
                    record,
                    &state,
                    if done {
                        "Contribution merged. Task completed."
                    } else {
                        "PR closed without merge. Task returned to ready."
                    },
                    done,
                    state.last_command,
                )?;
            } else {
                set_status(&issue, "review")?;
            }
        } else if c::lease_expired(&lease, Utc::now()) {
            let prs = linked_prs(n)?;
            if let Some(pr) = prs.first() {
                let l = state.lease.as_mut().unwrap();
                l.pr = Some(number(pr)?);
                l.expires = None;
                save(
                    n,
                    record,
                    &state,
                    &format!(
                        "Open linked PR preserved for review: {}",
                        pr["html_url"].as_str().unwrap_or("")
                    ),
                )?;
                set_status(&issue, "review")?;
            } else {
                release(
                    &issue,
                    record,
                    &state,
                    "Claim expired. Task returned to ready; partial findings are welcome in the issue.",
                    false,
                    state.last_command,
                )?;
            }
        }
    }
    Ok(())
}
fn coordinate() -> Result<(), String> {
    if env::var("GITHUB_REPOSITORY").as_deref() != Ok(REPO)
        || env::var("GITHUB_ACTIONS").as_deref() != Ok("true")
    {
        return Err(
            "Coordination runs only in upstream GitHub Actions, not on contributor machines".into(),
        );
    }
    let event: Value = serde_json::from_slice(
        &fs::read(env::var("GITHUB_EVENT_PATH").map_err(|_| "Missing GitHub event")?)
            .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    match env::var("GITHUB_EVENT_NAME").as_deref() {
        Ok("issue_comment") => process_comment(
            event["comment"]["id"]
                .as_u64()
                .ok_or("Missing comment ID")?,
        ),
        Ok("workflow_dispatch") => {
            if let Some(id) = event["inputs"]["comment_id"]
                .as_str()
                .filter(|s| !s.is_empty())
            {
                process_comment(id.parse().map_err(|_| "Invalid comment ID")?)
            } else {
                cleanup()
            }
        }
        Ok("schedule") => cleanup(),
        _ => Err("Unsupported coordination event".into()),
    }
}
fn seed() -> Result<(), String> {
    let label_colors = [
        ("phase:1", "0e8a16"),
        ("priority:P0", "b60205"),
        ("priority:P1", "fbca04"),
        ("status:ready", "0e8a16"),
        ("status:claimed", "1d76db"),
        ("status:review", "5319e7"),
        ("status:done", "6f7c85"),
        ("status:triage", "ededed"),
    ];
    let existing_labels = all("labels")?;
    for (name, color) in label_colors {
        if !existing_labels.iter().any(|l| l["name"] == name) {
            api("labels", "POST", Some(&json!({"name":name,"color":color})))?;
        }
    }
    let existing = all("issues?state=all")?;
    let tasks: Value =
        serde_json::from_slice(&fs::read("tasks/phase-1.json").map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    for task in tasks.as_array().ok_or("Expected phase-one task array")? {
        let id = task["id"].as_str().ok_or("Missing task ID")?;
        let title = format!(
            "[{id}] {}",
            task["title"].as_str().ok_or("Missing task title")?
        );
        if existing.iter().any(|i| i["title"] == title) {
            continue;
        }
        let acceptance = task["acceptance"]
            .as_array()
            .ok_or("Missing acceptance")?
            .iter()
            .map(|v| format!("- [ ] {}", v.as_str().unwrap_or("")))
            .collect::<Vec<_>>()
            .join("\n");
        let non_goals = task["non_goals"]
            .as_array()
            .ok_or("Missing non-goals")?
            .iter()
            .map(|v| format!("- {}", v.as_str().unwrap_or("")))
            .collect::<Vec<_>>()
            .join("\n");
        let priority = task["priority"].as_str().ok_or("Missing priority")?;
        let artifact = task["artifact"].as_str().ok_or("Missing artifact")?;
        let body = format!(
            "## Task\n\n{id} · Phase one · {priority}\n\n## Expected artifact\n\n`{artifact}` (or a justified equivalent)\n\n## Acceptance criteria\n\n{acceptance}\n\n## Non-goals\n\n{non_goals}\n\n## Start\n\nRead [CONTRIBUTING.md](https://github.com/{REPO}/blob/main/CONTRIBUTING.md), check related PRs, generate a distinct agent-session ID, and use `cargo run --locked -- claim N SESSION`. Wait for the bot to confirm your account and session.\n\nAfter opening your PR, use `cargo run --locked -- submit N SESSION PR_URL`. Include exact validation commands and results; identify draft or unmeasured work honestly.\n"
        );
        let issue = api(
            "issues",
            "POST",
            Some(
                &json!({"title":title,"body":body,"labels":["phase:1",format!("priority:{priority}"),"status:ready"]}),
            ),
        )?;
        println!("{}", issue["html_url"].as_str().unwrap_or(""));
    }
    Ok(())
}
pub fn cli(args: Vec<String>) -> Result<(), String> {
    match args.first().map(String::as_str).unwrap_or("help") {
        "check" if args.len() <= 2 => {
            let report = check::check(Path::new(args.get(1).map(String::as_str).unwrap_or(".")))?;
            println!(
                "{}",
                serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
            );
            if !report.ok {
                return Err("Contribution check failed".into());
            }
            Ok(())
        }
        "tasks" if args.len() == 1 => {
            let mut tasks: Vec<_> = all("issues?state=open&labels=phase%3A1,status%3Aready")?
                .into_iter()
                .filter(|i| i["pull_request"].is_null() && assignees(i).is_empty())
                .collect();
            tasks.sort_by_key(|i| i["title"].as_str().unwrap_or("").to_owned());
            if tasks.is_empty() {
                println!("No ready tasks. Check your existing PR or stop quietly.");
            }
            for t in tasks {
                println!(
                    "#{} {}\n{}\n",
                    number(&t)?,
                    t["title"].as_str().unwrap_or(""),
                    t["html_url"].as_str().unwrap_or("")
                );
            }
            Ok(())
        }
        "session" if args.len() == 1 => {
            println!("{}", Uuid::new_v4());
            Ok(())
        }
        "claim" | "renew" | "release" | "submit" => {
            let submitted = args[0] == "submit";
            if args.len() != if submitted { 4 } else { 3 } {
                return Err(
                    "Use: ri claim|renew|release ISSUE SESSION; ri submit ISSUE SESSION PR_URL"
                        .into(),
                );
            }
            let n: u64 = args[1].parse().map_err(|_| "Invalid issue number")?;
            if n == 0 || !c::valid_session(&args[2]) {
                return Err(
                    "Issue must be positive and session must be a canonical UUID from ri session"
                        .into(),
                );
            }
            let body = if submitted {
                if c::parse_pr_url(&args[3]).is_none() {
                    return Err("Expected an upstream PR URL".into());
                }
                format!("/submitted {} {}", args[2], args[3])
            } else {
                format!("/{} {}", args[0], args[2])
            };
            let response = say(n, &body)?;
            println!(
                "{}\nWait for the bot confirmation; posting a command does not reserve work.",
                response["html_url"].as_str().unwrap_or("")
            );
            Ok(())
        }
        "coordinate" if args.len() == 1 => coordinate(),
        "seed" if args.len() == 1 => seed(),
        _ => {
            println!(
                "ri check [path] | tasks | session | claim ISSUE SESSION | renew ISSUE SESSION | release ISSUE SESSION | submit ISSUE SESSION PR_URL\nThe coordinate subcommand is server-side only; seed is for maintainers. Nothing runs in the background."
            );
            Ok(())
        }
    }
}
