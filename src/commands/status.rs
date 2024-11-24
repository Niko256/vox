use crate::commands::commit::get_current_commit;
use crate::commands::index::index::Index;
use anyhow::{Context, Result};
use std::collections::hash_set::HashSet;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn status_command() -> Result<()> {
    let (added, modified, deleted, untracked) = get_status(Path::new("."))?;
    let current_commit = get_current_commit()?;

    print_status(&added, &modified, &deleted, &untracked, current_commit);
    Ok(())
}

#[derive(Default)]
struct FileStatus {
    added: Vec<PathBuf>,     // Files added to index
    modified: Vec<PathBuf>,  // Files modified after staging
    deleted: Vec<PathBuf>,   // Files deleted from working directory
    untracked: Vec<PathBuf>, // Files not in index
}

pub fn get_status(
    repo_path: &Path,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>)> {
    let mut index = Index::new();
    let index_path = repo_path.join(".vcs/index");

    if index_path.exists() {
        index.read_from_file(&index_path)?;
    }

    let mut status = FileStatus::default();

    let mut processed_files = HashSet::new();

    for (path, index_entry) in index.get_entries().iter() {
        processed_files.insert(path.clone());
        let full_path = repo_path.join(path);

        if !full_path.exists() {
            status.deleted.push(path.clone());
            continue;
        }

        let metadata = fs::metadata(&full_path)?;
        if metadata.mtime() as u64 != index_entry.mtime
            || metadata.size() as u32 != index_entry.size
        {
            status.modified.push(path.clone());
        } else {
            status.added.push(path.clone());
        }
    }

    for entry in WalkDir::new(repo_path)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| {
            !e.path().starts_with(repo_path.join(".vcs"))
                && !e.path().starts_with(repo_path.join(".git"))
                && !e.path().starts_with(repo_path.join("target"))
        })
    {
        let entry = entry.context("Failed to read directory entry")?;
        if !entry.file_type().is_file() {
            continue;
        }

        let relative_path = entry.path().strip_prefix(repo_path)?.to_path_buf();

        if !processed_files.contains(&relative_path) {
            if index.get_entries().contains_key(&relative_path) {
                let metadata = fs::metadata(entry.path())?;
                let index_entry = index.get_entries().get(&relative_path).unwrap();
                if metadata.mtime() as u64 != index_entry.mtime
                    || metadata.size() as u32 != index_entry.size
                {
                    status.modified.push(relative_path);
                }
            } else {
                status.untracked.push(relative_path);
            }
        }
    }

    Ok((
        status.added,
        status.modified,
        status.deleted,
        status.untracked,
    ))
}

fn print_status(
    added: &[PathBuf],
    modified: &[PathBuf],
    deleted: &[PathBuf],
    untracked: &[PathBuf],
    current_commit: Option<String>,
) {
    let branch_name = match get_current_branch() {
        Ok(name) => name,
        Err(_) => "unknown".to_string(),
    };

    println!("On branch {}", branch_name);
    if let Some(commit) = current_commit {
        println!("Current commit [{}]", &commit[..7]);
    }

    if added.is_empty() && modified.is_empty() && deleted.is_empty() && untracked.is_empty() {
        println!("✓ Working tree clean");
        return;
    }

    if !added.is_empty() {
        println!("Changes to be committed:");
        println!("  (use \"vcs reset HEAD <file>...\" to unstage)\n");
        for path in added {
            println!("\t\x1b[32mnew file:   {}\x1b[0m", path.display());
        }
        println!();
    }

    if !modified.is_empty() || !deleted.is_empty() {
        println!("Changes not added for commit:");
        println!("  (use \"vcs add <file>...\" to update what will be committed)");
        println!("  (use \"vcs restore <file>...\" to discard changes)\n");

        for path in modified {
            println!("\t\x1b[31mmodified:   {}\x1b[0m", path.display());
        }
        for path in deleted {
            println!("\t\x1b[31mdeleted:    {}\x1b[0m", path.display());
        }
        println!();
    }

    if !untracked.is_empty() {
        println!("Untracked files:");
        println!("  (use \"vcs add <file>...\" to include in what will be committed)\n");
        for path in untracked {
            println!("\t\x1b[31m{}\x1b[0m", path.display());
        }
        println!();
    }

    if !modified.is_empty() || !untracked.is_empty() {
        println!("no changes added to commit (use \"vcs add\" and/or \"vcs commit -a\")");
    }
}

fn get_current_branch() -> Result<String> {
    let head_content = fs::read_to_string(".vcs/HEAD").context("Failed to read HEAD file")?;

    // Parse "ref: refs/heads/branch_name" format
    let branch = head_content
        .strip_prefix("ref: refs/heads/")
        .and_then(|s| s.strip_suffix('\n'))
        .context("Invalid HEAD file format")?;

    Ok(branch.to_string())
}
