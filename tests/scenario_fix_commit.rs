use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

mod common;

#[test]
fn test_scenario_fix_wrong_identity() {
    // 场景：用户发现最近一次提交使用了错误的身份，想要快速修复
    let (temp_dir, repo) = common::setup_repo();
    
    // 模拟错误的提交
    common::create_commit(&repo, "Wrong identity commit");
    
    // 模拟用户配置环境
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
    
    // 用户执行 fix-commit 命令（使用 --yes 跳过交互）
    let mut cmd = Command::cargo_bin("gid").unwrap();
    cmd.env("GID_CONFIG_DIR", config_dir.to_str().unwrap())
       .current_dir(temp_dir.path())
       .arg("fix-commit")
       .arg("--identity")
       .arg("work")
       .arg("--yes")
       .assert()
       .success()
       .stdout(predicate::str::contains("提交已修复"));
       
    // 验证提交已被修复
    let head = repo.head().unwrap();
    let commit = head.peel_to_commit().unwrap();
    let author = commit.author();
    assert_eq!(author.name().unwrap(), "Correct User");
    assert_eq!(author.email().unwrap(), "correct@example.com");
}
