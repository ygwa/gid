use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

mod common;

#[test]
fn test_scenario_fix_wrong_identity() {
    // Scenario: User discovers wrong identity in last commit and wants to quick fix
    let (temp_dir, repo) = common::setup_repo();

    // Simulate wrong commit
    common::create_commit(&repo, "Wrong identity commit");

    // Simulate user configuration environment
    let home_dir = TempDir::new().unwrap();
    let config_dir = home_dir.path().join(".config/gid");
    fs::create_dir_all(&config_dir).unwrap();

    let config_content = r#"
[[identities]]
id = "work"
name = "Correct User"
email = "correct@example.com"
"#;
    fs::write(config_dir.join("config.toml"), config_content).unwrap();

    // User runs fix-commit command (using --yes to skip interaction)
    let mut cmd = Command::cargo_bin("gid").unwrap();
    cmd.env("GID_CONFIG_DIR", config_dir.to_str().unwrap())
        .current_dir(temp_dir.path())
        .arg("fix-commit")
        .arg("--identity")
        .arg("work")
        .arg("--yes")
        .assert()
        .success()
        .stdout(predicate::str::contains("Commit fixed"));

    // Verify commit has been fixed
    let head = repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    let author = commit.author();
    assert_eq!(author.name().unwrap(), "Correct User");
    assert_eq!(author.email().unwrap(), "correct@example.com");
}
