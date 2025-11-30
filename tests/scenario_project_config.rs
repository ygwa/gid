use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

mod common;

#[test]
fn test_scenario_simple_config() {
    // Scenario: User creates a simple .gid file in new project to specify identity
    let (temp_dir, _repo) = common::setup_repo();

    // User creates .gid file
    fs::write(temp_dir.path().join(".gid"), "work\n").unwrap();

    // User runs doctor to check configuration
    let mut cmd = Command::cargo_bin("gid").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Expected Identity: [work]"));
}

#[test]
fn test_scenario_toml_config() {
    // Scenario: User uses TOML format for complex project rules
    let (temp_dir, _repo) = common::setup_repo();

    // User creates .gid file with rules
    let config = r#"
identity = "personal"
[[rules]]
type = "path"
pattern = "src/**"
identity = "personal"
priority = 100
"#;
    fs::write(temp_dir.path().join(".gid"), config).unwrap();

    // User runs doctor to check configuration
    let mut cmd = Command::cargo_bin("gid").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Expected Identity: [personal]"));
}
