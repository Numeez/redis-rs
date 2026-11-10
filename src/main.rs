mod poll;

use std::{
    collections::HashMap,
    io::{ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    os::fd::{AsRawFd, RawFd},
};

use poll::PollFd;

use crate::poll::{POLLERR, POLLHUP, POLLIN, POLLNVAL, POLLOUT, poll_fds};

struct Client {
    stream: TcpStream,
    output_buffer: Vec<u8>,
    input_buffer: Vec<u8>,
}

impl Client {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            output_buffer: vec![],
            input_buffer: vec![],
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut connection_map: HashMap<RawFd, Client> = HashMap::new();
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
            let mut events = POLLIN;
            if !client.output_buffer.is_empty() {
                events |= POLLOUT;
            }
            fds.push(PollFd {
                fd: *key,
                events,
                revents: 0,
            });
        }
        match poll_fds(&mut fds, -1) {
            Ok(val) => eprintln!("The return value of poll: {val}"),
            Err(e) => {
                eprintln!("Err occured while  calling poll: {e}");
                continue;
            }
        }

        if fds[0].revents & POLLIN != 0 {
            loop {
                match listener.accept() {
                    Ok((stream, address)) => {
                        eprintln!("Client with addr: {address} connected");
                        if let Err(err) = stream.set_nonblocking(true) {
                            eprintln!("Error occured while setting non blocking stream: {err}");
                            continue;
                        };
                        let client_fd = stream.as_raw_fd();
                        connection_map.insert(client_fd, Client::new(stream));
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

        for entry in &fds[1..] {
            if entry.revents & POLLIN != 0 {
                if let Some(client) = connection_map.get_mut(&entry.fd) {
                    let mut buffer = [0u8; 1024];
                    match client.stream.read(&mut buffer) {
                        Ok(val) => {
                            if val == 0 {
                                connection_map.remove(&entry.fd);
                            } else {
                                client.input_buffer.extend_from_slice(&buffer[..val]);
                                while let Some(i) =
                                    client.input_buffer.iter().position(|&b| b == b'\n')
                                {
                                    let read_input: Vec<u8> =
                                        client.input_buffer.drain(..=i).collect();
                                    client.output_buffer.extend_from_slice(&read_input);
                                }
                            }
                        }
                        Err(err) => {
                            if err.kind() != ErrorKind::WouldBlock
                                && err.kind() != ErrorKind::Interrupted
                            {
                                connection_map.remove(&entry.fd);
                            }
                        }
                    }
                }
            }

            if entry.revents & POLLOUT != 0 {
                if let Some(client) = connection_map.get_mut(&entry.fd) {
                    match client.stream.write(&client.output_buffer) {
                        Ok(n) => {
                            client.output_buffer.drain(..n);
                        }
                        Err(e) => {
                            if e.kind() != ErrorKind::WouldBlock
                                && e.kind() != ErrorKind::Interrupted
                            {
                                connection_map.remove(&entry.fd);
                            }
                        }
                    }
                }
            }

            if entry.revents & (POLLERR | POLLHUP | POLLNVAL) != 0 {
                connection_map.remove(&entry.fd);
            }
        }
    }
}
