use assert_cmd::Command;
use predicates::prelude::*;

mod common;

#[test]
fn test_scenario_audit_repo() {
    // Scenario: User wants to audit commit history for identity issues
    let (temp_dir, repo) = common::setup_repo();

    // Create some commits
    common::create_commit(&repo, "Initial commit");
    common::create_commit(&repo, "Feature A");

    // User runs audit command
    let mut cmd = Command::cargo_bin("gid").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("audit")
        .assert()
        .success()
        .stdout(predicate::str::contains("Total Commits: 2"))
        .stdout(predicate::str::contains("Identity Usage Statistics"));
}
