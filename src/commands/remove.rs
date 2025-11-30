use anyhow::Result;
use colored::Colorize;
use dialoguer::Confirm;

use crate::config::Config;

/// 删除身份
pub fn execute(identity_id: &str) -> Result<()> {
    let mut config = Config::load()?;

    // 查找身份
    let identity = config
        .find_identity(identity_id)
        .ok_or_else(|| anyhow::anyhow!("Identity '{identity_id}' not found"))?
        .clone();

    // 确认删除
    println!(
        "About to remove identity: {} {} <{}>",
        format!("[{}]", identity.id).yellow(),
        identity.name,
        identity.email
    );

    let confirm = Confirm::new()
        .with_prompt("Are you sure you want to remove?")
        .default(false)
        .interact()?;

    if !confirm {
        println!("Operation cancelled");
        return Ok(());
    }

    // 删除身份
    config.remove_identity(identity_id)?;
    config.save()?;

    println!("{} Identity '{}' removed", "✓".green(), identity_id);

    Ok(())
}
