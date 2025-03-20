use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum DeltaType {
    Added,
    Deleted,
    Modified,
    Renamed,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct FileDelta {
    pub delta_type: DeltaType,
    pub old_path: Option<PathBuf>,
    pub new_path: Option<Pathbuf>,
    pub old_hash: Option<String>,
    pub new_hash: Option<String>,
    pub diff: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Delta {
    pub files: HashMap<Pathbuf, FileDelta>,
    pub from: Option<String>,
    pub to: Option<String>,
}
