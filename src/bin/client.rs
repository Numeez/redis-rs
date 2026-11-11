use std::{
    io::{Read, Write},
    net::TcpStream,
};

fn main() -> std::io::Result<()> {
    let args = std::env::args();
    assert!(args.len() > 2);
    let arguments: Vec<String> = args.collect();
    let message = build_message(&arguments[1..]);
    let mut stream = TcpStream::connect("127.0.0.1:6969")?;
    stream.write_all(&message)?;
    let mut read_buffer = [0u8; 1024];
    let n = stream.read(&mut read_buffer)?;
    let response = String::from_utf8_lossy(&read_buffer[..n]);
    eprintln!("Server responded with: {response}");

    Ok(())
}

fn build_message(arguments: &[String]) -> Vec<u8> {
    let mut result: Vec<u8> = vec![];
    let nstr = arguments.len() as u32;
    let mut payload: Vec<u8> = vec![];
    payload.extend_from_slice(&u32::to_le_bytes(nstr));
    for arg in arguments.iter() {
        payload.extend_from_slice(&u32::to_le_bytes(arg.len() as u32));
        payload.extend_from_slice(arg.as_bytes());
    }
    result.extend_from_slice(&u32::to_le_bytes(payload.len() as u32));
    result.extend_from_slice(&payload);
    result
}
