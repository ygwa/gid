use anyhow::Result;
use colored::Colorize;
use dialoguer::{Confirm, Input};
use std::path::PathBuf;

use crate::config::{Config, Identity};
use crate::gpg::GpgManager;
use crate::ssh::SshManager;

/// 添加新身份
pub fn execute(
    id: Option<String>,
    name: Option<String>,
    email: Option<String>,
    description: Option<String>,
    ssh_key: Option<PathBuf>,
    gpg_key: Option<String>,
) -> Result<()> {
    let mut config = Config::load()?;

    println!("{}", "添加新的 Git 身份".bold());
    println!();

    // 获取身份 ID
    let id = if let Some(id) = id {
        id
    } else {
        Input::<String>::new()
            .with_prompt("身份 ID (如: work, personal)")
            .interact_text()?
    };

    // 验证 ID 格式
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        anyhow::bail!("身份 ID 只能包含字母、数字、下划线和连字符");
    }

    // 检查 ID 是否已存在
    if config.find_identity(&id).is_some() {
        anyhow::bail!("身份 '{id}' 已存在");
    }

    // 获取姓名
    let name = if let Some(name) = name {
        name
    } else {
        Input::<String>::new().with_prompt("姓名").interact_text()?
    };

    if name.is_empty() {
        anyhow::bail!("姓名不能为空");
    }

    // 获取邮箱
    let email = if let Some(email) = email {
        email
    } else {
        Input::<String>::new().with_prompt("邮箱").interact_text()?
    };

    if !email.contains('@') || !email.contains('.') {
        anyhow::bail!("邮箱格式不正确");
    }

    // 获取描述
    let description = if description.is_some() {
        description
    } else {
        let desc: String = Input::new()
            .with_prompt("描述 (可选，直接回车跳过)")
            .allow_empty(true)
            .interact_text()?;
        if desc.is_empty() {
            None
        } else {
            Some(desc)
        }
    };

    // SSH 密钥配置
    let ssh_key = if ssh_key.is_some() {
        ssh_key
    } else {
        let configure_ssh = Confirm::new()
            .with_prompt("是否配置 SSH 密钥?")
            .default(false)
            .interact()?;

        if configure_ssh {
            configure_ssh_key(&id, &email)?
        } else {
            None
        }
    };

    // GPG 密钥配置
    let gpg_key = if gpg_key.is_some() {
        gpg_key
    } else {
        let configure_gpg = Confirm::new()
            .with_prompt("是否配置 GPG 签名密钥?")
            .default(false)
            .interact()?;

        if configure_gpg {
            configure_gpg_key(&email)?
        } else {
            None
        }
    };

    // 创建身份
    let identity = Identity::new(id.clone(), name.clone(), email.clone())
        .with_description(description)
        .with_ssh_key(ssh_key.clone())
        .with_gpg_key(gpg_key.clone());

    // 验证并保存
    identity.validate().map_err(|e| anyhow::anyhow!(e))?;
    config.add_identity(identity)?;
    config.save()?;

    println!();
    println!(
        "{} 身份已添加: {} {} <{}>",
        "✓".green(),
        format!("[{id}]").green().bold(),
        name,
        email.cyan()
    );

    if ssh_key.is_some() {
        println!("  {} SSH 密钥已配置", "🔑".dimmed());
    }
    if gpg_key.is_some() {
        println!("  {} GPG 签名已配置", "🔏".dimmed());
    }

    // 询问是否立即切换
    println!();
    let switch_now = Confirm::new()
        .with_prompt("是否立即切换到此身份?")
        .default(false)
        .interact()?;

    if switch_now {
        let global = Confirm::new()
            .with_prompt("切换到全局配置?")
            .default(false)
            .interact()?;

        crate::commands::switch::execute(&id, global)?;
    }

    Ok(())
}

/// 配置 SSH 密钥
fn configure_ssh_key(identity_id: &str, email: &str) -> Result<Option<PathBuf>> {
    let ssh = SshManager::new()?;

    println!();
    println!("{}", "SSH 密钥配置:".cyan());
    println!("  1. 使用现有密钥");
    println!("  2. 生成新密钥");
    println!("  3. 跳过");

    let choice: String = Input::new()
        .with_prompt("选择 [1/2/3]")
        .default("3".to_string())
        .interact_text()?;

    match choice.trim() {
        "1" => {
            let key_path: String = Input::new()
                .with_prompt("SSH 私钥路径")
                .default("~/.ssh/id_ed25519".to_string())
                .interact_text()?;

            let path = PathBuf::from(shellexpand::tilde(&key_path).to_string());

            if !ssh.key_exists(&path) {
                anyhow::bail!("密钥文件不存在: {}", path.display());
            }

            Ok(Some(path))
        }
        "2" => {
            println!("{} 生成新的 SSH 密钥...", "→".blue());
            let key_path = ssh.generate_key(identity_id, email)?;
            println!("{} 密钥已生成: {}", "✓".green(), key_path.display());

            // 显示公钥
            if let Ok(pub_key) = ssh.read_public_key(&key_path) {
                println!();
                println!("{}", "公钥内容 (添加到 GitHub/GitLab):".cyan());
                println!("{}", pub_key.trim().dimmed());
            }

            Ok(Some(key_path))
        }
        _ => Ok(None),
    }
}

/// 配置 GPG 密钥
fn configure_gpg_key(email: &str) -> Result<Option<String>> {
    let gpg = GpgManager::new();

    if !gpg.is_available() {
        println!("{} GPG 未安装，跳过配置", "!".yellow());
        return Ok(None);
    }

    println!();
    println!("{}", "GPG 签名配置:".cyan());

    // 查找现有密钥
    if let Ok(Some(key)) = gpg.find_key_by_email(email) {
        println!("找到匹配的 GPG 密钥: {}", key.key_id);
        let use_existing = Confirm::new()
            .with_prompt("使用此密钥?")
            .default(true)
            .interact()?;

        if use_existing {
            return Ok(Some(key.key_id));
        }
    }

    println!("  1. 输入现有密钥 ID");
    println!("  2. 列出所有密钥");
    println!("  3. 跳过");

    let choice: String = Input::new()
        .with_prompt("选择 [1/2/3]")
        .default("3".to_string())
        .interact_text()?;

    match choice.trim() {
        "1" => {
            let key_id: String = Input::new().with_prompt("GPG 密钥 ID").interact_text()?;

            if gpg.verify_key(&key_id)? {
                Ok(Some(key_id))
            } else {
                anyhow::bail!("无效的 GPG 密钥 ID");
            }
        }
        "2" => {
            let keys = gpg.list_keys()?;
            if keys.is_empty() {
                println!("{} 没有找到 GPG 密钥", "!".yellow());
                return Ok(None);
            }

            println!();
            for (i, key) in keys.iter().enumerate() {
                println!("  {}. {} - {}", i + 1, key.key_id, key.uid);
            }
            println!();

            let index: String = Input::new()
                .with_prompt("选择密钥序号 (直接回车跳过)")
                .allow_empty(true)
                .interact_text()?;

            if index.is_empty() {
                return Ok(None);
            }

            let index: usize = index.parse().map_err(|_| anyhow::anyhow!("无效的序号"))?;
            if index == 0 || index > keys.len() {
                anyhow::bail!("序号超出范围");
            }

            Ok(Some(keys[index - 1].key_id.clone()))
        }
        _ => Ok(None),
    }
}

mod shellexpand {
    pub fn tilde(path: &str) -> std::borrow::Cow<str> {
        if let Some(stripped) = path.strip_prefix("~/") {
            if let Some(home) = home::home_dir() {
                return std::borrow::Cow::Owned(format!("{}/{stripped}", home.display()));
            }
        }
        std::borrow::Cow::Borrowed(path)
    }
}
