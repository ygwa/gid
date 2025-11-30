use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

mod common;

#[test]
fn test_scenario_simple_config() {
    // 场景：用户在新项目中创建一个简单的 .gid 文件来指定身份
    let (temp_dir, _repo) = common::setup_repo();
    
    // 用户创建 .gid 文件
    fs::write(temp_dir.path().join(".gid"), "work\n").unwrap();
    
    // 用户运行 doctor 检查配置
    let mut cmd = Command::cargo_bin("gid").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("期望身份: [work]"));
}

#[test]
fn test_scenario_toml_config() {
    // 场景：用户使用 TOML 格式配置复杂的项目规则
    let (temp_dir, _repo) = common::setup_repo();
    
    // 用户创建包含规则的 .gid 文件
    let config = r#"
identity = "personal"
[[rules]]
type = "path"
pattern = "src/**"
identity = "personal"
priority = 100
"#;
    fs::write(temp_dir.path().join(".gid"), config).unwrap();
    
    // 用户运行 doctor 检查配置
    let mut cmd = Command::cargo_bin("gid").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("期望身份: [personal]"));
}
