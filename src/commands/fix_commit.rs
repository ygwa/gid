use anyhow::Result;
use colored::Colorize;
use git2::{Repository, Signature};

use crate::config::Config;
use crate::git::GitConfigManager;

/// Fix commit identity information
pub fn execute(
    commit_ref: &str,
    identity_id: Option<String>,
    range: Option<String>,
    yes: bool,
) -> Result<()> {
    let config = Config::load()?;
    let git = GitConfigManager::new()?;

    if !git.is_in_repo() {
        anyhow::bail!("Current directory is not a Git repository");
    }

    // Check for uncommitted changes
    let repo = Repository::discover(".")?;
    if has_uncommitted_changes(&repo)? {
        anyhow::bail!(
            "{}",
            "Uncommitted changes detected. Please commit or stash changes before fixing history."
                .red()
        );
    }

    // Get identity to use
    let identity_id = if let Some(id) = identity_id {
        id
    } else {
        // Use current identity
        let current_email = git
            .get_effective_user_email()
            .ok_or_else(|| anyhow::anyhow!("Could not get current email"))?;

        config
            .identities
            .iter()
            .find(|i| i.email == current_email)
            .map(|i| i.id.clone())
            .ok_or_else(|| anyhow::anyhow!("Current identity not in configuration list"))?
    };

    let identity = config
        .find_identity(&identity_id)
        .ok_or_else(|| anyhow::anyhow!("Identity '{identity_id}' not found"))?;

    // Handle batch fix
    if let Some(range_str) = range {
        fix_commit_range(&repo, &range_str, identity, yes)?;
    } else {
        fix_single_commit(&repo, commit_ref, identity, yes)?;
    }

    Ok(())
}

/// Fix single commit
fn fix_single_commit(
    repo: &Repository,
    commit_ref: &str,
    identity: &crate::config::Identity,
    yes: bool,
) -> Result<()> {
    // Only support fixing HEAD
    if commit_ref != "HEAD" {
        println!(
            "{} Fixing non-HEAD commits requires --range argument",
            "!".yellow()
        );
        println!("  Example: gid fix-commit --range HEAD~3..HEAD");
        anyhow::bail!("Fixing non-HEAD commits is not supported");
    }

    let obj = repo.revparse_single(commit_ref)?;
    let commit = obj.peel_to_commit()?;

    let current_author = commit.author();
    let current_name = current_author.name().unwrap_or("");
    let current_email = current_author.email().unwrap_or("");

    println!("{}", "Fixing commit identity...".bold());
    println!();
    println!("Commit: {}", commit.id().to_string()[..7].dimmed());
    println!(
        "Message: {}",
        commit.message().unwrap_or("").lines().next().unwrap_or("")
    );
    println!();
    println!(
        "Current Identity: {} <{}>",
        current_name,
        current_email.cyan()
    );
    println!(
        "New Identity:   {} <{}>",
        identity.name,
        identity.email.cyan()
    );
    println!();

    // Confirm
    if !yes {
        let confirm = dialoguer::Confirm::new()
            .with_prompt("Confirm fix?")
            .default(false)
            .interact()?;

        if !confirm {
            println!("Operation cancelled");
            return Ok(());
        }
    }

    // Modify commit
    let new_author = Signature::now(&identity.name, &identity.email)?;
    let tree = commit.tree()?;

    let new_commit_oid = if commit.parent_count() > 0 {
        let parent = commit.parent(0)?;
        repo.commit(
            None,
            &new_author,
            &commit.committer(),
            commit.message().unwrap_or(""),
            &tree,
            &[&parent],
        )?
    } else {
        // Initial commit
        repo.commit(
            None,
            &new_author,
            &commit.committer(),
            commit.message().unwrap_or(""),
            &tree,
            &[],
        )?
    };

    // Update HEAD
    let head_ref = repo.head()?;
    if head_ref.is_branch() {
        let branch_name = head_ref.name().unwrap();
        repo.reference(branch_name, new_commit_oid, true, "gid fix-commit")?;
    } else {
        // Detached HEAD
        repo.set_head_detached(new_commit_oid)?;
    }

    let new_commit = new_commit_oid;

    println!();
    println!("{} Commit fixed", "✓".green());
    println!("  New commit: {}", new_commit.to_string()[..7].green());
    println!();
    println!(
        "{} Commit hash changed, use {} to force push if already pushed",
        "⚠".yellow(),
        "git push --force".cyan()
    );

    Ok(())
}

/// Batch fix commit range
fn fix_commit_range(
    repo: &Repository,
    range: &str,
    identity: &crate::config::Identity,
    yes: bool,
) -> Result<()> {
    println!("{}", "Batch fixing commits...".bold());
    println!();
    println!("Range: {}", range.cyan());
    println!(
        "New Identity: {} <{}>",
        identity.name,
        identity.email.cyan()
    );
    println!();

    // Parse range
    let revspec = repo.revparse(range)?;

    if revspec.mode().contains(git2::RevparseMode::SINGLE) {
        anyhow::bail!("Please use range format, e.g., HEAD~3..HEAD");
    }

    let from = revspec
        .from()
        .ok_or_else(|| anyhow::anyhow!("Invalid range"))?;
    let to = revspec
        .to()
        .ok_or_else(|| anyhow::anyhow!("Invalid range"))?;

    // Get commits in range
    let mut revwalk = repo.revwalk()?;
    revwalk.push(to.id())?;
    revwalk.hide(from.id())?;

    let commit_count = revwalk.count();

    if commit_count == 0 {
        println!("{} No commits in range", "!".yellow());
        return Ok(());
    }

    println!("Will fix {commit_count} commits");
    println!();

    // Warning
    println!(
        "{} {} This will modify commit history, all subsequent commit hashes will change",
        "⚠".yellow().bold(),
        "WARNING:".yellow().bold()
    );
    println!("  If pushed, you will need to use git push --force");
    println!(
        "  Recommend backing up current branch: git branch backup-$(git branch --show-current)"
    );
    println!();

    // Confirm
    if !yes {
        let confirm = dialoguer::Confirm::new()
            .with_prompt("Confirm continue?")
            .default(false)
            .interact()?;

        if !confirm {
            println!("Operation cancelled");
            return Ok(());
        }
    }

    println!();
    println!("{} Batch fix not supported yet", "!".yellow());
    println!("  Recommend using git rebase or git filter-branch");
    println!("  Or use specialized tools like git-filter-repo");
    println!();
    println!("Example command:");
    println!(
        "  {}",
        format!(
            "git filter-branch --env-filter 'export GIT_AUTHOR_NAME=\"{}\" GIT_AUTHOR_EMAIL=\"{}\"' {}",
            identity.name, identity.email, range
        )
        .dimmed()
    );

    Ok(())
}

/// Check for uncommitted changes
fn has_uncommitted_changes(repo: &Repository) -> Result<bool> {
    let statuses = repo.statuses(None)?;
    Ok(!statuses.is_empty())
}
