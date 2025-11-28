pub mod switch;
pub mod list;
pub mod current;
pub mod add;
pub mod remove;
pub mod edit;
pub mod export;
pub mod import;
pub mod rule;
pub mod doctor;
pub mod auto;
pub mod hook;
pub mod audit;
pub mod completions;

use colored::Colorize;

/// 输出成功消息
pub fn success(msg: &str) {
    println!("{} {}", "✓".green(), msg);
}

/// 输出错误消息
pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red(), msg);
}

/// 输出警告消息
pub fn warn(msg: &str) {
    println!("{} {}", "!".yellow(), msg);
}

/// 输出信息消息
pub fn info(msg: &str) {
    println!("{} {}", "→".blue(), msg);
}

