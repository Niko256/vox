use crate::utils::OBJ_DIR;

use super::blob::Blob;
use super::commit::Commit;
use super::delta::Delta;
use super::tag::Tag;
use super::tree::Tree;
use anyhow::{anyhow, Result};
use sha1::{Digest, Sha1};
use std::path::PathBuf;
use std::str::FromStr;

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
    Delta(Delta),
    Unknown(String),
}

pub trait Storable {
    fn save(&self, objects_dir: &PathBuf) -> Result<String>;
}

pub trait Loadable {
    fn load(hash: &str, objects_dir: &PathBuf) -> Result<Self>
    where
        Self: Sized;
}

pub trait Diffable {
    fn diff(&self, other: &Self) -> Result<Delta>;
}

pub trait Mergeble {
    fn merge(&self, other: &Self) -> Result<Self>
    where
        Self: Sized;
}

impl VoxObject for Object {
    fn object_type(&self) -> &str {
        match self {
            Object::Blob(_) => "blob",
            Object::Commit(_) => "commit",
            Object::Tag(_) => "tag",
            Object::Tree(_) => "tree",
            Object::Delta(_) => "delta",
            Object::Unknown(_) => "unknown type",
        }
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        match self {
            Object::Blob(blob) => blob.serialize(),
            Object::Commit(commit) => commit.serialize(),
            Object::Tag(tag) => tag.serialize(),
            Object::Tree(tree) => tree.serialize(),
            Object::Delta(delta) => delta.serialize(),
            Object::Unknown(data) => Ok(data.as_bytes().to_vec()),
        }
    }

    fn hash(&self) -> Result<String> {
        match self {
            Object::Blob(blob) => blob.hash(),
            Object::Commit(commit) => commit.hash(),
            Object::Tag(tag) => tag.hash(),
            Object::Tree(tree) => tree.hash(),
            Object::Delta(delta) => delta.hash(),
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
            Object::Delta(delta) => delta.object_path(),
            Object::Unknown(data) => {
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
        if parts.len() < 2 {
            return Err(anyhow!("Invalid object format: expected 'type data'"));
        }

        let object_type = parts[0];
        let object_data = parts[1];

        match object_type {
            "blob" => {
                let blob = Blob::load(object_data, &*OBJ_DIR)?;
                Ok(Object::Blob(blob))
            }
            "commit" => {
                let commit = Commit::load(object_data, &*OBJ_DIR)?;
                Ok(Object::Commit(commit))
            }
            "tree" => {
                let tree = Tree::load(object_data, &*OBJ_DIR)?;
                Ok(Object::Tree(tree))
            }
            "tag" => {
                let tag = Tag::load(object_data, &*OBJ_DIR)?;
                Ok(Object::Tag(tag))
            }
            "delta" => {
                let delta = Delta::load(object_data, &*OBJ_DIR)?;
                Ok(Object::Delta(delta))
            }
            _ => Ok(Object::Unknown(s.to_string())),
        }
    }
}
