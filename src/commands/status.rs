use crate::commands::index::index::Index;
use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn status_command() -> Result<()> {
    let mut index = Index::new();
    let index_path = Path::new(".vcs/index");

    if index_path.exists() {
        index
            .read_from_file(index_path)
            .context("Failed to read index")?;
    }

    let mut untracked = Vec::new();
    let mut modified = Vec::new();
    let added: Vec<PathBuf> = Vec::new();
    let mut deleted = Vec::new();

    for entry in WalkDir::new(".")
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| {
            !e.path().starts_with("./.vcs")
                && !e.path().starts_with("./.git")
                && !e.path().starts_with("./target")
        })
    {
        let entry = entry.context("Failed to read directory entry")?;
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry
            .path()
            .strip_prefix("./")
            .unwrap_or(entry.path())
            .to_path_buf();

        if let Some(index_entry) = index.get_entry(&path) {
            if let Ok(metadata) = fs::metadata(&path) {
                if metadata.mtime() as u64 != index_entry.mtime
                    || metadata.size() as u32 != index_entry.size
                {
                    modified.push(path);
                }
            } else {
                deleted.push(path);
            }
        } else {
            untracked.push(path);
        }
    }

    for path in index.get_entries().keys() {
        if !path.exists() {
            deleted.push(path.to_path_buf());
        }
    }

    if !added.is_empty() || !modified.is_empty() || !deleted.is_empty() {
        println!("Changes to be committed:");
        for path in &added {
            println!("\tnew file:   {}", path.display());
        }
        for path in &modified {
            println!("\tmodified:   {}", path.display());
        }
        for path in &deleted {
            println!("\tdeleted:    {}", path.display());
        }
        println!();
    }

    if !modified.is_empty() {
        println!("Changes not staged for commit:");
        for path in &modified {
            println!("\tmodified:   {}", path.display());
        }
        println!();
    }

    if !untracked.is_empty() {
        println!("Untracked files:");
        println!("  (use \"vcs add <file>...\" to include in what will be committed)");
        println!();
        for path in &untracked {
            println!("\t{}", path.display());
        }
        println!();
    }

    if added.is_empty() && modified.is_empty() && deleted.is_empty() && untracked.is_empty() {
        println!("nothing to commit, working tree clean");
    }

    Ok(())
}

pub fn get_status(
    repo_path: &Path,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>)> {
    let mut untracked = Vec::new();
    let mut modified = Vec::new();
    let added: Vec<PathBuf> = Vec::new();
    let mut deleted = Vec::new();

    let mut index = Index::new();
    let index_path = repo_path.join(".vcs/index");

    if index_path.exists() {
        index.read_from_file(&index_path)?;
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

        let path = entry.path().strip_prefix(repo_path)?.to_path_buf();

        if let Some(index_entry) = index.get_entry(&path) {
            if let Ok(metadata) = fs::metadata(entry.path()) {
                if metadata.mtime() as u64 != index_entry.mtime
                    || metadata.size() as u32 != index_entry.size
                {
                    modified.push(path);
                }
            }
        } else {
            untracked.push(path);
        }
    }

    for path in index.get_entries().keys() {
        if !repo_path.join(path).exists() {
            deleted.push(path.to_path_buf());
        }
    }

    Ok((added, modified, deleted, untracked))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::index::index::{Index, IndexEntry};
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_repo() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        fs::create_dir(repo_path.join(".vcs")).unwrap();
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

    #[test]
    fn test_modified_file() {
        let (_, repo_path) = setup_test_repo();
        let mut index = Index::new();

        let file_path = create_test_file(&repo_path, "test.txt", "initial content");
        let relative_path = PathBuf::from("test.txt");
        let entry = IndexEntry::new(&file_path).unwrap();
        index.add_entry(entry);
        index.write_to_file(&repo_path.join(".vcs/index")).unwrap();

        std::thread::sleep(std::time::Duration::from_secs(1));
        fs::write(&file_path, "modified content").unwrap();

        let (added, modified, deleted, untracked) = get_status(&repo_path).unwrap();

        assert!(added.is_empty());
        assert_eq!(modified.len(), 1);
        assert_eq!(modified[0], PathBuf::from("test.txt"));
        assert!(deleted.is_empty());
        assert!(untracked.is_empty());
    }

    #[test]
    fn test_clean_working_directory() {
        let (_, repo_path) = setup_test_repo();
        let mut index = Index::new();

        let file_path = create_test_file(&repo_path, "test.txt", "content");
        let relative_path = PathBuf::from("test.txt");
        let entry = IndexEntry::new(&file_path).unwrap();
        index.add_entry(entry);
        index.write_to_file(&repo_path.join(".vcs/index")).unwrap();

        let (added, modified, deleted, untracked) = get_status(&repo_path).unwrap();

        assert!(added.is_empty());
        assert!(modified.is_empty());
        assert!(deleted.is_empty());
        assert!(untracked.is_empty());
    }
}
