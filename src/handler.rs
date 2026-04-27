use anyhow::Result;
use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::protocol::RespCodec;
use crate::store::KvStore;

/// Handle a single client connection
pub async fn handle_client(mut socket: TcpStream, store: KvStore) -> Result<()> {
    let mut buffer = BytesMut::with_capacity(4096);

    loop {
        let n = socket.read_buf(&mut buffer).await?;
        if n == 0 {
            return Ok(());
        }

        while let Ok(Some(command)) = RespCodec::parse_command(&mut buffer) {
            let response = handle_command(command, &store).await;
            socket.write_all(&response).await?;
        }
    }
}

/// Process a single command
pub async fn handle_command(command: Vec<Vec<u8>>, store: &KvStore) -> Vec<u8> {
    if command.is_empty() {
        return RespCodec::encode_error("empty command");
    }

    let cmd = match std::str::from_utf8(&command[0]) {
        Ok(s) => s.to_uppercase(),
        Err(_) => return RespCodec::encode_error("invalid command encoding"),
    };

    match cmd.as_str() {
        "GET" => {
            if command.len() != 2 {
                return RespCodec::encode_error("GET expects 1 argument");
            }
            let key = match std::str::from_utf8(&command[1]) {
                Ok(k) => k,
                Err(_) => return RespCodec::encode_error("invalid key encoding"),
            };

            match store.get(key).await {
                Ok(Some(value)) => RespCodec::encode_bulk_string(&value),
                Ok(None) => RespCodec::encode_null(),
                Err(e) => RespCodec::encode_error(&e.to_string()),
            }
        }
        "SET" => {
            if command.len() < 3 || command.len() > 5 {
                return RespCodec::encode_error("SET expects key, value, and optional EX seconds");
            }

            let key = match std::str::from_utf8(&command[1]) {
                Ok(k) => k.to_string(),
                Err(_) => return RespCodec::encode_error("invalid key encoding"),
            };

            let value = command[2].clone();
            let mut ttl: Option<u64> = None;

            // Handle optional EX parameter
            if command.len() >= 4 {
                let modifier = match std::str::from_utf8(&command[3]) {
                    Ok(m) => m.to_uppercase(),
                    Err(_) => return RespCodec::encode_error("invalid modifier encoding"),
                };

                if modifier == "EX" && command.len() >= 5 {
                    match std::str::from_utf8(&command[4]) {
                        Ok(secs_str) => match secs_str.parse::<u64>() {
                            Ok(secs) => ttl = Some(secs),
                            Err(_) => return RespCodec::encode_error("invalid EX value"),
                        },
                        Err(_) => return RespCodec::encode_error("invalid EX encoding"),
                    }
                }
            }

            match store.set(key, value, ttl).await {
                Ok(_) => RespCodec::encode_simple_string("OK"),
                Err(e) => RespCodec::encode_error(&e.to_string()),
            }
        }
        "DEL" => {
            if command.len() < 2 {
                return RespCodec::encode_error("DEL expects at least 1 key");
            }

            let mut deleted_count = 0;
            for key_bytes in &command[1..] {
                if let Ok(key) = std::str::from_utf8(key_bytes) {
                    match store.delete(key).await {
                        Ok(true) => deleted_count += 1,
                        Ok(false) => {}
                        Err(_) => {}
                    }
                }
            }

            RespCodec::encode_integer(deleted_count)
        }
        "PING" => {
            if command.len() == 1 {
                RespCodec::encode_simple_string("PONG")
            } else {
                RespCodec::encode_bulk_string(&command[1])
            }
        }
        "ECHO" => {
            if command.len() != 2 {
                return RespCodec::encode_error("ECHO expects 1 argument");
            }
            RespCodec::encode_bulk_string(&command[1])
        }
        _ => RespCodec::encode_error(&format!("unknown command: {}", cmd)),
    }
}
