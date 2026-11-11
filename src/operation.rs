use std::collections::HashMap;

use crate::{RES_NX, RES_OK, buffer::Buffer};

#[derive(Debug)]
pub enum Command {
    Get { key: Vec<u8> },
    Set { key: Vec<u8>, val: Vec<u8> },
    Del { key: Vec<u8> },
}

pub fn write_response(output_buffer: &mut Buffer, status: u32, data: &[u8]) {
    let length = 4 + data.len() as u32;
    output_buffer.append(&length.to_le_bytes());
    output_buffer.append(&status.to_le_bytes());
    output_buffer.append(data);
}

pub fn parse_command(mut data: Vec<Vec<u8>>) -> std::io::Result<Command> {
    if data.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "invalid arguments",
        ));
    }
    let name = data[0].to_ascii_lowercase();
    match (name.as_slice(), data.len()) {
        (b"set", 3) => {
            let val = data.pop().unwrap();
            let key = data.pop().unwrap();
            Ok(Command::Set { key, val })
        }
        (b"get", 2) => Ok(Command::Get {
            key: data.pop().unwrap(),
        }),
        (b"del", 2) => Ok(Command::Del {
            key: data.pop().unwrap(),
        }),
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid arguments",
            ));
        }
    }
}

pub fn execute(output_buffer: &mut Buffer, cmd: Command, store: &mut HashMap<Vec<u8>, Vec<u8>>) {
    match cmd {
        Command::Set { key, val } => {
            execute_set(store, key, val);
            write_response(output_buffer, RES_OK, b"");
        }
        Command::Get { key } => match execute_get(store, key) {
            Some(val) => write_response(output_buffer, RES_OK, val),
            None => {
                write_response(output_buffer, RES_NX, b"");
            }
        },
        Command::Del { key } => {
            execute_del(store, key);
            write_response(output_buffer, RES_OK, b"");
        }
    }
}

fn execute_set(
    store: &mut HashMap<Vec<u8>, Vec<u8>>,
    key: Vec<u8>,
    val: Vec<u8>,
) -> Option<Vec<u8>> {
    store.insert(key, val)
}

pub fn execute_get(store: &mut HashMap<Vec<u8>, Vec<u8>>, key: Vec<u8>) -> Option<&Vec<u8>> {
    store.get(&key)
}

pub fn execute_del(store: &mut HashMap<Vec<u8>, Vec<u8>>, key: Vec<u8>) -> Option<Vec<u8>> {
    store.remove(&key)
}
