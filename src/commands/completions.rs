use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

use crate::cli::{Cli, ShellType};

/// 生成 Shell 补全脚本
pub fn execute(shell: ShellType) -> Result<()> {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    let shell: Shell = shell.into();

    generate(shell, &mut cmd, name, &mut io::stdout());

    Ok(())
}
