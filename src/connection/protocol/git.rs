use anyhow::{anyhow, Context, Result};
use bytes::{Buf, Bytes};
use bytes::{BufMut, BytesMut};
use std::collections::HashMap;
use std::collections::HashSet;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use url::Url;

/// Represents a Git reference (branch/tag) with its associated hash and capabilities
#[derive(Debug)]
pub struct GitRef {
    /// The full reference name (e.g., "refs/heads/main")
    pub name: String,
    /// The SHA-1 hash of the referenced commit
    pub hash: String,
    /// List of server capabilities advertised with this reference
    pub capabilities: Vec<String>,
}

/// Represents the response from a packfile fetch operation
#[derive(Debug)]
pub struct PackfileResponse {
    /// The raw packfile data (may be sideband-encoded)
    pub data: Bytes,
    /// Final acknowledgment status from server
    pub ack_status: Option<AckStatus>,
    /// All commit hashes acknowledged during negotiation
    pub seen_acks: HashSet<String>,
}

/// Represents server acknowledgment status during negotiation
#[derive(Debug)]
pub enum AckStatus {
    /// Positive acknowledgment with commit hash
    Ack(String),
    /// Negative acknowledgment
    Nak,
}

/// Handles Git protocol communication for discovering repository references
pub struct GitProtocolHandler;

impl GitProtocolHandler {
    /// Discovers repository references by communicating with a Git server
    /// # Arguments
    /// * `url` - Git server URL
    ///
    /// # Returns
    /// Vector of `GitRef` containing all repository references
    ///
    pub async fn discover_repo(url: &str) -> Result<Vec<GitRef>> {
        // Parse the Git URL into components
        let (host, port, path) = Self::parse_git_url(url)?;

        // Establish TCP connection to Git server
        let mut stream = TcpStream::connect(format!("{}:{}", host, port)).await?;

        // Build and send the initial protocol request
        let request = Self::build_initial_request(&path, &host);
        stream.write_all(&request).await?;

        // Read server response in chunks
        let mut response = Vec::new();
        let mut buf = [0u8; 4096]; // 4KB read buffer
        loop {
            let n = stream.read(&mut buf).await?;
            if n == 0 {
                // Connection closed by server
                break;
            }
            response.extend_from_slice(&buf[..n]); // Append received data
        }

        // Parse the protocol response into GitRef objects
        Self::parse_upload_pack_response(&response)
    }

    /// Builds the initial protocol request for git-upload-pack.
    fn build_initial_request(path: &str, host: &str) -> BytesMut {
        let mut buf = BytesMut::new();

        // Format: "git-upload-pack PATH\0host=HOST\0"
        buf.put_slice(b"git-upload-pack "); // Service identifier
        buf.put_slice(path.as_bytes()); // Repository path
        buf.put_slice(b"\0host="); // Null-byte separator
        buf.put_slice(host.as_bytes()); // Host information
        buf.put_slice(b"\0"); // Terminating null-byte
        buf
    }

    /// Parses a Git URL into its components (host, port, path)
    fn parse_git_url(url: &str) -> Result<(String, u16, String)> {
        let parsed = Url::parse(url)?;

        // Extract host (required)
        let host = parsed
            .host_str()
            .ok_or(anyhow!("URL must contain host"))?
            .to_string();

        // Use default Git port (9418) if not specified
        let port = parsed.port().unwrap_or(9418);

        // Extract and clean path
        let path = parsed.path().trim_start_matches('/');
        if path.is_empty() {
            return Err(anyhow!("URL must contain repository path"));
        }

        Ok((host, port, path.to_string()))
    }

    /// Parses the server's upload-pack response into GitRef objects
    fn parse_upload_pack_response(data: &[u8]) -> Result<Vec<GitRef>> {
        let mut refs = Vec::new();
        // Split response into lines
        let mut lines = data.split(|&b| b == b'\n');

        // First line contains HEAD reference and capabilities
        if let Some(first_line) = lines.next() {
            if let Ok((head_ref, caps)) = Self::parse_head_line(first_line) {
                refs.push(head_ref);

                // Process remaining reference lines
                for line in lines {
                    if line.is_empty() {
                        continue; // Skip empty lines
                    }

                    // Parse regular reference lines (format: "hash refname")
                    if let Ok((name, hash)) = Self::parse_ref_line(line) {
                        refs.push(GitRef {
                            name,
                            hash,
                            capabilities: caps.clone(), // Share capabilities from HEAD
                        });
                    }
                }
            }
        }

        Ok(refs)
    }

    /// Parses the special first line containing HEAD reference and capabilities
    fn parse_head_line(line: &[u8]) -> Result<(GitRef, Vec<String>)> {
        // Split at null byte: "hash refname\0cap1 cap2"
        let mut parts = line.splitn(2, |&b| b == b'\0');
        let ref_part = parts.next().ok_or(anyhow!("Invalid head line"))?;
        let caps_part = parts.next().unwrap_or_default(); // Capabilities are optional

        // Split reference part into hash and name
        let (hash, name) = Self::split_at_space(ref_part)?;
        // Parse capabilities string into vector
        let capabilities = Self::parse_capabilities(caps_part);

        Ok((
            GitRef {
                name: name.to_string(),
                hash: hash.to_string(),
                capabilities: capabilities.clone(),
            },
            capabilities,
        ))
    }

    /// Parses a regular reference line (without capabilities)
    fn parse_ref_line(line: &[u8]) -> Result<(String, String)> {
        // Simple "hash refname" format
        let (hash, name) = Self::split_at_space(line)?;
        Ok((name.to_string(), hash.to_string()))
    }

    /// Splits a line at the first space character
    fn split_at_space(line: &[u8]) -> Result<(&str, &str)> {
        // Convert bytes to UTF-8 string
        let line_str = std::str::from_utf8(line)?;
        // Split at first space
        let mut parts = line_str.splitn(2, ' ');
        let hash = parts.next().ok_or(anyhow!("No hash in line"))?;
        let name = parts.next().ok_or(anyhow!("No ref name in line"))?;
        Ok((hash, name))
    }

    /// Parses capabilities string into individual capabilities
    fn parse_capabilities(caps: &[u8]) -> Vec<String> {
        std::str::from_utf8(caps)
            .unwrap_or("") // Fallback to empty string if not UTF-8
            .split_whitespace() // Split by any whitespace
            .map(|s| s.to_string()) // Convert &str to String
            .collect()
    }

    /// Fetches a packfile from a Git server using the upload-pack protocol
    ///
    /// # Arguments
    /// * `url` - Git repository URL (e.g., "git://github.com/user/repo.git")
    /// * `wants` - List of commit hashes the client wants
    /// * `hashes` - List of commit hashes the client already has (for delta compression)
    ///
    /// # Returns
    /// `PackfileResponse` containing the packfile data and negotiation status
    ///
    pub async fn fetch_pack(
        url: &str,
        wants: &[String],
        hashes: &[String],
    ) -> Result<PackfileResponse> {
        // Step 1: Parse URL and establish connection
        let (host, port, path) = Self::parse_git_url(url)?;
        let mut stream = TcpStream::connect(format!("{}:{}", host, port)).await?;

        // Step 2: Build and send fetch request
        let request = Self::build_fetch_request(&path, &host, wants, hashes)?;
        stream.write_all(&request).await?;

        // Step 3: Process server response
        let mut pack_data = BytesMut::new(); // Buffer for accumulated packfile data
        let mut ack_status = None; // Final ACK/NAK status
        let mut seen_acks = HashSet::new(); // All received ACKs during negotiation

        loop {
            // Read data in 4KB chunks
            let mut buf = [0u8; 4096];
            let n = stream.read(&mut buf).await?;
            if n == 0 {
                break;
            } // Server closed connection

            let mut chunk = Bytes::copy_from_slice(&buf[..n]);

            // Process each packet in the chunk
            while !chunk.is_empty() {
                match chunk[0] {
                    // Sideband data packet (actual packfile content)
                    1 => {
                        let len = Self::parse_sideband_len(&mut chunk)?;
                        // Extract payload (skip 1-byte sideband type)
                        pack_data.extend_from_slice(&chunk.split_to(len - 1)[1..]);
                    }

                    // Progress message (ignored)
                    2 => {
                        let len = Self::parse_sideband_len(&mut chunk)?;
                        chunk.advance(len - 1); // Skip progress messages
                    }

                    // Error message
                    3 => {
                        let len = Self::parse_sideband_len(&mut chunk)?;
                        let err_msg = String::from_utf8(chunk[1..len - 1].to_vec())?;
                        return Err(anyhow!("Server error: {}", err_msg));
                    }

                    // ACK/NAK response
                    b'A' | b'N' => {
                        ack_status = Some(Self::parse_ack(&mut chunk)?);
                        if let Some(AckStatus::Ack(ref hash)) = ack_status {
                            seen_acks.insert(hash.clone()); // Track acknowledged hashes
                        }
                    }

                    // Raw PACK header (unbanded data)
                    b'P' if &chunk[..4] == b"PACK" => {
                        pack_data.extend_from_slice(&chunk);
                        chunk.advance(chunk.len());
                    }

                    // Unknown packet type
                    _ => return Err(anyhow!("Unexpected packet type")),
                }
            }
        }

        Ok(PackfileResponse {
            data: pack_data.freeze(), // Convert to immutable Bytes
            ack_status,
            seen_acks,
        })
    }

    /// Builds a fetch request for the git-upload-pack protocol
    ///
    /// # Protocol Format
    /// 1. Initial request line with path and host
    /// 2. "want" lines for desired commits
    /// 3. Capability declarations
    /// 4. "have" lines for common commits
    /// 5. "done" marker
    fn build_fetch_request(
        path: &str,
        host: &str,
        wants: &[String],
        hashes: &[String],
    ) -> Result<BytesMut> {
        let mut buf = BytesMut::new();

        // Initial request line with null-terminated headers
        buf.put_slice(b"git-upload-pack ");
        buf.put_slice(path.as_bytes());
        buf.put_slice(b"\0host=");
        buf.put_slice(host.as_bytes());
        buf.put_slice(b"\0");

        // List wanted commits (client requirements)
        for want in wants {
            buf.put_slice(b"want ");
            buf.put_slice(want.as_bytes());
            buf.put_slice(b"\n");
        }

        // Advertise client capabilities
        buf.put_slice(b"multi_ack\n"); // Support multiple ACKs
        buf.put_slice(b"side-band-64k\n"); // Use sideband for data transfer
        buf.put_slice(b"no-progress\n"); // Don't send progress messages

        // List common commits (server can omit these)
        for have in hashes {
            buf.put_slice(b"have ");
            buf.put_slice(have.as_bytes());
            buf.put_slice(b"\n");
        }

        // End of request
        buf.put_slice(b"done\n");

        Ok(buf)
    }

    /// Parses the length prefix of a sideband packet.
    ///
    /// # Packet Format
    /// 4-byte length header (network byte order)
    /// Followed by payload of (length-4) bytes
    ///
    fn parse_sideband_len(chunk: &mut Bytes) -> Result<usize> {
        if chunk.len() < 4 {
            return Err(anyhow!("Invalid sideband packet"));
        }

        // Read prefix
        let len_bytes = &chunk[..4];
        let len =
            u32::from_be_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]) as usize;

        // Validate length
        if len < 4 || len > chunk.len() {
            return Err(anyhow!("Invalid packet length"));
        }

        chunk.advance(4); // Consume length header
        Ok(len)
    }

    /// Parses an ACK/NAK response from the server.
    ///
    /// # Possible Responses
    /// - "ACK <hash>" (positive acknowledgment)
    /// - "NAK" (negative acknowledgment)
    ///
    fn parse_ack(chunk: &mut Bytes) -> Result<AckStatus> {
        if chunk.len() < 3 {
            return Err(anyhow!("Invalid ACK packet"));
        }

        match chunk[0] {
            b'A' => {
                // Extract line and parse hash
                let ack_line =
                    String::from_utf8(chunk[..].split(|&b| b == b'\n').next().unwrap().to_vec())?;
                let hash = ack_line
                    .split_whitespace()
                    .nth(1)
                    .ok_or(anyhow!("Missing hash in ACK"))?;
                Ok(AckStatus::Ack(hash.to_string()))
            }
            b'N' => Ok(AckStatus::Nak),
            _ => Err(anyhow!("Invalid ACK type")),
        }
    }
}
