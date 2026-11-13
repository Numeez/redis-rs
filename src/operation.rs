use crate::{
    buffer::Buffer,
    hash,
    protocol::{self, out_arr, out_nil, out_str},
};

#[derive(Debug)]
pub enum Command {
    Get { key: Vec<u8> },
    Set { key: Vec<u8>, val: Vec<u8> },
    Del { key: Vec<u8> },
    Keys,
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
        (b"keys", 1) => Ok(Command::Keys),
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid arguments",
            ));
        }
    }
}

pub fn execute(output_buffer: &mut Buffer, cmd: Command, store: &mut hash::HMap) {
    match cmd {
        Command::Set { key, val } => {
            execute_set(store, key, val);
            protocol::out_nil(output_buffer);
        }
        Command::Get { key } => match execute_get(store, key) {
            Some(val) => out_str(output_buffer, val),
            None => {
                out_nil(output_buffer);
            }
        },
        Command::Del { key } => {
            let val: i64;
            if execute_del(store, key) {
                val = 1;
            } else {
                val = 0;
            }
            protocol::out_int(output_buffer, val);
        }
        Command::Keys => {
            let keys = store.keys();
            out_arr(output_buffer, keys.len() as u32);
            for key in keys {
                out_str(output_buffer, key);
            }
        }
    }
}

fn execute_set(store: &mut hash::HMap, key: Vec<u8>, val: Vec<u8>) {
    store.insert(key, val)
}

pub fn execute_get(store: &mut hash::HMap, key: Vec<u8>) -> Option<&Vec<u8>> {
    store.look_up(&key)
}

pub fn execute_del(store: &mut hash::HMap, key: Vec<u8>) -> bool {
    store.remove(&key)
}
