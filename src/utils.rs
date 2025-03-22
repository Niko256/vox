use lazy_static::lazy_static;
use std::path::PathBuf;

lazy_static! {
    pub static ref VOX_DIR: PathBuf = PathBuf::from(".vox");
    pub static ref OBJ_DIR: PathBuf = VOX_DIR.join("objects");
    pub static ref REFS_DIR: PathBuf = VOX_DIR.join("refs");
    pub static ref HEAD_DIR: PathBuf = VOX_DIR.join("HEAD");
    pub static ref INDEX_FILE: PathBuf = VOX_DIR.join("index");
}
