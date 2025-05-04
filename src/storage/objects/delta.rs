use anyhow::{anyhow, Context, Result};
use bytes::{Buf, Bytes};
use std::cmp;

pub fn apply_delta(base: &[u8], delta: &[u8]) -> Result<Vec<u8>> {
    let mut delta = Bytes::copy_from_slice(delta);

    let base_size = parse_delta_size(&mut delta)?;
    if base_size != base.len() as u64 {
        return Err(anyhow!("Base size mismatch"));
    }

    let result_size = parse_delta_size(&mut delta)?;
    let mut result = Vec::with_capacity(result_size as usize);

    while delta.has_remaining() {
        let instruction = delta.get_u8();

        if instruction & 0x80 != 0 {
            let mut offset = 0;
            let mut size = 0;

            for i in 0..4 {
                if instruction & (1 << i) != 0 {
                    offset |= (delta.get_u8() as u64) << (i * 8);
                }
            }

            for i in 4..7 {
                if instruction & (1 << i) != 0 {
                    size |= (delta.get_u8() as u64) << ((i - 4) * 8);
                }
            }

            if size == 0 {
                size = 0x10000;
            }

            let end = cmp::min(offset + size, base_size);
            if offset > end || end > base_size {
                return Err(anyhow!("Invalid delta copy instruction"));
            }
            result.extend_from_slice(&base[offset as usize..end as usize]);
        } else {
            let size = instruction as usize;
            if delta.remaining() < size {
                return Err(anyhow!("Invalid delta insert instruction"));
            }
            result.extend_from_slice(&delta.copy_to_bytes(size));
        }
    }

    if result.len() != result_size as usize {
        return Err(anyhow!("Result size mismatch"));
    }

    Ok(result)
}

fn parse_delta_size(delta: &mut Bytes) -> Result<u64> {
    let mut size = 0;
    let mut shift = 0;

    loop {
        if !delta.has_remaining() {
            return Err(anyhow!("Unexpected end of delta"));
        }

        let byte = delta.get_u8();
        size |= ((byte & 0x7F) as u64) << shift;
        shift += 7;

        if byte & 0x80 == 0 {
            break;
        }

        if shift > 64 {
            return Err(anyhow!("Delta size too large"));
        }
    }

    Ok(size)
}
