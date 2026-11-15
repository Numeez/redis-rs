mod avl;
mod buffer;
mod hash;
mod key;
mod operation;
mod poll;
mod protocol;
mod zset;

use std::{
    collections::{HashMap, VecDeque},
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    os::fd::{AsRawFd, RawFd},
    time::{Duration, Instant},
};

use poll::PollFd;

use crate::{
    buffer::{Buffer, response_end, response_start},
    hash::HMap,
    poll::{POLLERR, POLLHUP, POLLIN, POLLNVAL, POLLOUT, poll_fds},
};

const IDLE_TIMEOUT: Duration = Duration::from_secs(5);
struct Client {
    stream: TcpStream,
    output_buffer: Buffer,
    input_buffer: Buffer,
    last_active: Instant,
}

impl Client {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            output_buffer: Buffer::new(),
            input_buffer: Buffer::new(),
            last_active: Instant::now(),
        }
    }
    fn try_write(&mut self) -> bool {
        match self.stream.write(self.output_buffer.data()) {
            Ok(n) => {
                self.output_buffer.consume(n);
            }
            Err(e) => {
                if e.kind() != ErrorKind::WouldBlock && e.kind() != ErrorKind::Interrupted {
                    return true;
                }
            }
        }
        false
    }
}

fn main() -> std::io::Result<()> {
    let mut connection_map: HashMap<RawFd, Client> = HashMap::new();
    let mut idles: VecDeque<(Instant, RawFd)> = VecDeque::new();
    let mut store = hash::HMap::new(4);
    let listener = TcpListener::bind("127.0.0.1:6969")?;
    eprintln!("Server is listening on the  port: 6969");
    listener.set_nonblocking(true)?;
    let listener_fd = listener.as_raw_fd();
    loop {
        let mut fds: Vec<PollFd> = vec![PollFd {
            fd: listener_fd,
            events: POLLIN,
            revents: 0,
        }];

        for (key, client) in connection_map.iter() {
            let events: i16;
            if !client.output_buffer.is_empty() {
                events = POLLOUT;
            } else {
                events = POLLIN;
            }
            fds.push(PollFd {
                fd: *key,
                events,
                revents: 0,
            });
        }
        match poll_fds(&mut fds, next_timeout_ms(&idles)) {
            Ok(val) => eprintln!("The return value of poll: {val}"),
            Err(e) => {
                eprintln!("Err occured while  calling poll: {e}");
                continue;
            }
        }

        if fds[0].revents & POLLIN != 0 {
            accept_connection(&listener, &mut idles, &mut connection_map)
        }

        for entry in &fds[1..] {
            handle_events(entry, &mut idles, &mut connection_map, &mut store)
        }
        process_timers(&mut idles, &mut connection_map);
    }
}

fn accept_connection(
    listener: &TcpListener,
    idles: &mut VecDeque<(Instant, RawFd)>,
    connection_map: &mut HashMap<RawFd, Client>,
) {
    loop {
        match listener.accept() {
            Ok((stream, address)) => {
                eprintln!("Client with addr: {address} connected");
                if let Err(err) = stream.set_nonblocking(true) {
                    eprintln!("Error occured while setting non blocking stream: {err}");
                    continue;
                };
                let client = Client::new(stream);
                let client_fd = client.stream.as_raw_fd();
                idles.push_back((client.last_active, client_fd));
                connection_map.insert(client_fd, client);
            }
            Err(e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    eprintln!("Error occured: {e}");
                }
                break;
            }
        }
    }
}

fn handle_events(
    entry: &PollFd,
    idles: &mut VecDeque<(Instant, RawFd)>,
    connection_map: &mut HashMap<RawFd, Client>,
    store: &mut HMap,
) {
    if entry.revents & POLLIN != 0 {
        if let Some(client) = connection_map.get_mut(&entry.fd) {
            let mut buffer = [0u8; 1024];
            let mut close = false;
            client.last_active = Instant::now();
            idles.push_back((client.last_active, entry.fd));
            match client.stream.read(&mut buffer) {
                Ok(val) => {
                    if val == 0 {
                        connection_map.remove(&entry.fd);
                    } else {
                        client.input_buffer.append(&buffer[..val]);
                        loop {
                            if client.input_buffer.len() < 4 {
                                break;
                            }
                            let length = u32::from_le_bytes(
                                client.input_buffer.data()[..4].try_into().unwrap(),
                            ) as usize;
                            if length > buffer::MAX_SIZE {
                                close = true;
                                break;
                            }
                            if client.input_buffer.len() >= 4 + length {
                                handle_request(client, length, store);
                            } else {
                                break;
                            }
                        }
                        if !close && !client.output_buffer.is_empty() {
                            close = client.try_write();
                        }
                    }
                }
                Err(err) => {
                    if err.kind() != ErrorKind::WouldBlock && err.kind() != ErrorKind::Interrupted {
                        connection_map.remove(&entry.fd);
                    }
                }
            }
            if close {
                connection_map.remove(&entry.fd);
            }
        }
    }

    if entry.revents & POLLOUT != 0 {
        if let Some(client) = connection_map.get_mut(&entry.fd) {
            if client.try_write() {
                connection_map.remove(&entry.fd);
            }
        }
    }

    if entry.revents & (POLLERR | POLLHUP | POLLNVAL) != 0 {
        connection_map.remove(&entry.fd);
    }
}

fn parse_client_request(mut request_buffer: &[u8]) -> std::io::Result<Vec<Vec<u8>>> {
    let mut result: Vec<Vec<u8>> = vec![];
    let mut buffer = [0u8; 4];
    request_buffer.read_exact(&mut buffer)?;
    let nstr = u32::from_le_bytes(buffer) as usize;
    if nstr > 200_000 {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "nstr value is too big should be less than 200_000",
        ));
    }
    for _ in 0..nstr {
        request_buffer.read_exact(&mut buffer)?;
        let length = u32::from_le_bytes(buffer) as usize;
        if length > request_buffer.len() {
            return Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "string length exceeds payload",
            ));
        }
        let mut buf = vec![0u8; length];
        request_buffer.read_exact(&mut buf)?;
        result.push(buf);
    }
    if !request_buffer.is_empty() {
        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "buffer is not empty nstr is wrong",
        ));
    }

    Ok(result)
}

fn handle_request(client: &mut Client, length: usize, store: &mut HMap) {
    let header = response_start(&mut client.output_buffer);
    process_request(
        &client.input_buffer.data()[4..4 + length],
        &mut client.output_buffer,
        store,
    );
    response_end(&mut client.output_buffer, header);
    client.input_buffer.consume(4 + length);
}

fn process_request(payload: &[u8], out: &mut Buffer, store: &mut HMap) {
    let args = match parse_client_request(payload) {
        Ok(args) => args,
        Err(e) => {
            protocol::out_error(out, protocol::ERR_UNKNOWN, e.to_string().as_bytes());
            return;
        }
    };
    let cmd = match operation::parse_command(args) {
        Ok(cmd) => cmd,
        Err(e) => {
            protocol::out_error(out, protocol::ERR_UNKNOWN, e.to_string().as_bytes());
            return;
        }
    };
    operation::execute(out, cmd, store);
}

fn next_timeout_ms(idles: &VecDeque<(Instant, RawFd)>) -> i32 {
    match idles.front() {
        None => -1,
        Some(&(t, _)) => {
            let deadline = t + IDLE_TIMEOUT;
            let now = Instant::now();
            if deadline <= now {
                0
            } else {
                (deadline - now).as_millis() as i32 + 1
            }
        }
    }
}

fn process_timers(
    idles: &mut VecDeque<(Instant, RawFd)>,
    connection_map: &mut HashMap<RawFd, Client>,
) {
    let now = Instant::now();
    while let Some(&(t, fd)) = idles.front() {
        if t + IDLE_TIMEOUT > now {
            break;
        }
        idles.pop_front();
        let still_idle = connection_map
            .get(&fd)
            .map_or(false, |c| c.last_active == t);
        if still_idle {
            eprintln!("closing idle connection {fd}");
            connection_map.remove(&fd);
        }
    }
}
