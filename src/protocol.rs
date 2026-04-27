use anyhow::Result;
use bytes::{Buf, BytesMut};

/// RESP (REdis Serialization Protocol) parser and responder
pub struct RespCodec;

impl RespCodec {
    /// Parse a RESP array from buffer
    pub fn parse_command(buffer: &mut BytesMut) -> Result<Option<Vec<Vec<u8>>>> {
        if buffer.is_empty() {
            return Ok(None);
        }

        if buffer[0] != b'*' {
            return Err(anyhow::anyhow!("Invalid RESP format"));
        }

        // Find first \r\n
        let mut pos = 1;
        while pos < buffer.len() && buffer[pos] != b'\r' {
            pos += 1;
        }

        if pos >= buffer.len() - 1 {
            return Ok(None); // Incomplete
        }

        let array_len: usize = std::str::from_utf8(&buffer[1..pos])?
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid array length"))?;

        pos += 2; // Skip \r\n

        let mut elements = Vec::new();
        for _ in 0..array_len {
            if pos >= buffer.len() || buffer[pos] != b'$' {
                return Ok(None); // Incomplete
            }
            pos += 1;

            // Find \r\n for bulk string length
            let len_start = pos;
            while pos < buffer.len() && buffer[pos] != b'\r' {
                pos += 1;
            }

            if pos >= buffer.len() - 1 {
                return Ok(None); // Incomplete
            }

            let str_len: usize = std::str::from_utf8(&buffer[len_start..pos])?
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid string length"))?;

            pos += 2; // Skip \r\n

            if pos + str_len + 2 > buffer.len() {
                return Ok(None); // Incomplete
            }

            let element = buffer[pos..pos + str_len].to_vec();
            elements.push(element);
            pos += str_len + 2; // Skip \r\n
        }

        buffer.advance(pos);
        Ok(Some(elements))
    }

    /// Encode a response in RESP format
    pub fn encode_simple_string(msg: &str) -> Vec<u8> {
        format!("+{}\r\n", msg).into_bytes()
    }

    pub fn encode_bulk_string(data: &[u8]) -> Vec<u8> {
        let mut result = format!("${}\r\n", data.len()).into_bytes();
        result.extend_from_slice(data);
        result.extend_from_slice(b"\r\n");
        result
    }

    pub fn encode_null() -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }

    pub fn encode_error(msg: &str) -> Vec<u8> {
        format!("-ERR {}\r\n", msg).into_bytes()
    }

    pub fn encode_integer(num: i64) -> Vec<u8> {
        format!(":{}\r\n", num).into_bytes()
    }
}
