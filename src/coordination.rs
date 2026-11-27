use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const LEASE_SECONDS: i64 = 7200;
pub const MARKER: &str = "<!-- ri-lease:v2 -->";
pub const LEGACY_MARKER: &str = "<!-- ri-lease:v1 -->";
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Lease {
    pub user: String,
    #[serde(default)]
    pub session: Option<String>,
    pub expires: Option<String>,
    pub pr: Option<u64>,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct State {
    pub lease: Option<Lease>,
    #[serde(default)]
    pub last_command: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Claim,
    Renew,
    Release,
    Submitted(u64),
}
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub command: Command,
    pub session: String,
}
#[derive(Clone, Debug, Default)]
pub struct Task {
    pub phase_one: bool,
    pub ready: bool,
    pub assignees: Vec<String>,
}
#[derive(Debug, PartialEq)]
pub enum Verdict {
    Claim,
    Renew,
    Release,
    Submitted(u64),
    AlreadyYours,
    Unavailable,
    NotOwner,
    InReview,
    Expired,
    OtherActive,
    NotTask,
    Replayed,
}
impl Verdict {
    pub fn message(&self) -> &'static str {
        match self {
            Self::AlreadyYours => "already confirmed for this session",
            Self::Unavailable => {
                "claimed or unavailable; a different session using your account also counts as another claimant"
            }
            Self::NotOwner => "this account and session do not own the lease",
            Self::InReview => "work is already in review",
            Self::Expired => "lease expired; wait for cleanup or release it",
            Self::OtherActive => {
                "this session already holds work, or this account reached the three-task limit"
            }
            Self::NotTask => "not a phase-one task",
            Self::Replayed => "command already processed or superseded",
            _ => "accepted",
        }
    }
}
pub fn valid_session(id: &str) -> bool {
    Uuid::parse_str(id).is_ok_and(|u| u.to_string() == id)
}
pub fn parse_command(body: &str) -> Option<Request> {
    if body.trim().lines().count() != 1 {
        return None;
    }
    let parts: Vec<_> = body.split_whitespace().collect();
    if !(parts.len() == 2 || parts.len() == 3) || !valid_session(parts[1]) {
        return None;
    }
    let command = match (parts[0], parts.len()) {
        ("/claim", 2) => Command::Claim,
        ("/renew", 2) => Command::Renew,
        ("/release", 2) => Command::Release,
        ("/submitted", 3) => Command::Submitted(parse_pr_url(parts[2])?),
        _ => return None,
    };
    Some(Request {
        command,
        session: parts[1].into(),
    })
}
pub fn parse_pr_url(url: &str) -> Option<u64> {
    let id = url.strip_prefix("https://github.com/reefyai/rewrite-internet/pull/")?;
    if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    id.parse::<u64>().ok().filter(|n| *n > 0)
}
pub fn closes_issue(body: &str, number: u64) -> bool {
    body.lines()
        .any(|l| l.trim().eq_ignore_ascii_case(&format!("Closes #{number}")))
}
pub fn parse_state(body: &str, trusted_bot: bool) -> Result<Option<State>, String> {
    if !trusted_bot {
        return Ok(None);
    }
    if body.starts_with(MARKER) {
        return serde_json::from_str(body.lines().nth(1).unwrap_or(""))
            .map(Some)
            .map_err(|_| "Malformed bot lease; maintainer intervention required".into());
    }
    if body.starts_with(LEGACY_MARKER) {
        let lease: Option<Lease> = serde_json::from_str(body.lines().nth(1).unwrap_or(""))
            .map_err(|_| "Malformed legacy bot lease")?;
        return Ok(Some(State {
            lease,
            last_command: 0,
        }));
    }
    Ok(None)
}
pub fn trusted_comment(comment: &Value) -> bool {
    comment["user"]["login"] == "github-actions[bot]" && comment["user"]["type"] == "Bot"
}
pub fn lease_expired(lease: &Lease, now: DateTime<Utc>) -> bool {
    lease.pr.is_none()
        && lease
            .expires
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .is_none_or(|expires| expires <= now)
}
pub fn decide(
    request: &Request,
    actor: &str,
    task: &Task,
    state: &State,
    other_active: bool,
    comment_id: u64,
    now: DateTime<Utc>,
) -> Verdict {
    if !task.phase_one {
        return Verdict::NotTask;
    }
    if comment_id <= state.last_command {
        return Verdict::Replayed;
    }
    let owns = state
        .lease
        .as_ref()
        .is_some_and(|l| l.user == actor && l.session.as_deref() == Some(&request.session))
        && task.assignees.iter().any(|a| a == actor);
    if request.command == Command::Claim {
        if owns {
            return Verdict::AlreadyYours;
        }
        if !task.ready || !task.assignees.is_empty() || state.lease.is_some() {
            return Verdict::Unavailable;
        }
        return if other_active {
            Verdict::OtherActive
        } else {
            Verdict::Claim
        };
    }
    if !owns {
        return Verdict::NotOwner;
    }
    let lease = state.lease.as_ref().unwrap();
    if lease.pr.is_some() {
        return Verdict::InReview;
    }
    match request.command {
        Command::Release => Verdict::Release,
        Command::Renew if lease_expired(lease, now) => Verdict::Expired,
        Command::Renew => Verdict::Renew,
        Command::Submitted(n) => Verdict::Submitted(n),
        Command::Claim => unreachable!(),
    }
}
