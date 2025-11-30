use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "gid",
    about = "Git Identity Manager - A complete solution for managing multiple Git identities",
    version,
    author,
    after_help = "For more information: https://github.com/your-username/gid"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Switch to a specified identity
    #[command(visible_alias = "sw")]
    Switch {
        /// Identity ID
        identity: String,

        /// Global switch (affects all repositories)
        #[arg(short, long)]
        global: bool,
    },

    /// List all identities
    #[command(visible_alias = "ls")]
    List,

    /// Show current identity
    #[command(visible_alias = "c")]
    Current,

    /// Add a new identity
    Add {
        /// Identity ID (e.g., work, personal)
        #[arg(short, long)]
        id: Option<String>,

        /// Name
        #[arg(short, long)]
        name: Option<String>,

        /// Email
        #[arg(short, long)]
        email: Option<String>,

        /// Description
        #[arg(short, long)]
        description: Option<String>,

        /// SSH private key path
        #[arg(long)]
        ssh_key: Option<PathBuf>,

        /// GPG key ID
        #[arg(long)]
        gpg_key: Option<String>,
    },

    /// Remove an identity
    #[command(visible_alias = "rm")]
    Remove {
        /// Identity ID to remove
        identity: String,
    },

    /// Edit configuration file
    Edit,

    /// Export configuration
    Export {
        /// Export file path
        #[arg(default_value = "gid-config.toml")]
        file: PathBuf,
    },

    /// Import configuration
    Import {
        /// File path to import
        file: PathBuf,
    },

    /// Manage rules
    Rule {
        #[command(subcommand)]
        action: RuleAction,
    },

    /// Check identity configuration in current directory
    Doctor {
        /// Automatically fix issues
        #[arg(short, long)]
        fix: bool,
    },

    /// Automatically switch identity based on rules
    Auto,

    /// Manage Git hooks
    Hook {
        #[command(subcommand)]
        action: HookAction,
    },

    /// Audit identity information in commit history
    Audit {
        /// Path to audit (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Attempt to fix issues
        #[arg(short, long)]
        fix: bool,
    },

    /// Fix identity information in commits
    FixCommit {
        /// Commit to fix (defaults to HEAD)
        #[arg(default_value = "HEAD")]
        commit: String,

        /// Use specified identity (defaults to current identity)
        #[arg(short, long)]
        identity: Option<String>,

        /// Batch fix commit range (e.g., HEAD~3..HEAD)
        #[arg(short, long)]
        range: Option<String>,

        /// Skip confirmation prompts
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Generate shell completion scripts
    Completions {
        /// Shell type
        #[arg(value_enum)]
        shell: ShellType,
    },
}

#[derive(Subcommand, Clone)]
pub enum RuleAction {
    /// Add a rule
    Add {
        /// Rule type
        #[arg(short, long, value_enum)]
        rule_type: RuleType,

        /// Match pattern
        #[arg(short, long)]
        pattern: String,

        /// Identity to use when matched
        #[arg(short, long)]
        identity: String,

        /// Rule priority (lower number = higher priority)
        #[arg(long, default_value = "100")]
        priority: u32,
    },

    /// List all rules
    List,

    /// Remove a rule
    Remove {
        /// Rule index
        index: usize,
    },

    /// Test rule matching
    Test {
        /// Test path
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Test remote URL
        #[arg(short, long)]
        remote: Option<String>,
    },
}

#[derive(Subcommand, Clone)]
pub enum HookAction {
    /// Install Git hook
    Install {
        /// Global installation (using core.hooksPath)
        #[arg(short, long)]
        global: bool,
    },

    /// Uninstall Git hook
    Uninstall {
        /// Global uninstallation
        #[arg(short, long)]
        global: bool,
    },

    /// Show hook status
    Status,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RuleType {
    /// Path matching rule
    Path,
    /// Remote URL matching rule
    Remote,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

impl From<ShellType> for clap_complete::Shell {
    fn from(shell: ShellType) -> Self {
        match shell {
            ShellType::Bash => clap_complete::Shell::Bash,
            ShellType::Zsh => clap_complete::Shell::Zsh,
            ShellType::Fish => clap_complete::Shell::Fish,
            ShellType::PowerShell => clap_complete::Shell::PowerShell,
        }
    }
}
