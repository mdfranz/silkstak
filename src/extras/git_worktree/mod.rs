use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct WorktreeInfo {
    pub branch: String,
    pub worktree_path: PathBuf,
    pub main_repo_path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum MergeOutcome {
    Success,
    Conflicts(Vec<String>),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct MergeState {
    pub info: WorktreeInfo,
    pub original_branch: String,
    pub orig_dir: PathBuf,
    pub stashed: bool,
}

pub fn detect() -> Option<WorktreeInfo> {
    let output = Command::new("git")
        .args(["rev-parse", "--git-common-dir"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let common_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let worktree_path: PathBuf = Path::new(&git_dir).canonicalize().ok()?;

    if common_dir == git_dir {
        return None;
    }

    let main_repo_path: PathBuf = Path::new(&common_dir).parent().map(|p| p.to_path_buf())?;
    let main_repo_path = main_repo_path.canonicalize().ok()?;

    let branch = current_branch().unwrap_or_default();

    Some(WorktreeInfo {
        branch,
        worktree_path: worktree_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or(worktree_path),
        main_repo_path,
    })
}

pub fn current_branch() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch == "HEAD" { None } else { Some(branch) }
}

pub fn default_branch(repo_path: &Path) -> Option<String> {
    for name in &["main", "master"] {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo_path)
            .args(["rev-parse", "--verify", name])
            .output()
            .ok();
        if let Some(out) = output
            && out.status.success()
        {
            return Some(name.to_string());
        }
    }
    None
}

pub fn create(name: &str, base_dir: Option<&Path>) -> Result<(PathBuf, WorktreeInfo), String> {
    let target = match base_dir {
        Some(dir) => dir.join(name),
        None => PathBuf::from(format!("../{}", name)),
    };

    let output = Command::new("git")
        .args(["worktree", "add", "-b", name])
        .arg(&target)
        .output()
        .map_err(|e| format!("failed to run git: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git worktree add failed: {}", stderr.trim()));
    }

    let wt_path = target
        .canonicalize()
        .map_err(|e| format!("failed to resolve worktree path: {}", e))?;

    let main_repo =
        std::env::current_dir().map_err(|e| format!("failed to get current dir: {}", e))?;

    Ok((
        wt_path.clone(),
        WorktreeInfo {
            branch: name.to_string(),
            worktree_path: wt_path,
            main_repo_path: main_repo,
        },
    ))
}

pub fn repo_name(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Phase 1: Change to main repo, stash, fetch, checkout target, pull, merge.
/// On Success or Conflicts, current directory is left in the main repo (on target).
/// On Error, the function cleans up and restores the original directory.
pub fn try_merge(info: &WorktreeInfo, target: &str) -> (MergeState, MergeOutcome) {
    let orig_dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            return (
                MergeState {
                    info: info.clone(),
                    original_branch: String::new(),
                    stashed: false,
                    orig_dir: PathBuf::new(),
                },
                MergeOutcome::Error(e.to_string()),
            );
        }
    };

    if let Err(e) = std::env::set_current_dir(&info.main_repo_path) {
        return (
            MergeState {
                info: info.clone(),
                original_branch: String::new(),
                stashed: false,
                orig_dir,
            },
            MergeOutcome::Error(e.to_string()),
        );
    }

    let original_branch = current_branch().unwrap_or_default();
    let stashed = has_uncommitted_changes() && run_git(["stash", "--include-untracked"]).is_ok();

    if let Err(e) = run_git(["fetch", "--all"]) {
        if stashed {
            let _ = run_git_quiet(["stash", "pop"]);
        }
        let _ = std::env::set_current_dir(&orig_dir);
        return (
            MergeState {
                info: info.clone(),
                original_branch,
                stashed,
                orig_dir,
            },
            MergeOutcome::Error(format!("fetch failed: {}", e)),
        );
    }

    if let Err(e) = run_git(["checkout", target]) {
        if stashed {
            let _ = run_git_quiet(["stash", "pop"]);
        }
        let _ = std::env::set_current_dir(&orig_dir);
        return (
            MergeState {
                info: info.clone(),
                original_branch,
                stashed,
                orig_dir,
            },
            MergeOutcome::Error(format!("checkout failed: {}", e)),
        );
    }

    if let Err(e) = run_git(["pull", "--no-edit"]) {
        let _ = run_git_quiet(["checkout", &original_branch]);
        if stashed {
            let _ = run_git_quiet(["stash", "pop"]);
        }
        let _ = std::env::set_current_dir(&orig_dir);
        return (
            MergeState {
                info: info.clone(),
                original_branch,
                stashed,
                orig_dir,
            },
            MergeOutcome::Error(format!("pull failed: {}", e)),
        );
    }

    let state = MergeState {
        info: info.clone(),
        original_branch,
        stashed,
        orig_dir,
    };

    match run_git(["merge", "--no-edit", &info.branch]) {
        Ok(_) => (state, MergeOutcome::Success),
        Err(_) if has_merge_conflict() => {
            let files = conflicted_files();
            (state, MergeOutcome::Conflicts(files))
        }
        Err(e) => {
            let _ = run_git_quiet(["merge", "--abort"]);
            let _ = run_git_quiet(["checkout", &state.original_branch]);
            if stashed {
                let _ = run_git_quiet(["stash", "pop"]);
            }
            let _ = std::env::set_current_dir(&state.orig_dir);
            (state, MergeOutcome::Error(format!("merge failed: {}", e)))
        }
    }
}

/// Phase 2: After a successful merge (or after conflicts are resolved),
/// push, delete the worktree, delete the branch, restore original dir.
pub fn complete_merge(state: &MergeState) -> Result<(), String> {
    let current = std::env::current_dir().map_err(|e| e.to_string())?;
    let _ = std::env::set_current_dir(&state.info.main_repo_path);

    let result = (|| {
        run_git(["push"])?;
        run_git([
            "worktree",
            "remove",
            &state.info.worktree_path.to_string_lossy(),
        ])?;
        run_git(["branch", "-D", &state.info.branch])?;
        Ok::<(), String>(())
    })();

    if result.is_ok() {
        if state.stashed {
            let _ = run_git_quiet(["stash", "pop"]);
        }
        let _ = std::env::set_current_dir(&state.orig_dir);
    } else {
        let _ = std::env::set_current_dir(&current);
    }

    result
}

/// Cancel an in-progress merge: abort, restore original branch, pop stash, restore dir.
pub fn cancel_merge(state: &MergeState) -> Result<(), String> {
    let _ = std::env::set_current_dir(&state.info.main_repo_path);

    if has_merge_conflict() {
        let _ = run_git_quiet(["merge", "--abort"]);
    }
    if !state.original_branch.is_empty() {
        let _ = run_git_quiet(["checkout", &state.original_branch]);
    }
    if state.stashed {
        let _ = run_git_quiet(["stash", "pop"]);
    }
    let _ = std::env::set_current_dir(&state.orig_dir);

    Ok(())
}

/// Check if there is an active merge conflict in the current directory.
pub fn has_merge_conflict() -> bool {
    let output = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .ok();
    if let Some(out) = output
        && out.status.success()
    {
        let git_dir = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let merge_head = Path::new(&git_dir).join("MERGE_HEAD");
        if merge_head.exists() {
            return true;
        }
    }
    let output = Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=U"])
        .output();
    match output {
        Ok(out) if out.status.success() => !String::from_utf8_lossy(&out.stdout).trim().is_empty(),
        _ => false,
    }
}

/// List files with merge conflicts in the current directory.
pub fn conflicted_files() -> Vec<String> {
    let output = Command::new("git")
        .args(["diff", "--name-only", "--diff-filter=U"])
        .output();
    match output {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout)
            .trim()
            .lines()
            .map(|s| s.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

// --- Private helpers ---

fn run_git<const N: usize>(args: [&str; N]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|e| format!("git failed: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git {} failed: {}", args.join(" "), stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn run_git_quiet<const N: usize>(args: [&str; N]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn has_uncommitted_changes() -> bool {
    let output = Command::new("git").args(["status", "--porcelain"]).output();
    match output {
        Ok(out) if out.status.success() => !String::from_utf8_lossy(&out.stdout).trim().is_empty(),
        _ => false,
    }
}
