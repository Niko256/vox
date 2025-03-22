use super::blob::Blob;
use super::commit::Commit;
use super::delta::Delta;
use super::tag::Tag;
use super::tree::Tree;
use anyhow::Result;
use std::path::PathBuf;

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
    //Conflict(Conflict),
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
        }
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        match self {
            Object::Blob(blob) => blob.serialize(),
            Object::Commit(commit) => commit.serialize(),
            Object::Tag(tag) => tag.serialize(),
            Object::Tree(tree) => tree.hash().into(),
            Object::Delta(delta) => delta.serialize(),
        }
    }

    fn hash(&self) -> Result<String> {
        match self {
            Object::Blob(blob) => blob.hash(),
            Object::Commit(commit) => commit.hash(),
            Object::Tree(tree) => tree.hash(),
            Object::Tag(tag) => tag.hash(),
            Object::Delta(delta) => delta.hash(),
        }
    }

    fn object_path(&self) -> Result<String> {
        match self {
            Object::Blob(blob) => blob.object_path(),
            Object::Tree(tree) => tree.object_path(),
            Object::Tag(tag) => tag.object_path(),
            Object::Commit(commit) => commit.object_path(),
            Object::Delta(delta) => delta.object_path(),
        }
    }
}
