use std::process::Command;
#[test]
fn coordinate_refuses_contributor_machine() {
    let out = Command::new(env!("CARGO_BIN_EXE_ri"))
        .arg("coordinate")
        .env_remove("GITHUB_ACTIONS")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("not on contributor machines"));
}
#[test]
fn session_is_a_unique_uuid_without_background_service() {
    let mut ids = Vec::new();
    for _ in 0..2 {
        let out = Command::new(env!("CARGO_BIN_EXE_ri"))
            .arg("session")
            .output()
            .unwrap();
        assert!(out.status.success());
        let s = String::from_utf8(out.stdout).unwrap();
        assert!(rewrite_internet::coordination::valid_session(s.trim()));
        ids.push(s);
    }
    assert_ne!(ids[0], ids[1]);
}
#[test]
fn invalid_claim_is_rejected_before_github_access() {
    let out = Command::new(env!("CARGO_BIN_EXE_ri"))
        .args(["claim", "42", "invalid-session"])
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("canonical UUID"));
}
