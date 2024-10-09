use lazy_static::lazy_static;

lazy_static! {
    pub static ref VCS_DIR: String = ".vcs".to_string();
    pub static ref OBJ_DIR: String = format!("{}/objects", *VCS_DIR);
    pub static ref REFS_DIR: String = format!("{}/refs", *VCS_DIR);
    pub static ref HEAD_DIR: String = format!("{}/HEAD", *VCS_DIR);
}
