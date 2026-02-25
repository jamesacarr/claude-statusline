use std::path::Path;

/// Return the current git branch name for the given directory string.
/// Returns `None` if `dir` is empty, not inside a git repo, or HEAD is detached.
pub fn get_branch(dir: &str) -> Option<String> {
    if dir.is_empty() {
        return None;
    }
    get_branch_from(Path::new(dir))
}

/// Return the current git branch name by walking up from `dir` to find a `.git` entry.
/// Returns `None` if no git repository is found, HEAD is detached, or any filesystem
/// error occurs. Never panics.
pub(crate) fn get_branch_from(dir: &Path) -> Option<String> {
    let mut current = dir;
    loop {
        let git_entry = current.join(".git");
        if git_entry.exists() {
            let head_path = if git_entry.is_dir() {
                git_entry.join("HEAD")
            } else {
                // .git is a file (worktree or submodule) containing "gitdir: <path>"
                let content = std::fs::read_to_string(&git_entry).ok()?;
                let gitdir = content.strip_prefix("gitdir: ").map(|s| s.trim())?;
                let gitdir_path = Path::new(gitdir);
                if gitdir_path.is_absolute() {
                    gitdir_path.join("HEAD")
                } else {
                    // Relative path is relative to the directory containing the .git file
                    let base = current;
                    base.join(gitdir_path).join("HEAD")
                }
            };

            let head_content = std::fs::read_to_string(&head_path).ok()?;
            return head_content
                .trim()
                .strip_prefix("ref: refs/heads/")
                .map(|branch| branch.to_string());
        }

        // Walk up to parent
        match current.parent() {
            Some(parent) if parent != current => current = parent,
            _ => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn returns_branch_name_from_standard_git_head() {
        let tmp = tempfile::tempdir().unwrap();
        let git_dir = tmp.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        assert_eq!(get_branch_from(tmp.path()), Some("main".to_string()));
    }

    #[test]
    fn returns_branch_name_with_slashes() {
        let tmp = tempfile::tempdir().unwrap();
        let git_dir = tmp.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/feature/my-branch\n").unwrap();
        assert_eq!(
            get_branch_from(tmp.path()),
            Some("feature/my-branch".to_string())
        );
    }

    #[test]
    fn returns_none_for_detached_head() {
        let tmp = tempfile::tempdir().unwrap();
        let git_dir = tmp.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        fs::write(
            git_dir.join("HEAD"),
            "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2\n",
        )
        .unwrap();
        assert_eq!(get_branch_from(tmp.path()), None);
    }

    #[test]
    fn returns_none_when_no_git_directory() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(get_branch_from(tmp.path()), None);
    }

    #[test]
    fn returns_none_for_empty_head_file() {
        let tmp = tempfile::tempdir().unwrap();
        let git_dir = tmp.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        fs::write(git_dir.join("HEAD"), "").unwrap();
        assert_eq!(get_branch_from(tmp.path()), None);
    }

    #[test]
    fn returns_none_for_empty_dir_string() {
        assert_eq!(get_branch(""), None);
    }

    #[test]
    fn finds_git_dir_in_parent_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let git_dir = tmp.path().join(".git");
        fs::create_dir(&git_dir).unwrap();
        fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        let sub = tmp.path().join("sub");
        fs::create_dir(&sub).unwrap();
        assert_eq!(get_branch_from(&sub), Some("main".to_string()));
    }

    #[test]
    fn follows_gitdir_file_for_worktrees() {
        let tmp = tempfile::tempdir().unwrap();
        // Create the actual git directory with a HEAD file
        let actual_git = tmp.path().join("actual_git");
        fs::create_dir(&actual_git).unwrap();
        fs::write(actual_git.join("HEAD"), "ref: refs/heads/wt-branch\n").unwrap();
        // Create the worktree root with a .git file pointing to the actual_git dir
        let worktree = tmp.path().join("worktree");
        fs::create_dir(&worktree).unwrap();
        let gitdir_content = format!("gitdir: {}\n", actual_git.to_string_lossy());
        fs::write(worktree.join(".git"), &gitdir_content).unwrap();
        assert_eq!(get_branch_from(&worktree), Some("wt-branch".to_string()));
    }
}
