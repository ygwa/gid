use assert_cmd::Command;
use predicates::prelude::*;

mod common;

#[test]
fn test_scenario_audit_repo() {
    // 场景：用户想要审计一个仓库的提交历史，查看是否有身份问题
    let (temp_dir, repo) = common::setup_repo();
    
    // 创建一些提交
    common::create_commit(&repo, "Initial commit");
    common::create_commit(&repo, "Feature A");
    
    // 用户运行 audit 命令
    let mut cmd = Command::cargo_bin("gid").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("audit")
        .assert()
        .success()
        .stdout(predicate::str::contains("总提交数: 2"))
        .stdout(predicate::str::contains("身份使用统计"));
}
