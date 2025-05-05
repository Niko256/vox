use anyhow::{anyhow, bail, Result};
use std::ops::Range;

/// Represents a single delta operation - either Copy or Insert
#[derive(Debug)]
enum DeltaOp {
    /// Copy operation that references data from the base object
    Copy {
        /// Offset in base object to copy from
        offset: usize,
        /// Number of bytes to copy
        length: usize,
    },
    /// Insert operation that adds new data
    Insert(Vec<u8>),
}

/// The delta format is a compact binary format that encodes differences between objects
/// It consists of:
///   1. Header with base and result sizes
///   2. Series of COPY/INSERT operations
struct Delta<'a> {
    /// The raw delta data being parsed
    data: &'a [u8],
    /// Current read position within the data
    position: usize,
}

impl<'a> Delta<'a> {
    /// Creates a new Delta from raw bytes
    fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    /// -------------------------------------------------------------------
    /// DELTA HEADER PARSING
    /// -------------------------------------------------------------------
    /// Header format:
    ///   [base_size][result_size]
    ///
    /// Where sizes are variable-length integers (see read_size())
    ///
    fn parse_header(&mut self) -> Result<(usize, usize)> {
        let base_size = self.read_size()?;
        let result_size = self.read_size()?;
        Ok((base_size, result_size))
    }

    /// -------------------------------------------------------------------
    /// OPERATION PARSING
    /// -------------------------------------------------------------------
    /// Delta operations come in two types:
    ///   1. COPY - references data from base object
    ///   2. INSERT - adds new data
    ///
    fn parse_ops(&mut self) -> Result<Vec<DeltaOp>> {
        let mut ops = Vec::new();

        while self.position < self.data.len() {
            let cmd = self.read_byte()?;

            // determines operation type
            if cmd & 0x80 != 0 {
                // indicates a Copy operation
                let (offset, length) = self.parse_copy_op(cmd)?;
                ops.push(DeltaOp::Copy { offset, length });
            } else {
                // Otherwise it's an Insert operation
                let data = self.parse_insert_data(cmd)?;
                ops.push(DeltaOp::Insert(data));
            }
        }

        Ok(ops)
    }

    /// Parses a Copy operation from the command byte
    ///
    /// # Arguments
    /// * `cmd` - The command byte
    ///
    /// # Returns
    /// Tuple of (offset, length) for the copy operation
    ///
    /// # Errors
    /// Returns error if the offset/length encoding is invalid
    ///
    fn parse_copy_op(&mut self, cmd: u8) -> Result<(usize, usize)> {
        let mut offset = 0;
        let mut length = 0;

        // ----------------------------------------------------------------------------
        // PARSE OFFSET (bits 0-3 of cmd byte)
        // ----------------------------------------------------------------------------
        // The command byte uses bits 0-3 as flags indicating which offset bytes follow:
        // - Bit 0 (0x01): if set, offset byte 0 exists
        // - Bit 1 (0x02): if set, offset byte 1 exists
        // - Bit 2 (0x04): if set, offset byte 2 exists
        // - Bit 3 (0x08): if set, offset byte 3 exists
        //
        // Each existing byte contributes 8 bits to the final offset value
        // in little-endian order (first byte is least significant)
        for i in 0..4 {
            // Check if bit 'i' is set in the command byte
            if cmd & (1 << i) != 0 {
                // Read the next byte and incorporate it into the offset
                // Shift it left by (i * 8) bits to place it in correct position
                // Example:
                // - If i=0 (first byte), no shift (least significant byte)
                // - If i=1, shift left 8 bits (second byte)
                // - etc.
                offset |= (self.read_byte()? as usize) << (i * 8);
            }
        }

        // ------------------------------------------------------------------------
        // PARSE LENGTH (bits 4-6 of cmd byte)
        // ------------------------------------------------------------------------
        // The command byte uses bits 4-6 as flags indicating which length bytes follow:
        // - Bit 4 (0x10): if set, length byte 0 exists
        // - Bit 5 (0x20): if set, length byte 1 exists
        // - Bit 6 (0x40): if set, length byte 2 exists
        //
        // Each existing byte contributes 8 bits to the final length value
        // in little-endian order (first byte is least significant)
        for i in 4..7 {
            // Check if bit 'i' is set in the command byte
            if cmd & (1 << i) != 0 {
                // Read the next byte and incorporate it into the length
                // Shift it left by ((i-4) * 8) bits to normalize position
                // (since we're using bits 4-6 but want to treat them as 0-2)
                // Example:
                // - If i=4, shift left 0 bits (length byte 0)
                // - If i=5, shift left 8 bits (length byte 1)
                // - If i=6, shift left 16 bits (length byte 2)
                length |= (self.read_byte()? as usize) << ((i - 4) * 8);
            }
        }

        // ------------------------------------------------------------------------
        // SPECIAL CASE HANDLING
        // ------------------------------------------------------------------------
        // In Git's delta format, a length of 0 is special and means 64KB (0x10000)
        // This allows representing large copies efficiently
        //
        if length == 0 {
            length = 0x10000; // 64KB
        }

        Ok((offset, length))
    }

    /// -------------------------------------------------------------------
    /// INSERT OPERATION PARSING
    /// -------------------------------------------------------------------
    /// Format:
    /// [LENGTH][DATA...]
    /// Where:
    /// - LENGTH is a 1-byte value (0 <= LENGTH <= 127)
    /// - Followed by exactly LENGTH bytes of data
    ///
    /// # Arguments
    /// * `cmd` - The command byte (length of data to insert)
    ///
    fn parse_insert_data(&mut self, cmd: u8) -> Result<Vec<u8>> {
        let length = cmd as usize;
        let start = self.position;
        let end = start + length;

        if end > self.data.len() {
            bail!("Insert operation overflows delta data");
        }

        self.position = end;
        Ok(self.data[start..end].to_vec())
    }

    /// Reads a single byte from the delta data
    ///
    /// # Errors
    /// Returns error if we've reached end of data
    ///
    fn read_byte(&mut self) -> Result<u8> {
        if self.position >= self.data.len() {
            bail!("Unexpected end of delta");
        }
        let b = self.data[self.position];
        self.position += 1;
        Ok(b)
    }

    /// Reads a variable-length size encoding
    ///
    /// # Returns
    /// The decoded size value
    ///
    fn read_size(&mut self) -> Result<usize> {
        let mut size = 0;
        let mut shift = 0;
        loop {
            let byte = self.read_byte()?;
            size |= ((byte & 0x7F) as usize) << shift;
            if (byte & 0x80) == 0 {
                break;
            }
            shift += 7;
        }
        Ok(size)
    }
}

/// -------------------------------------------------------------------
/// DELTA APPLICATION
/// -------------------------------------------------------------------
/// Reconstructs target object by applying delta to base object
///
/// # Arguments
/// * `base` - The base object data
/// * `delta` - The delta data to apply
///
/// # Returns
/// The reconstructed target object data
///
/// # Errors
/// Returns error if:
/// - Base size doesn't match delta header
/// - Any operation is invalid
/// - Result size doesn't match delta header
///
pub fn apply_delta(base: &[u8], delta: &[u8]) -> Result<Vec<u8>> {
    let mut parser = Delta::new(delta);

    // 1: Parse header and validate
    let (base_size, result_size) = parser.parse_header()?;
    if base.len() != base_size {
        bail!(
            "Base size mismatch (expected {}, got {})",
            base_size,
            base.len()
        );
    }

    // 2: Parse all operations
    let ops = parser.parse_ops()?;

    // 3: Apply operations
    let mut result = Vec::with_capacity(result_size);
    for op in ops {
        match op {
            DeltaOp::Copy { offset, length } => {
                let end = offset + length;
                if end > base.len() {
                    bail!(
                        "Copy range {}-{} exceeds base size {}",
                        offset,
                        end,
                        base.len()
                    );
                }
                result.extend_from_slice(&base[offset..end]);
            }
            DeltaOp::Insert(data) => {
                result.extend(data);
            }
        }
    }

    // 4: Verify final size matches header
    if result.len() != result_size {
        bail!(
            "Result size mismatch (expected {}, got {})",
            result_size,
            result.len()
        );
    }

    Ok(result)
}
