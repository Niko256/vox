use lazy_static::lazy_static;

lazy_static! {
    pub static ref VOX_DIR: String = ".vox".to_string();
    pub static ref OBJ_DIR: String = format!("{}/objects", *VOX_DIR);
    pub static ref REFS_DIR: String = format!("{}/refs", *VOX_DIR);
    pub static ref HEAD_DIR: String = format!("{}/HEAD", *VOX_DIR);
    pub static ref INDEX_FILE: String = format!("{}/index", *VOX_DIR);
}
