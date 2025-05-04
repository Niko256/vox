use bytes::{Buf, Bytes};
use std::fmt;

/// Represents a Git packfile containing object data and metadata.
///
/// A packfile is Git's efficient way to store and transfer multiple objects
/// It consists of a header, a series of packed objects, and a trailing checksum
///
#[derive(Debug)]
pub struct Packfile {
    /// The packfile header containing format information
    pub header: PackfileHeader,
    /// The collection of packed objects in this packfile
    pub objects: Vec<PackObject>,
}

/// Header information for a Git packfile.
///
/// Packfiles always begin with this fixed-size header structure
///
#[derive(Debug)]
pub struct PackfileHeader {
    /// Should always be "PACK" in ASCII (0x50, 0x41, 0x43, 0x4B)
    pub signature: [u8; 4],
    /// Version number (typically 2 or 3)
    pub version: u32,
    /// Total number of objects contained in this packfile
    pub num_objects: u32,
}

/// Represents a single packed object within a packfile
///
/// Git uses delta compression to efficiently store similar objects
///
#[derive(Debug)]
pub enum PackObject {
    /// A complete base object (not delta-compressed)
    Base {
        /// The type of Git object
        type_: GitObjectType,
        /// The raw object data
        data: Bytes,
    },
    /// An offset delta object (relative to another object in this pack)
    OfsDelta {
        /// Byte offset to the base object within this packfile
        base_offset: u64,
        /// Delta instructions to reconstruct the object
        delta: Bytes,
    },
    /// A reference delta object (points to object by hash)
    RefDelta {
        /// SHA-1 hash of the base object
        base_hash: [u8; 20],
        /// Delta instructions to reconstruct the object
        delta: Bytes,
    },
}

/// The type of a Git object
#[derive(Debug, Clone, Copy)]
pub enum GitObjectType {
    Commit = 1,
    Tree = 2,
    Blob = 3,
    Tag = 4,
}

impl Packfile {
    /// Parses a packfile from raw bytes.
    ///
    /// # Arguments
    /// * `data` - The raw bytes of a packfile
    ///
    /// # Returns
    /// A parsed `Packfile` structure containing all objects
    ///
    /// # Errors
    /// Returns an error if the packfile is malformed or uses unsupported features
    pub fn parse(mut data: Bytes) -> Result<Self> {
        // Step 1: Parse the fixed-size header
        let header = PackfileHeader {
            signature: Self::parse_signature(&mut data)?,
            version: Self::parse_version(&mut data)?,
            num_objects: Self::parse_num_objects(&mut data)?,
        };

        // Step 2: Parse all objects until we reach the checksum
        let mut objects = Vec::with_capacity(header.num_objects as usize);
        while data.remaining() > 20 {
            // 20 bytes reserved for SHA-1 checksum
            objects.push(Self::parse_object(&mut data)?);
        }

        // Step 3: Verify we have exactly 20 bytes left (checksum)
        if data.remaining() != 20 {
            return Err(anyhow!("Invalid packfile length"));
        }
        let _checksum = data.copy_to_bytes(20); // TODO: Validate checksum

        Ok(Packfile { header, objects })
    }

    /// Verifies and extracts the packfile signature.
    ///
    /// All valid packfiles must start with "PACK" in ASCII.
    fn parse_signature(data: &mut Bytes) -> Result<[u8; 4]> {
        if data.remaining() < 4 {
            return Err(anyhow!("Invalid packfile header"));
        }
        let sig = data[..4].try_into()?;
        if sig != *b"PACK" {
            return Err(anyhow!("Invalid packfile signature"));
        }
        data.advance(4);
        Ok(sig)
    }

    /// Parses and validates the packfile version
    ///
    /// (Currently only versions 2 and 3 are supported)
    fn parse_version(data: &mut Bytes) -> Result<u32> {
        if data.remaining() < 4 {
            return Err(anyhow!("Invalid packfile version"));
        }
        let version = data.get_u32();
        if version != 2 && version != 3 {
            return Err(anyhow!("Unsupported packfile version"));
        }
        Ok(version)
    }

    /// Reads the number of objects contained in the packfile
    fn parse_num_objects(data: &mut Bytes) -> Result<u32> {
        if data.remaining() < 4 {
            return Err(anyhow!("Invalid object count"));
        }
        Ok(data.get_u32())
    }

    /// Parses a single packed object from the data stream
    ///
    /// Git uses a sophisticated encoding for packed objects:
    /// - Variable-length size encoding
    /// - Three possible object types (base, offset delta, ref delta)
    fn parse_object(data: &mut Bytes) -> Result<PackObject> {
        // First byte contains type and initial size bits
        let byte = data.get_u8();
        let type_ = ((byte >> 4) & 0x07) as u8; // Extract type from bits 4-6
        let mut size = (byte & 0x0F) as usize; // Initial size in bits 0-3
        let mut shift = 4; // Already consumed 4 bits

        // Parse variable-length size (MSB continuation)
        if byte & 0x80 != 0 {
            // More size bytes?
            loop {
                let byte = data.get_u8();
                size |= ((byte & 0x7F) as usize) << shift;
                shift += 7;
                if byte & 0x80 == 0 {
                    // Last byte has MSB cleared
                    break;
                }
            }
        }

        match type_ {
            // Regular object types (1-4)
            1..=4 => {
                let obj_type = match type_ {
                    1 => GitObjectType::Commit,
                    2 => GitObjectType::Tree,
                    3 => GitObjectType::Blob,
                    4 => GitObjectType::Tag,
                    _ => unreachable!(), // Already validated by range
                };
                Ok(PackObject::Base {
                    type_: obj_type,
                    data: data.copy_to_bytes(size), // Read entire object data
                })
            }
            // Offset delta (relative to another object in this pack)
            6 => {
                let mut offset = 0;
                let mut byte = data.get_u8();
                offset |= (byte & 0x7F) as u64; // First 7 bits
                while byte & 0x80 != 0 {
                    // Continue while MSB set
                    byte = data.get_u8();
                    offset = (offset + 1) << 7 | (byte & 0x7F) as u64;
                }
                Ok(PackObject::OfsDelta {
                    base_offset: offset,
                    delta: data.copy_to_bytes(size), // Read delta instructions
                })
            }
            // Reference delta (points to object by hash)
            7 => {
                let mut hash = [0u8; 20];
                data.copy_to_slice(&mut hash); // Read 20-byte base hash
                Ok(PackObject::RefDelta {
                    base_hash: hash,
                    delta: data.copy_to_bytes(size), // Read delta instructions
                })
            }
            _ => Err(anyhow!("Invalid object type")),
        }
    }

    pub async fn apply_deltas(&mut self, object_store: &impl ObjectStore) -> Result<()> {
        let mut bases = HashMap::new();

        for obj in &self.objects {
            if let PackObject::Base { type_, data } = obj {
                let hash = Self::calculate_hash(*type_, &data)?;
                bases.insert(hash, data.clone());
            }
        }

        for obj in &mut self.objects {
            match obj {
                PackObject::OfsDelta { base_offset, delta } => {
                    let base = self.find_base_by_offset(*base_offset)?;
                    let reconstructed = apply_delta(&base, delta)?;
                    *obj = PackObject::Base {
                        type_: Self::detect_type(&reconstructed)?,
                        data: reconstructed.into(),
                    };
                }
                PackObject::RefDelta { base_hash, delta } => {
                    let base = object_store
                        .get_object(base_hash)
                        .await?
                        .ok_or(anyhow!("Missing base object"))?;
                    let reconstructed = apply_delta(&base.data(), delta)?;
                    *obj = PackObject::Base {
                        type_: Self::detect_type(&reconstructed)?,
                        data: reconstructed.into(),
                    };
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn find_base_by_offset(&self, offset: u64) -> Result<Vec<u8>> {
        unimplemented!()
    }

    fn calculate_hash(type_: ObjectType, data: &[u8]) -> Result<[u8; 20]> {
        unimplemented!()
    }

    fn detect_type(data: &[u8]) -> Result<ObjectType> {
        unimplemented!()
    }
}
