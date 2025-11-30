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

    println!("{}", "Add new Git identity".bold());
    println!();

    // 获取身份 ID
    let id = if let Some(id) = id {
        id
    } else {
        Input::<String>::new()
            .with_prompt("Identity ID (e.g., work, personal)")
            .interact_text()?
    };

    // 验证 ID 格式
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        anyhow::bail!("Identity ID can only contain letters, numbers, underscores, and hyphens");
    }

    // 检查 ID 是否已存在
    if config.find_identity(&id).is_some() {
        anyhow::bail!("Identity '{id}' already exists");
    }

    // 获取姓名
    let name = if let Some(name) = name {
        name
    } else {
        Input::<String>::new().with_prompt("Name").interact_text()?
    };

    if name.is_empty() {
        anyhow::bail!("Name cannot be empty");
    }

    // 获取邮箱
    let email = if let Some(email) = email {
        email
    } else {
        Input::<String>::new()
            .with_prompt("Email")
            .interact_text()?
    };

    if !email.contains('@') || !email.contains('.') {
        anyhow::bail!("Invalid email format");
    }

    // 获取描述
    let description = if description.is_some() {
        description
    } else {
        let desc: String = Input::new()
            .with_prompt("Description (optional, press Enter to skip)")
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
            .with_prompt("Configure SSH key?")
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
            .with_prompt("Configure GPG signing key?")
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
        "{} Identity added: {} {} <{}>",
        "✓".green(),
        format!("[{id}]").green().bold(),
        name,
        email.cyan()
    );

    if ssh_key.is_some() {
        println!("  {} SSH key configured", "🔑".dimmed());
    }
    if gpg_key.is_some() {
        println!("  {} GPG signing configured", "🔏".dimmed());
    }

    // 询问是否立即切换
    println!();
    let switch_now = Confirm::new()
        .with_prompt("Switch to this identity now?")
        .default(false)
        .interact()?;

    if switch_now {
        let global = Confirm::new()
            .with_prompt("Switch to global configuration?")
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
    println!("{}", "SSH Key Configuration:".cyan());
    println!("  1. Use existing key");
    println!("  2. Generate new key");
    println!("  3. Skip");

    let choice: String = Input::new()
        .with_prompt("Select [1/2/3]")
        .default("3".to_string())
        .interact_text()?;

    match choice.trim() {
        "1" => {
            let key_path: String = Input::new()
                .with_prompt("SSH private key path")
                .default("~/.ssh/id_ed25519".to_string())
                .interact_text()?;

            let path = PathBuf::from(shellexpand::tilde(&key_path).to_string());

            if !ssh.key_exists(&path) {
                anyhow::bail!("Key file does not exist: {}", path.display());
            }

            Ok(Some(path))
        }
        "2" => {
            println!("{} Generating new SSH key...", "→".blue());
            let key_path = ssh.generate_key(identity_id, email)?;
            println!("{} Key generated: {}", "✓".green(), key_path.display());

            // 显示公钥
            if let Ok(pub_key) = ssh.read_public_key(&key_path) {
                println!();
                println!("{}", "Public key content (add to GitHub/GitLab):".cyan());
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
        println!("{} GPG not installed, skipping configuration", "!".yellow());
        return Ok(None);
    }

    println!();
    println!("{}", "GPG Signing Configuration:".cyan());

    // 查找现有密钥
    if let Ok(Some(key)) = gpg.find_key_by_email(email) {
        println!("Found matching GPG key: {}", key.key_id);
        let use_existing = Confirm::new()
            .with_prompt("Use this key?")
            .default(true)
            .interact()?;

        if use_existing {
            return Ok(Some(key.key_id));
        }
    }

    println!("  1. Enter existing key ID");
    println!("  2. List all keys");
    println!("  3. Skip");

    let choice: String = Input::new()
        .with_prompt("Select [1/2/3]")
        .default("3".to_string())
        .interact_text()?;

    match choice.trim() {
        "1" => {
            let key_id: String = Input::new().with_prompt("GPG Key ID").interact_text()?;

            if gpg.verify_key(&key_id)? {
                Ok(Some(key_id))
            } else {
                anyhow::bail!("Invalid GPG Key ID");
            }
        }
        "2" => {
            let keys = gpg.list_keys()?;
            if keys.is_empty() {
                println!("{} No GPG keys found", "!".yellow());
                return Ok(None);
            }

            println!();
            for (i, key) in keys.iter().enumerate() {
                println!("  {}. {} - {}", i + 1, key.key_id, key.uid);
            }
            println!();

            let index: String = Input::new()
                .with_prompt("Select key index (press Enter to skip)")
                .allow_empty(true)
                .interact_text()?;

            if index.is_empty() {
                return Ok(None);
            }

            let index: usize = index
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid index"))?;
            if index == 0 || index > keys.len() {
                anyhow::bail!("Index out of range");
            }

            Ok(Some(keys[index - 1].key_id.clone()))
        }
        _ => Ok(None),
    }
}

mod shellexpand {
    pub fn tilde(path: &str) -> std::borrow::Cow<'_, str> {
        if let Some(stripped) = path.strip_prefix("~/") {
            if let Some(home) = home::home_dir() {
                return std::borrow::Cow::Owned(format!("{}/{stripped}", home.display()));
            }
        }
        std::borrow::Cow::Borrowed(path)
    }
}
