use crate::storage::objects::{Blob, Commit, Object, Tag, Tree, VoxObject};
use crate::storage::utils::{OBJ_TYPE_BLOB, OBJ_TYPE_COMMIT, OBJ_TYPE_TAG, OBJ_TYPE_TREE};
use anyhow::{anyhow, bail, Context, Result};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use flate2::bufread::ZlibDecoder;
use flate2::{write::ZlibEncoder, Compression};
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::path::Path;

#[derive(Debug)]
pub struct Packfile {
    pub objects: Vec<PackObject>,
    pub index: HashMap<String, ObjectLocation>,
}

#[derive(Debug)]
pub struct ObjectLocation {
    pub offset: u64,
    pub size: u32,
    pub type_code: u8,
}

#[derive(Debug)]
pub enum PackObject {
    Base(Vec<u8>, ObjectType),
    Delta { base_hash: String, data: Vec<u8> },
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectType {
    Commit = 1,
    Tree = 2,
    Blob = 3,
    Tag = 4,
    DeltaRef = 7,
}

impl Packfile {
    pub fn new() -> Self {
        Packfile {
            objects: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub fn add_object(&mut self, obj: &dyn VoxObject) -> Result<()> {
        let obj_type = match obj.object_type() {
            OBJ_TYPE_COMMIT => ObjectType::Commit,
            OBJ_TYPE_TREE => ObjectType::Tree,
            OBJ_TYPE_BLOB => ObjectType::Blob,
            OBJ_TYPE_TAG => ObjectType::Tag,
            _ => bail!("Unsopportet object type"),
        };

        let data = obj.serialize()?;
        self.objects.push(PackObject::Base(data, obj_type));
        Ok(())
    }

    pub fn serialize(&mut self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer.write_all(b"VOXPACK")?;
        buffer.write_u32::<BigEndian>(self.objects.len() as u32)?;

        let mut offset = 12;

        for obj in &self.objects {
            let (type_code, content) = match obj {
                PackObject::Base(data, obj_type) => (*obj_type as u8, data.clone()),
                PackObject::Delta { base_hash, data } => (ObjectType::DeltaRef as u8, data.clone()),
            };

            let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
            encoder.write_all(&content)?;
            let compressed = encoder.finish()?;

            let size = compressed.len() as u32;
            let mut header = Vec::new();
            header.write_u8((type_code << 4) | 0x80)?;
            header.write_u24::<BigEndian>(size)?;

            let mut hasher = Sha1::new();
            hasher.update(&content);
            let hash = format!("{:x}", hasher.finalize());

            buffer.write_all(&header)?;
            buffer.write_all(&compressed)?;

            self.index.insert(
                hash,
                ObjectLocation {
                    offset: offset as u64,
                    size,
                    type_code,
                },
            );

            offset += header.len() + compressed.len();
        }

        Ok(buffer)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let mut magic = [0u8; 7];
        cursor.read_exact(&mut magic)?;

        if &magic != b"VOXPACK" {
            bail!("Invalid pack format");
        }

        let object_count = cursor.read_u32::<BigEndian>()?;
        let mut pack = Packfile::new();
        let mut offset = 12;

        for _ in 0..object_count {
            let first_byte = cursor.read_u8()?;
            let type_code = (first_byte >> 4) & 0x07;
            let compressed_size = cursor.read_u24::<BigEndian>()?;

            let mut compressed = vec![0u8; compressed_size as usize];
            cursor.read_exact(&mut compressed)?;

            let mut decoder = ZlibDecoder::new(&compressed[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)?;

            let obj_type = match type_code {
                1 => ObjectType::Commit,
                2 => ObjectType::Tree,
                3 => ObjectType::Blob,
                4 => ObjectType::Tag,
                7 => ObjectType::DeltaRef,
                _ => bail!("Invalid object type"),
            };

            let (obj, hash) = match obj_type {
                ObjectType::DeltaRef => {
                    let mut base_hash = [0u8; 20];
                    base_hash.copy_from_slice(&decompressed[..20]);
                    let data = decompressed[20..].to_vec();
                    (
                        PackObject::Delta {
                            base_hash: hex::encode(base_hash),
                            data: data.clone(),
                        },
                        hex::encode(Sha1::digest(&data)),
                    )
                }
                _ => {
                    let hash = hex::encode(Sha1::digest(&decompressed));
                    (PackObject::Base(decompressed, obj_type), hash)
                }
            };

            pack.objects.push(obj);
            pack.index.insert(
                hash,
                ObjectLocation {
                    offset,
                    size: compressed_size,
                    type_code,
                },
            );

            offset += 4 + compressed_size as u64;
        }

        Ok(pack)
    }

    pub fn apply_deltas(&self, base_objects: &HashMap<String, Vec<u8>>) -> Result<Vec<Object>> {
        let mut results = Vec::new();
        for obj in &self.objects {
            match obj {
                PackObject::Base(data, obj_type) => {
                    let obj = Self::parse_object(*obj_type, data)?;
                    results.push(obj);
                }
                PackObject::Delta { base_hash, data } => {
                    let base_data = base_objects
                        .get(base_hash)
                        .ok_or_else(|| anyhow!("Missing base object {}", base_hash))?;

                    let reconstructed = apply_delta(base_data, data)?;
                    let obj_type = Self::detect_type(&reconstructed)?;
                    let obj = Self::parse_object(obj_type, &reconstructed)?;
                    results.push(obj);
                }
            }
        }
        Ok(results)
    }

    fn parse_object(obj_type: ObjectType, data: &[u8]) -> Result<Object> {
        match obj_type {
            ObjectType::Commit => {
                let commit = Commit::parse(&String::from_utf8(data.to_vec()))?;
                Ok(Object::Commit(commit))
            }
            ObjectType::Tree => {
                let tree = Tree::parse(data)?;
                Ok(Object::Tree(tree))
            }
            ObjectType::Blob => Ok(Object::Blob(Blob {
                data: data.to_vec(),
            })),
            ObjectType::Tag => {
                let tag = Tag::parse(&String::from_utf8(data.to_vec())?)?;
                Ok(Object::Tag(tag))
            }
            _ => bail!("Unsupported object type"),
        }
    }

    pub fn detect_type(data: &[u8]) -> Result<ObjectType> {
        if data.starts_with(b"commit") {
            Ok(ObjectType::Commit)
        } else if data.starts_with(b"tree") {
            Ok(ObjectType::Tree)
        } else if data.starts_with(b"tag") {
            Ok(ObjectType::Tag)
        } else {
            Ok(ObjectType::Blob)
        }
    }
}

fn apply_delta(base: &[u8], delta: &[u8]) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    let mut cursor = Cursor::new(delta);

    let base_size = read_size_encoding(&mut cursor)?;
    let result_size = read_size_encoding(&mut cursor)?;

    if base_size != base.len() {
        bail!("Base size mismatch");
    }

    while cursor.position() < delta.len() as u64 {
        let cmd = cursor.read_u8()?;
        if cmd & 0x80 != 0 {
            let mut offset = 0u64;
            for i in 0..4 {
                if cmd & (0x80 >> i) != 0 {
                    offset |= (cursor.read_u8()? as u64) << (i * 8);
                }
            }
            let mut length = 0u64;
            for i in 4..7 {
                if cmd & (0x80 >> i) != 0 {
                    length |= (cursor.read_u8()? as u64) << ((i - 4) * 8);
                }
            }
            if length == 0 {
                length = 0x10000;
            }

            let start = offset as usize;
            let end = start + length as usize;
            result.extend_from_slice(&base[start..end]);
        } else {
            let length = cmd as usize;
            let mut data = vec![0u8; length];
            cursor.read_exact(&mut data)?;
            result.extend(data);
        }
    }

    if result.len() != result_size {
        bail!("Result size mismatch");
    }

    Ok(result)
}

fn read_size_encoding<R: Read>(reader: &mut R) -> Result<usize> {
    let mut size = 0;
    let mut shift = 0;
    loop {
        let byte = reader.read_u8()?;
        size |= ((byte & 0x7F) as usize) << shift;
        shift += 7;
        if (byte & 0x80) == 0 {
            break;
        }
    }
    Ok(size)
}
