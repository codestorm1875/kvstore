use anyhow::Result;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() -> Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:6379")?;

    // Test PING
    println!("Testing PING:");
    send_command(&mut stream, &["PING"])?;
    println!();

    // Test SET
    println!("Testing SET key1 'hello':");
    send_command(&mut stream, &["SET", "key1", "hello"])?;
    println!();

    // Test GET
    println!("Testing GET key1:");
    send_command(&mut stream, &["GET", "key1"])?;
    println!();

    // Test SET with TTL
    println!("Testing SET key2 'world' EX 10 (expires in 10 seconds):");
    send_command(&mut stream, &["SET", "key2", "world", "EX", "10"])?;
    println!();

    // Test GET
    println!("Testing GET key2:");
    send_command(&mut stream, &["GET", "key2"])?;
    println!();

    // Test DEL
    println!("Testing DEL key1:");
    send_command(&mut stream, &["DEL", "key1"])?;
    println!();

    // Test GET on deleted key
    println!("Testing GET key1 (should be nil):");
    send_command(&mut stream, &["GET", "key1"])?;
    println!();

    // Test ECHO
    println!("Testing ECHO 'Hello, KVStore!':");
    send_command(&mut stream, &["ECHO", "Hello, KVStore!"])?;
    println!();

    Ok(())
}

fn send_command(stream: &mut TcpStream, args: &[&str]) -> Result<()> {
    // Build RESP array
    let mut request = format!("*{}\r\n", args.len()).into_bytes();
    for arg in args {
        request.extend_from_slice(format!("${}\r\n", arg.len()).as_bytes());
        request.extend_from_slice(arg.as_bytes());
        request.extend_from_slice(b"\r\n");
    }

    // Send command
    stream.write_all(&request)?;
    println!("-> {}", args.join(" "));

    // Read response
    let mut response = vec![0; 1024];
    let n = stream.read(&mut response)?;
    let response_str = String::from_utf8_lossy(&response[..n]);
    println!("<- {}", response_str.trim());

    Ok(())
}
