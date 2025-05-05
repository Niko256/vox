use crate::storage::objects::blob::Blob;
use crate::storage::objects::change::ChangeSet;
use crate::storage::objects::commit::Commit;
use crate::storage::objects::tag::Tag;
use crate::storage::objects::tree::Tree;
use anyhow::{anyhow, Result};
use bytes::Bytes;
use sha1::{Digest, Sha1};
use std::str::FromStr;
use std::{collections::HashMap, path::Path};

use crate::storage::utils::{
    OBJ_DIR, OBJ_TYPE_BLOB, OBJ_TYPE_CHANGE, OBJ_TYPE_COMMIT, OBJ_TYPE_TAG, OBJ_TYPE_TREE,
    UNKNOWN_TYPE,
};

pub mod blob;
pub mod branch;
pub mod change;
pub mod commit;
pub mod delta;
pub mod pack;
pub mod tag;
pub mod tree;

pub trait VoxObject {
    fn object_type(&self) -> &str;
    fn serialize(&self) -> Result<Vec<u8>>;
    fn hash(&self) -> Result<String>;
    fn object_path(&self) -> Result<String>;
}

pub(crate) enum Object {
    Blob(Blob),
    Commit(Commit),
    Tree(Tree),
    Tag(Tag),
    ChangeSet(ChangeSet),
    Unknown(String),
}

pub trait ObjectStore {
    async fn get_object(&self, hash: &[u8; 20]) -> Result<Option<Box<dyn VoxObject>>>;
    async fn put_object(&self, obj: Box<dyn VoxObject>) -> Result<()>;
}

pub trait Storable {
    fn save(&self, objects_dir: &Path) -> Result<String>;
}

pub trait Loadable {
    fn load(hash: &str, objects_dir: &Path) -> Result<Self>
    where
        Self: Sized;
}

pub trait Diffable {
    fn diff(&self, other: &Self) -> Result<ChangeSet>;
}

pub trait Mergeble {
    fn merge(&self, other: &Self) -> Result<Self>
    where
        Self: Sized;
}

impl VoxObject for Object {
    fn object_type(&self) -> &str {
        match self {
            Object::Blob(_) => OBJ_TYPE_BLOB,
            Object::Commit(_) => OBJ_TYPE_COMMIT,
            Object::Tag(_) => OBJ_TYPE_TAG,
            Object::Tree(_) => OBJ_TYPE_TREE,
            Object::ChangeSet(_) => OBJ_TYPE_CHANGE,
            Object::Unknown(_) => UNKNOWN_TYPE,
        }
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        match self {
            Object::Blob(blob) => blob.serialize(),
            Object::Commit(commit) => commit.serialize(),
            Object::Tag(tag) => tag.serialize(),
            Object::Tree(tree) => tree.serialize(),
            Object::ChangeSet(changes) => changes.serialize(),
            Object::Unknown(_data) => Ok(_data.as_bytes().to_vec()),
        }
    }

    fn hash(&self) -> Result<String> {
        match self {
            Object::Blob(blob) => blob.hash(),
            Object::Commit(commit) => commit.hash(),
            Object::Tag(tag) => tag.hash(),
            Object::Tree(tree) => tree.hash(),
            Object::ChangeSet(changes) => changes.hash(),
            Object::Unknown(data) => {
                let mut hasher = Sha1::new();
                hasher.update(data.as_bytes());
                Ok(format!("{:x}", hasher.finalize()))
            }
        }
    }

    fn object_path(&self) -> Result<String> {
        match self {
            Object::Blob(blob) => blob.object_path(),
            Object::Commit(commit) => commit.object_path(),
            Object::Tag(tag) => tag.object_path(),
            Object::Tree(tree) => tree.object_path(),
            Object::ChangeSet(changes) => changes.object_path(),
            Object::Unknown(_data) => {
                let hash = self.hash()?;
                Ok(format!(
                    "{}/{}/{}",
                    OBJ_DIR.display(),
                    &hash[..2],
                    &hash[2..]
                ))
            }
        }
    }
}

impl FromStr for Object {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.splitn(2, ' ').collect();

        if parts.len() != 2 {
            return Err(anyhow!("Invalid object format: expected 'type data'"));
        }

        let object_type = parts[0];
        let object_data = parts[1];

        match object_type {
            OBJ_TYPE_BLOB => {
                let blob = Blob::load(object_data, &OBJ_DIR)?;
                Ok(Object::Blob(blob))
            }
            OBJ_TYPE_COMMIT => {
                let commit = Commit::load(object_data, &OBJ_DIR)?;
                Ok(Object::Commit(commit))
            }
            OBJ_TYPE_TREE => {
                let tree = Tree::load(object_data, &OBJ_DIR)?;
                Ok(Object::Tree(tree))
            }
            OBJ_TYPE_TAG => {
                let tag = Tag::load(object_data, &OBJ_DIR)?;
                Ok(Object::Tag(tag))
            }
            OBJ_TYPE_CHANGE => {
                let changes = ChangeSet::load(object_data, &OBJ_DIR)?;
                Ok(Object::ChangeSet(changes))
            }
            _ => Ok(Object::Unknown(s.to_string())),
        }
    }
}
