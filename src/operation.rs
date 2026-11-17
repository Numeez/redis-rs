use std::time::{Duration, Instant};

use crate::{
    buffer::Buffer,
    hash::{self, Value},
    key::TTLEntry,
    min_heap::MinHeap,
    protocol::{self, ERR_UNKNOWN, out_arr, out_dbl, out_error, out_int, out_nil, out_str},
    threadpool::{ThreadPool, drop_value},
    zset::ZSet,
};

#[derive(Debug)]
pub enum Command {
    Get {
        key: Vec<u8>,
    },
    Set {
        key: Vec<u8>,
        val: Value,
    },
    Del {
        key: Vec<u8>,
    },
    ZAdd {
        key: Vec<u8>,
        score: f64,
        name: String,
    },
    ZRem {
        key: Vec<u8>,
        name: String,
    },
    ZScore {
        key: Vec<u8>,
        name: String,
    },
    ZQuery {
        key: Vec<u8>,
        score: f64,
        name: String,
        offset: u32,
        limit: u32,
    },
    Pexpire {
        key: Vec<u8>,
        dead_line: Instant,
    },
    Pttl {
        key: Vec<u8>,
    },
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
            Ok(Command::Set {
                key,
                val: Value::Str(val),
            })
        }
        (b"get", 2) => Ok(Command::Get {
            key: data.pop().unwrap(),
        }),
        (b"del", 2) => Ok(Command::Del {
            key: data.pop().unwrap(),
        }),
        (b"keys", 1) => Ok(Command::Keys),
        (b"zadd", 4) => {
            let name = parse_name(data.pop().unwrap())?;
            let score = parse_score(&data.pop().unwrap())?;
            let key = data.pop().unwrap();
            Ok(Command::ZAdd { key, score, name })
        }
        (b"zrem", 3) => {
            let name = parse_name(data.pop().unwrap())?;
            let key = data.pop().unwrap();
            Ok(Command::ZRem { key, name })
        }
        (b"zscore", 3) => {
            let name = parse_name(data.pop().unwrap())?;
            let key = data.pop().unwrap();
            Ok(Command::ZScore { key, name })
        }
        (b"zquery", 6) => {
            let limit = parse_u32(&data.pop().unwrap())?;
            let offset = parse_u32(&data.pop().unwrap())?;
            let name = parse_name(data.pop().unwrap())?;
            let score = parse_score(&data.pop().unwrap())?;
            let key = data.pop().unwrap();
            Ok(Command::ZQuery {
                key,
                score,
                name,
                offset,
                limit,
            })
        }
        (b"pexpire", 3) => {
            let ms = parse_u32(&data.pop().unwrap())?;
            let key = data.pop().unwrap();
            let dead_line = Instant::now().checked_add(Duration::from_millis(ms as u64));
            if dead_line.is_none() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid deadline",
                ));
            }
            Ok(Command::Pexpire {
                key,
                dead_line: dead_line.unwrap(),
            })
        }
        (b"pttl", 2) => {
            let key = data.pop().unwrap();
            Ok(Command::Pttl { key })
        }
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid arguments",
            ));
        }
    }
}

pub fn execute(
    output_buffer: &mut Buffer,
    cmd: Command,
    ttl_entries: &mut MinHeap<TTLEntry>,
    store: &mut hash::HMap,
    thread_pool: &ThreadPool,
) {
    match cmd {
        Command::Set { key, val } => {
            if let Some(val) = execute_set(store, key, val) {
                drop_value(val, thread_pool);
            }
            protocol::out_nil(output_buffer);
        }
        Command::Get { key } => match execute_get(store, key) {
            Some(Value::Str(val)) => out_str(output_buffer, val),
            Some(Value::ZSet(_)) => out_error(output_buffer, ERR_UNKNOWN, b"expect string"),
            None => out_nil(output_buffer),
        },
        Command::Del { key } => {
            let val: i64;
            let removed = execute_del(store, key);
            if removed.is_some() {
                drop_value(removed.unwrap(), thread_pool);
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
        Command::ZAdd { key, score, name } => {
            if store.look_up(&key).is_none() {
                if let Some(val) = store.insert(key.clone(), Value::ZSet(ZSet::new())) {
                    drop_value(val, thread_pool);
                }
            }
            match store.look_up_mut(&key) {
                Some(Value::ZSet(zset)) => {
                    let added = zset.add(&name, score);
                    out_int(output_buffer, added as i64);
                }
                _ => out_error(output_buffer, ERR_UNKNOWN, b"expect zset"),
            }
        }
        Command::ZRem { key, name } => match store.look_up_mut(&key) {
            Some(Value::ZSet(zset)) => out_int(output_buffer, zset.remove(&name) as i64),
            Some(_) => out_error(output_buffer, ERR_UNKNOWN, b"expect zset"),
            None => out_int(output_buffer, 0),
        },
        Command::ZScore { key, name } => match store.look_up(&key) {
            Some(Value::ZSet(zset)) => match zset.score(&name) {
                Some(score) => out_dbl(output_buffer, score),
                None => out_nil(output_buffer),
            },
            Some(_) => out_error(output_buffer, ERR_UNKNOWN, b"expect zset"),
            None => out_nil(output_buffer),
        },
        Command::ZQuery {
            key,
            score,
            name,
            offset,
            limit,
        } => match store.look_up(&key) {
            Some(Value::ZSet(zset)) => {
                let keys = zset.query(score, &name, offset, limit);
                out_arr(output_buffer, (keys.len() * 2) as u32);
                for k in keys {
                    out_str(output_buffer, k.name().as_bytes());
                    out_dbl(output_buffer, k.score());
                }
            }
            Some(_) => out_error(output_buffer, ERR_UNKNOWN, b"expect zset"),
            None => out_arr(output_buffer, 0),
        },
        Command::Pexpire { key, dead_line } => {
            let node = store.look_up_mut_node(&key);
            match node {
                None => out_int(output_buffer, 0),
                Some(val) => {
                    if !val.deadline.is_none() {
                        val.deadline = Some(dead_line);
                    } else {
                        val.deadline = Some(dead_line);
                    }
                    let entry = TTLEntry::new(key, dead_line);
                    ttl_entries.insert(entry);
                    out_int(output_buffer, 1);
                }
            }
        }
        Command::Pttl { key } => {
            let node = store.look_up_mut_node(&key);
            match node {
                None => out_int(output_buffer, -2),
                Some(val) => match val.deadline {
                    None => out_int(output_buffer, -1),
                    Some(val) => {
                        let ms_left =
                            val.saturating_duration_since(Instant::now()).as_millis() as i64;
                        out_int(output_buffer, ms_left);
                    }
                },
            }
        }
    }
}

fn execute_set(store: &mut hash::HMap, key: Vec<u8>, val: Value) -> Option<Value> {
    store.insert(key, val)
}

pub fn execute_get(store: &mut hash::HMap, key: Vec<u8>) -> Option<&Value> {
    store.look_up(&key)
}

pub fn execute_del(store: &mut hash::HMap, key: Vec<u8>) -> Option<Value> {
    store.remove(&key)
}

fn invalid(msg: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, msg.to_string())
}

fn parse_name(bytes: Vec<u8>) -> std::io::Result<String> {
    String::from_utf8(bytes).map_err(|_| invalid("name is not valid utf-8"))
}

fn parse_score(bytes: &[u8]) -> std::io::Result<f64> {
    let score: f64 = std::str::from_utf8(bytes)
        .ok()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| invalid("score is not a number"))?;
    if score.is_nan() {
        return Err(invalid("score is NaN"));
    }
    Ok(score)
}

fn parse_u32(bytes: &[u8]) -> std::io::Result<u32> {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| invalid("expected a non-negative integer"))
}
