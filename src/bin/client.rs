use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub const TAG_NIL: u8 = 0;
pub const TAG_ERR: u8 = 1;
pub const TAG_STR: u8 = 2;
pub const TAG_INT: u8 = 3;
pub const TAG_DBL: u8 = 4;
pub const TAG_ARR: u8 = 5;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: client <command> [args...]");
        std::process::exit(1);
    }
    let message = build_message(&args);
    let mut stream = TcpStream::connect("127.0.0.1:6969")?;
    stream.write_all(&message)?;
    let response = read_response(&mut stream)?;
    eprintln!("{response}");

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

fn read_response(stream: &mut impl Read) -> std::io::Result<String> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let length = u32::from_le_bytes(len_buf) as usize;

    let mut response = vec![0u8; length];
    stream.read_exact(&mut response)?;

    let mut body: &[u8] = &response;
    decode(&mut body)
}

fn read_u32(body: &mut &[u8]) -> std::io::Result<u32> {
    let mut b = [0u8; 4];
    body.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}

fn read_bytes(body: &mut &[u8], n: usize) -> std::io::Result<Vec<u8>> {
    let mut v = vec![0u8; n];
    body.read_exact(&mut v)?;
    Ok(v)
}

fn decode(body: &mut &[u8]) -> std::io::Result<String> {
    let mut tag = [0u8; 1];
    body.read_exact(&mut tag)?;

    match tag[0] {
        TAG_NIL => Ok("(nil)".to_string()),
        TAG_ERR => {
            let code = read_u32(body)?;
            let len = read_u32(body)? as usize;
            let msg = read_bytes(body, len)?;
            Ok(format!("(error {code}) {}", String::from_utf8_lossy(&msg)))
        }
        TAG_STR => {
            let len = read_u32(body)? as usize;
            let s = read_bytes(body, len)?;
            Ok(String::from_utf8_lossy(&s).into_owned())
        }
        TAG_INT => {
            let mut b = [0u8; 8];
            body.read_exact(&mut b)?;
            Ok(format!("(integer) {}", i64::from_le_bytes(b)))
        }
        TAG_DBL => {
            let mut b = [0u8; 8];
            body.read_exact(&mut b)?;
            Ok(format!("(double) {}", f64::from_le_bytes(b)))
        }
        TAG_ARR => {
            let n = read_u32(body)?;
            let mut lines = Vec::new();
            for i in 1..=n {
                let item = decode(body)?;
                lines.push(format!("{i}) {item}"));
            }
            if lines.is_empty() {
                Ok("(empty array)".to_string())
            } else {
                Ok(lines.join("\n"))
            }
        }
        other => Ok(format!("(unknown tag {other})")),
    }
}
