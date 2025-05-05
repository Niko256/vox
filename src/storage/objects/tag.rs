use crate::storage::objects::{Storable, VoxObject};
use crate::storage::utils::{OBJ_DIR, OBJ_TYPE_TAG};
use anyhow::{anyhow, Context, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

pub struct Tag {
    pub object: String,
    pub object_type: String,
    pub tag: String,
    pub tagger: (String, String, chrono::DateTime<chrono::Utc>),
    pub message: String,
}

impl Tag {
    pub fn parse(data: &str) -> Result<Self> {
        let mut lines = data.lines();
        let mut object = None;
        let mut object_type = None;
        let mut tag_name = None;
        let mut tagger = None;
        let mut message = String::new();
        let mut in_message = false;

        for line in lines {
            if in_message {
                message.push_str(line);
                continue;
            }

            if line.is_empty() {
                in_message = true;
                continue;
            }

            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() != 2 {
                continue;
            }

            match parts[0] {
                "object" => object = Some(parts[1].trim().to_string()),
                "type" => object_type = Some(parts[1].trim().to_string()),
                "tagger" => {
                    tagger = Some(Self::parse_identity(parts[1])?);
                }
                _ => {}
            }
        }

        Ok(Tag {
            object: object.ok_or_else(|| anyhow!("Missing object in tag"))?,
            object_type: object_type.ok_or_else(|| anyhow!("Missing object type in tag"))?,
            tag: tag_name.ok_or_else(|| anyhow!("Missing tag name"))?,
            tagger: tagger.ok_or_else(|| anyhow!("Missing tagger"))?,
            message: message.trim().to_string(),
        })
    }

    fn parse_identity(s: &str) -> Result<(String, String, chrono::DateTime<chrono::Utc>)> {
        // format: "Name <email> timestamp timezone"
        let re = regex::Regex::new(r"^(.*) <(.*?)> (\d+) ([\+\-]\d{4})$")?;
        let caps = re
            .captures(s)
            .ok_or_else(|| anyhow!("Invalid tagger format"))?;

        let name = caps[1].trim().to_string();
        let email = caps[2].trim().to_string();
        let timestamp = caps[3].parse::<i64>()?;
        let timezone_offset = caps[4].parse::<i32>()? * 36;

        let dt = chrono::NaiveDateTime::from_timestamp_opt(timestamp, 0)
            .ok_or_else(|| anyhow!("Invalid timestamp"))?
            .and_local_timezone(chrono::FixedOffset::east_opt(timezone_offset).unwrap())
            .unwrap()
            .to_utc();

        Ok((name, email, dt))
    }
}

impl VoxObject for Tag {
    fn object_type(&self) -> &str {
        OBJ_TYPE_TAG
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut content = Vec::new();
        writeln!(content, "object {}", self.object)?;
        writeln!(content, "type {}", self.object_type)?;
        writeln!(content, "tag {}", self.tag)?;
        writeln!(
            content,
            "tagger {} <{}> {} {}",
            self.tagger.0,
            self.tagger.1,
            self.tagger.2.timestamp(),
            self.tagger.2.format("%z").to_string()
        )?;
        writeln!(content)?;
        write!(content, "{}", self.message)?;
        Ok(content)
    }

    fn hash(&self) -> Result<String> {
        let mut hasher = Sha1::new();
        hasher.update(&self.serialize()?);
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn object_path(&self) -> Result<String> {
        let hash = self.hash()?;
        Ok(format!(
            "{}/{}/{}",
            OBJ_DIR.display(),
            &hash[..2],
            &hash[2..]
        ))
    }
}

impl Storable for Tag {
    fn save(&self, objects_dir: &Path) -> Result<String> {
        let hash = self.hash()?;
        let content = self.serialize()?;

        let header = format!("tag {}\0", content.len());
        let full_content = [header.as_bytes(), &content].concat();

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&full_content)?;
        let compressed_data = encoder.finish()?;

        let dir_path = objects_dir.join(&hash[..2]);
        fs::create_dir_all(&dir_path)?;
        let object_path = dir_path.join(&hash[2..]);
        fs::write(&object_path, compressed_data)?;

        Ok(hash)
    }
}

impl Tag {
    pub fn load(hash: &str, objects_dir: &Path) -> Result<Self> {
        let dir_path = objects_dir.join(&hash[..2]);
        let object_path = dir_path.join(&hash[2..]);

        let compressed_data = fs::read(&object_path)
            .with_context(|| format!("Failed to read tag object at {}", object_path.display()))?;

        let mut decoder = ZlibDecoder::new(&compressed_data[..]);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;

        let data = String::from_utf8(decompressed_data)?;

        let mut id = None;
        let mut target = None;

        for line in data.lines() {
            if let Some(stripped) = line.strip_prefix("id ") {
                id = Some(stripped.to_string());
            } else if let Some(stripped) = line.strip_prefix("target ") {
                target = Some(stripped.to_string());
            }
        }

        let id = id.ok_or_else(|| anyhow!("Missing 'id' in tag object"))?;
        let target = target.ok_or_else(|| anyhow!("Missing 'target' in tag object"))?;

        Ok(Tag { id, target })
    }
}
