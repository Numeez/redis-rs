pub const POLLIN: i16 = 0x1;
pub const POLLOUT: i16 = 0x4;
pub const POLLERR: i16 = 0x8;
pub const POLLHUP: i16 = 0x10;
pub const POLLNVAL: i16 = 0x20;

#[cfg(target_os = "macos")]
type NfdsT = u32;
#[cfg(target_os = "linux")]
type NfdsT = u64;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PollFd {
    pub fd: i32,
    pub events: i16,
    pub revents: i16,
}

unsafe extern "C" {
    fn poll(fds: *mut PollFd, nfds: NfdsT, timeout: i32) -> i32;
}

pub fn poll_fds(fds: &mut [PollFd], timeout: i32) -> std::io::Result<usize> {
    let result = unsafe { poll(fds.as_mut_ptr(), fds.len() as NfdsT, timeout) };
    if result == -1 {
        return Err(std::io::Error::last_os_error());
    }

    Ok(result as usize)
}
