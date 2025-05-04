use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
    pub static ref VOX_DIR: PathBuf = PathBuf::from(".vox");
    pub static ref OBJ_DIR: PathBuf = VOX_DIR.join("objects");
    pub static ref REFS_DIR: PathBuf = VOX_DIR.join("refs");
    pub static ref HEAD_DIR: PathBuf = VOX_DIR.join("HEAD");
    pub static ref INDEX_FILE: PathBuf = VOX_DIR.join("index");
}

pub const OBJ_TYPE_BLOB: &str = "blob";
pub const OBJ_TYPE_COMMIT: &str = "commit";
pub const OBJ_TYPE_TAG: &str = "tag";
pub const OBJ_TYPE_TREE: &str = "tree";
pub const OBJ_TYPE_CHANGE: &str = "change";
pub const UNKNOWN_TYPE: &str = "unknown type";

pub const PERM_FILE: &str = "100644";
pub const PERM_DIR: &str = "40000";

pub mod errors {}
