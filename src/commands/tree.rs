use anyhow::Result;
use crate::objects::tree::create_tree_from_index;
use crate::commands::index::load_index;

pub fn write_tree_command() -> Result<()> {
    let index = load_index()?;
    let index_vec: Vec<(String, String)> = index.into_iter().collect();

    let tree_hash = create_tree_from_index(&index_vec)?;
    println!("{}", tree_hash);
    Ok(())
}
