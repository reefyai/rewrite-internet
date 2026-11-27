use chrono::{TimeZone, Utc};
use rewrite_internet::coordination::*;
use serde_json::json;
const A: &str = "11111111-1111-4111-8111-111111111111";
const B: &str = "22222222-2222-4222-8222-222222222222";
fn now() -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap()
}
fn task() -> Task {
    Task {
        phase_one: true,
        ready: true,
        assignees: vec![],
    }
}
fn request(command: Command, session: &str) -> Request {
    Request {
        command,
        session: session.into(),
    }
}
fn held() -> State {
    State {
        lease: Some(Lease {
            user: "synthetic-owner".into(),
            session: Some(A.into()),
            expires: Some("2030-01-01T01:00:00Z".into()),
            pr: None,
        }),
        last_command: 10,
    }
}
fn assigned() -> Task {
    Task {
        assignees: vec!["synthetic-owner".into()],
        ready: false,
        ..task()
    }
}
#[test]
fn ready_task_is_claimable() {
    assert_eq!(
        decide(
            &request(Command::Claim, A),
            "synthetic-owner",
            &task(),
            &State::default(),
            false,
            11,
            now()
        ),
        Verdict::Claim
    );
}
#[test]
fn second_account_cannot_overwrite_claim() {
    assert_eq!(
        decide(
            &request(Command::Claim, B),
            "synthetic-other",
            &assigned(),
            &held(),
            false,
            11,
            now()
        ),
        Verdict::Unavailable
    );
}
#[test]
fn second_session_same_account_is_not_told_already_yours() {
    assert_eq!(
        decide(
            &request(Command::Claim, B),
            "synthetic-owner",
            &assigned(),
            &held(),
            false,
            11,
            now()
        ),
        Verdict::Unavailable
    );
}
#[test]
fn same_session_receives_already_yours() {
    assert_eq!(
        decide(
            &request(Command::Claim, A),
            "synthetic-owner",
            &assigned(),
            &held(),
            false,
            11,
            now()
        ),
        Verdict::AlreadyYours
    );
}
#[test]
fn one_task_per_session_or_account_cap() {
    assert_eq!(
        decide(
            &request(Command::Claim, A),
            "synthetic-owner",
            &task(),
            &State::default(),
            true,
            11,
            now()
        ),
        Verdict::OtherActive
    );
}
#[test]
fn another_session_cannot_renew_or_release() {
    for command in [Command::Renew, Command::Release] {
        assert_eq!(
            decide(
                &request(command, B),
                "synthetic-owner",
                &assigned(),
                &held(),
                false,
                11,
                now()
            ),
            Verdict::NotOwner
        );
    }
}
#[test]
fn live_lease_renews_but_expired_does_not() {
    let r = request(Command::Renew, A);
    assert_eq!(
        decide(
            &r,
            "synthetic-owner",
            &assigned(),
            &held(),
            false,
            11,
            now()
        ),
        Verdict::Renew
    );
    assert_eq!(
        decide(
            &r,
            "synthetic-owner",
            &assigned(),
            &held(),
            false,
            11,
            now() + chrono::Duration::hours(2)
        ),
        Verdict::Expired
    );
}
#[test]
fn review_has_no_expiry_or_release() {
    let mut s = held();
    s.lease.as_mut().unwrap().pr = Some(123);
    s.lease.as_mut().unwrap().expires = None;
    assert!(!lease_expired(s.lease.as_ref().unwrap(), now()));
    assert_eq!(
        decide(
            &request(Command::Release, A),
            "synthetic-owner",
            &assigned(),
            &s,
            false,
            11,
            now()
        ),
        Verdict::InReview
    );
}
#[test]
fn duplicate_and_old_commands_do_not_mutate_state() {
    for id in [9, 10] {
        assert_eq!(
            decide(
                &request(Command::Renew, A),
                "synthetic-owner",
                &assigned(),
                &held(),
                false,
                id,
                now()
            ),
            Verdict::Replayed
        );
    }
}
#[test]
fn released_state_does_not_replay_old_claim() {
    let s = State {
        lease: None,
        last_command: 20,
    };
    assert_eq!(
        decide(
            &request(Command::Claim, A),
            "synthetic-owner",
            &task(),
            &s,
            false,
            11,
            now()
        ),
        Verdict::Replayed
    );
}
#[test]
fn stale_assignment_blocks_new_claim_even_without_lease() {
    assert_eq!(
        decide(
            &request(Command::Claim, A),
            "synthetic-owner",
            &assigned(),
            &State::default(),
            false,
            11,
            now()
        ),
        Verdict::Unavailable
    );
}
#[test]
fn exact_closing_reference_required() {
    assert!(closes_issue("Task F001\nCloses #42\n", 42));
    assert!(!closes_issue("Closes #420", 42));
    assert!(!closes_issue("Not actually Closes #42", 42));
}
#[test]
fn command_requires_session_and_exact_pr_repository() {
    assert!(parse_command(&format!("/claim {A}")).is_some());
    assert!(parse_command("/claim").is_none());
    assert!(parse_command(&format!("/claim {A}\nrun a command")).is_none());
    assert!(
        parse_command(&format!(
            "/submitted {A} https://github.com/reefyai/rewrite-internet/pull/42"
        ))
        .is_some()
    );
    assert!(parse_pr_url("https://example.invalid/pull/42").is_none());
    assert!(parse_pr_url("https://github.com/reefyai/rewrite-internet/pull/42?x=1").is_none());
}
#[test]
fn untrusted_lease_marker_is_ignored() {
    assert_eq!(
        parse_state(&format!("{MARKER}\n{{bad}}"), false).unwrap(),
        None
    );
    assert!(parse_state(&format!("{MARKER}\n{{bad}}"), true).is_err());
    assert!(!trusted_comment(
        &json!({"user":{"login":"synthetic-owner","type":"User"}})
    ));
}
#[test]
fn legacy_review_lease_survives_migration() {
    let body =
        format!("{LEGACY_MARKER}\n{{\"user\":\"synthetic-owner\",\"expires\":null,\"pr\":123}}");
    let state = parse_state(&body, true).unwrap().unwrap();
    assert_eq!(state.lease.as_ref().unwrap().session, None);
    assert!(!lease_expired(state.lease.as_ref().unwrap(), now()));
    assert_eq!(state.lease.unwrap().pr, Some(123));
}
#[test]
fn v2_state_round_trips() {
    let s = held();
    let b = format!(
        "{MARKER}\n{}\n\nHuman text",
        serde_json::to_string(&s).unwrap()
    );
    assert_eq!(parse_state(&b, true).unwrap(), Some(s));
}
