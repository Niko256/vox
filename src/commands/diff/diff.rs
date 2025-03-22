use crate::commands::commit::get_current_commit;
use crate::objects::object::Loadable;
use crate::objects::{commit::Commit, delta::Delta, tree::Tree};
use crate::utils::OBJ_DIR;
use anyhow::{Context, Result};
use std::collections::HashSet;

pub fn diff_commits(from: &str, to: &str) -> Result<Delta> {
    let from_tree = get_commit_tree(from)?;
    let to_tree = get_commit_tree(to)?;

    let mut delta = Delta {
        from: Some(from.to_string()),
        to: Some(to.to_string()),
        ..Default::default()
    };

    compare_trees(&from_tree, &to_tree, &mut delta)?;
    Ok(delta)
}

pub fn get_commit_tree(commit_hash: &str) -> Result<Tree> {
    let commit = Commit::load(commit_hash, &OBJ_DIR)?;
    Tree::load(&commit.tree, &OBJ_DIR)?;
}

pub fn compare_trees(first: &Tree, second: &Tree, delta: &mut Delta) -> Result<()> {
    
    for path in first.entries.keys()

}
