use crate::commands::index::index::Index;
use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn status_command() -> Result<()> {
    let (added, modified, deleted, untracked) = get_status(Path::new("."))?;
    print_status(&added, &modified, &deleted, &untracked);
    Ok(())
}

fn print_status(
    added: &[PathBuf],
    modified: &[PathBuf],
    deleted: &[PathBuf],
    untracked: &[PathBuf],
) {
    println!("On branch main\n");

    if added.is_empty() && modified.is_empty() && deleted.is_empty() && untracked.is_empty() {
        println!("✓ Working tree clean");
        return;
    }

    if !added.is_empty() || !modified.is_empty() || !deleted.is_empty() {
        println!("Changes to be committed:");
        println!("  (use \"vcs reset HEAD <file>...\" to unstage)\n");
        for path in added {
            println!("\t\x1b[32mnew file:   {}\x1b[0m", path.display());
        }
        for path in modified {
            println!("\t\x1b[32mmodified:   {}\x1b[0m", path.display());
        }
        for path in deleted {
            println!("\t\x1b[32mdeleted:    {}\x1b[0m", path.display());
        }
        println!();
    }

    if !modified.is_empty() {
        println!("Changes not staged for commit:");
        println!("  (use \"vcs add <file>...\" to update what will be committed)");
        println!("  (use \"vcs restore <file>...\" to discard changes)\n");
        for path in modified {
            println!("\t\x1b[31mmodified:   {}\x1b[0m", path.display());
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
        println!("no changes added to commit (use \"vcs add\" and/or \"vcs commit -a\")");
    }
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
    scan_working_directory(repo_path, &mut index, &mut status)?;
    scan_index(repo_path, &index, &mut status)?;

    Ok((
        status.added,
        status.modified,
        status.deleted,
        status.untracked,
    ))
}

#[derive(Default)]
struct FileStatus {
    added: Vec<PathBuf>,
    modified: Vec<PathBuf>,
    deleted: Vec<PathBuf>,
    untracked: Vec<PathBuf>,
}

fn scan_working_directory(
    repo_path: &Path,
    index: &mut Index,
    status: &mut FileStatus,
) -> Result<()> {
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

        let path = entry.path().strip_prefix(repo_path)?.to_path_buf();

        if let Some(index_entry) = index.get_entry(&path) {
            if let Ok(metadata) = fs::metadata(entry.path()) {
                if metadata.mtime() as u64 != index_entry.mtime
                    || metadata.size() as u32 != index_entry.size
                {
                    status.modified.push(path);
                } else {
                    status.added.push(path);
                }
            }
        } else {
            status.untracked.push(path);
        }
    }
    Ok(())
}

fn scan_index(repo_path: &Path, index: &Index, status: &mut FileStatus) -> Result<()> {
    for path in index.get_entries().keys() {
        if !repo_path.join(path).exists() {
            status.deleted.push(path.to_path_buf());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::add::add_command;
    use crate::commands::index::index::{Index, IndexEntry};
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_repo() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        fs::create_dir_all(repo_path.join(".vcs/objects")).unwrap();
        fs::create_dir_all(repo_path.join(".vcs/refs")).unwrap();
        fs::write(repo_path.join(".vcs/HEAD"), "ref: refs/heads/main\n").unwrap();
        (temp_dir, repo_path)
    }

    fn create_test_file(repo_path: &Path, name: &str, content: &str) -> PathBuf {
        let file_path = repo_path.join(name);
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file_path
    }

    #[test]
    fn test_deleted_file() {
        let (temp_dir, repo_path) = setup_test_repo();
        let mut index = Index::new();

        let file_path = create_test_file(&repo_path, "test.txt", "content");
        let entry = IndexEntry::new(&file_path).unwrap();
        index.add_entry(entry);
        index.write_to_file(&repo_path.join(".vcs/index")).unwrap();

        fs::remove_file(&file_path).unwrap();

        let (added, modified, deleted, untracked) = get_status(&repo_path).unwrap();

        assert!(added.is_empty());
        assert!(modified.is_empty());
        assert_eq!(deleted.len(), 1);
        assert_eq!(deleted[0], file_path);
        assert!(untracked.is_empty());
    }

    #[test]
    fn test_untracked_file() {
        let (_, repo_path) = setup_test_repo();
        let index = Index::new();
        index.write_to_file(&repo_path.join(".vcs/index")).unwrap();

        let file_path = create_test_file(&repo_path, "untracked.txt", "content");
        let relative_path = PathBuf::from("untracked.txt");

        let (added, modified, deleted, untracked) = get_status(&repo_path).unwrap();

        assert!(added.is_empty());
        assert!(modified.is_empty());
        assert!(deleted.is_empty());
        assert_eq!(untracked.len(), 1);
        assert_eq!(untracked[0], relative_path);
    }
}
