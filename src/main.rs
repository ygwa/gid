mod cli;
mod commands;
mod config;
mod git;
mod rules;
mod ssh;
mod gpg;
mod audit;

use anyhow::Result;
use cli::{Cli, Commands};
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Switch { identity, global } => {
            commands::switch::execute(&identity, global)?;
        }
        Commands::List => {
            commands::list::execute()?;
        }
        Commands::Current => {
            commands::current::execute()?;
        }
        Commands::Add { 
            id, 
            name, 
            email, 
            description,
            ssh_key,
            gpg_key,
        } => {
            commands::add::execute(id, name, email, description, ssh_key, gpg_key)?;
        }
        Commands::Remove { identity } => {
            commands::remove::execute(&identity)?;
        }
        Commands::Edit => {
            commands::edit::execute()?;
        }
        Commands::Export { file } => {
            commands::export::execute(file)?;
        }
        Commands::Import { file } => {
            commands::import::execute(&file)?;
        }
        Commands::Rule { action } => {
            commands::rule::execute(action)?;
        }
        Commands::Doctor { fix } => {
            commands::doctor::execute(fix)?;
        }
        Commands::Auto => {
            commands::auto::execute()?;
        }
        Commands::Hook { action } => {
            commands::hook::execute(action)?;
        }
        Commands::Audit { path, fix } => {
            commands::audit::execute(path, fix)?;
        }
        Commands::Completions { shell } => {
            commands::completions::execute(shell)?;
        }
    }
    
    Ok(())
}

