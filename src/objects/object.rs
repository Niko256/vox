use super::blob::Blob;
use super::commit::Commit;
use super::tag::Tag;
use super::tree::Tree;
use anyhow::Result;
use std::path::PathBuf;

pub trait VcsObject {
    fn object_type(&self) -> &str;
    fn serialize(&self) -> Vec<u8>;
    fn hash(&self) -> String;
    fn object_path(&self) -> String;
}

enum Object {
    Blob(Blob),
    Commit(Commit),
    Tree(Tree),
    Tag(Tag),
    //Delta(Delta),
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

impl VcsObject for Object {
    fn object_type(&self) -> &str {
        match self {
            Object::Blob(_) => "blob",
            Object::Commit(_) => "commit",
            Object::Tag(_) => "tag",
            Object::Tree(_) => "tree",
        }
    }

    fn serialize(&self) -> Vec<u8> {
        match self {
            Object::Blob(blob) => blob.serialize(),
            Object::Commit(commit) => commit.serialize(),
            Object::Tag(tag) => tag.serialize(),
            Object::Tree(tree) => tree.hash().into(),
        }
    }

    fn hash(&self) -> String {
        match self {
            Object::Blob(blob) => blob.hash(),
            Object::Commit(commit) => commit.hash(),
            Object::Tree(tree) => tree.hash(),
            Object::Tag(tag) => tag.hash(),
        }
    }

    fn object_path(&self) -> String {
        match self {
            Object::Blob(blob) => blob.object_path(),
            Object::Tree(tree) => tree.object_path(),
            Object::Tag(tag) => tag.object_path(),
            Object::Commit(commit) => commit.object_path(),
        }
    }
}
