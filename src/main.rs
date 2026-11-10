mod poll;


use std::{
    collections::HashMap,
    io::{ErrorKind},
    net::{TcpListener, TcpStream},
    os::fd::{AsRawFd, RawFd},
};

fn main() -> std::io::Result<()> {
    let mut connection_map: HashMap<RawFd, TcpStream> = HashMap::new();
    let listener = TcpListener::bind("127.0.0.1:6969")?;
    eprintln!("Server is listening on the  port: 6969");
    listener.set_nonblocking(true)?;
    let _ = listener.as_raw_fd();
    loop {
        match listener.accept() {
            Ok((stream, address)) => {
                eprintln!("Client with addr: {address} connected");
                stream.set_nonblocking(true).unwrap_or_else(|err| {
                    eprintln!("Error occured while setting non blocking stream: {err}")
                });
                let client_fd = stream.as_raw_fd();
                connection_map.insert(client_fd, stream);
            }
            Err(e) => {
                if e.kind() != ErrorKind::WouldBlock {
                    eprintln!("Error occured: {e}");
                }
                break;
            }
        }
    }

    Ok(())
}
