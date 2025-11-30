use anyhow::Result;
use colored::Colorize;

use crate::config::Config;
use crate::git::GitConfigManager;
use crate::ssh::SshManager;

/// 切换身份
pub fn execute(identity_id: &str, global: bool) -> Result<()> {
    let config = Config::load()?;

    // 查找身份
    let identity = config
        .find_identity(identity_id)
        .ok_or_else(|| anyhow::anyhow!("Identity '{identity_id}' not found"))?;

    let git = GitConfigManager::new()?;

    // 非全局模式需要在 Git 仓库中
    if !global && !git.is_in_repo() {
        anyhow::bail!("Current directory is not a Git repository. Use --global for global switch");
    }

    // 设置 Git 配置
    git.set_user_name(&identity.name, global)?;
    git.set_user_email(&identity.email, global)?;

    // 设置 GPG 签名
    if let Some(ref gpg_key) = identity.gpg_key {
        git.set_signing_key(gpg_key, global)?;
        git.set_gpg_sign(identity.gpg_sign, global)?;
    }

    // 配置 SSH（如果有）
    if let Some(ref ssh_key) = identity.ssh_key {
        let ssh = SshManager::new()?;
        if ssh.key_exists(ssh_key) {
            // 检查 ssh-agent 是否运行
            if ssh.is_agent_running() {
                // 添加密钥到 ssh-agent
                if let Err(e) = ssh.add_to_agent(ssh_key) {
                    eprintln!("{} Failed to add key to ssh-agent: {}", "!".yellow(), e);
                } else {
                    println!("  {} SSH key added to agent", "🔑".dimmed());
                }
            } else {
                println!(
                    "  {} ssh-agent not running, skipping key addition",
                    "!".yellow()
                );
                println!(
                    "    Tip: Run 'ssh-add {}' after starting ssh-agent",
                    ssh_key.display()
                );
            }

            // 为常见的 Git 托管服务配置 SSH
            let hosts = ["github.com", "gitlab.com", "bitbucket.org"];
            for host in hosts {
                if let Err(e) = ssh.configure_for_identity(identity_id, host, ssh_key) {
                    eprintln!("{} Failed to configure SSH ({}): {}", "!".yellow(), host, e);
                }
            }
        } else {
            eprintln!(
                "{} SSH key file does not exist: {}",
                "!".yellow(),
                ssh_key.display()
            );
        }
    }

    // 输出结果
    let scope = if global { "global" } else { "project" };
    println!(
        "{} Switched to {} identity: {} {} <{}>",
        "✓".green(),
        scope,
        format!("[{}]", identity.id).green().bold(),
        identity.name,
        identity.email.cyan()
    );

    if let Some(ref desc) = identity.description {
        println!("  {}", desc.dimmed());
    }

    if identity.ssh_key.is_some() {
        println!("  {} SSH key configured", "🔑".dimmed());
    }

    if identity.gpg_key.is_some() {
        println!("  {} GPG signing enabled", "🔏".dimmed());
    }

    Ok(())
}
