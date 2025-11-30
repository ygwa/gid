use git2::Repository;
use tempfile::TempDir;

// Helper to setup a temp git repo
#[allow(dead_code)]
pub fn setup_repo() -> (TempDir, Repository) {
    let temp_dir = TempDir::new().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();
    
    // Configure user for commit
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test User").unwrap();
    config.set_str("user.email", "test@example.com").unwrap();
    
    (temp_dir, repo)
}

// Helper to create a commit
#[allow(dead_code)]
pub fn create_commit(repo: &Repository, message: &str) {
    let mut index = repo.index().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    
    let sig = repo.signature().unwrap();
    
    let parent_commit = if let Ok(head) = repo.head() {
        Some(head.peel_to_commit().unwrap())
    } else {
        None
    };
    
    let parents = if let Some(ref c) = parent_commit {
        vec![c]
    } else {
        vec![]
    };
    
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        message,
        &tree,
        &parents,
    ).unwrap();
}
