use anyhow::Result;
use colored::Colorize;
use dialoguer::Confirm;

use crate::config::Config;

/// 删除身份
pub fn execute(identity_id: &str) -> Result<()> {
    let mut config = Config::load()?;
    
    // 查找身份
    let identity = config.find_identity(identity_id)
        .ok_or_else(|| anyhow::anyhow!("找不到身份 '{}'", identity_id))?
        .clone();
    
    // 确认删除
    println!(
        "将要删除身份: {} {} <{}>",
        format!("[{}]", identity.id).yellow(),
        identity.name,
        identity.email
    );
    
    let confirm = Confirm::new()
        .with_prompt("确定要删除吗?")
        .default(false)
        .interact()?;
    
    if !confirm {
        println!("操作已取消");
        return Ok(());
    }
    
    // 删除身份
    config.remove_identity(identity_id)?;
    config.save()?;
    
    println!(
        "{} 身份 '{}' 已删除",
        "✓".green(),
        identity_id
    );
    
    Ok(())
}

