use rustix::termios::{isatty, tcgetwinsize};
use std::os::fd::{AsRawFd, BorrowedFd};
use std::os::unix::io::RawFd;

pub fn terminal_size() -> Option<(u16, u16)> {
    if let Some(size) = terminal_size_using_pd(std::io::stdout().as_raw_fd()) {
        return Some(size);
    } else if let Some(size) = terminal_size_using_pd(std::io::stderr().as_raw_fd()) {
        return Some(size);
    } else if let Some(size) = terminal_size_using_pd(std::io::stdin().as_raw_fd()) {
        return Some(size);
    }

    None
}
fn terminal_size_using_pd(fd: RawFd) -> Option<(u16, u16)> {
    let fd = unsafe { BorrowedFd::borrow_raw(fd) };

    if !isatty(fd) {
        return None;
    }

    let winsize = tcgetwinsize(fd).ok()?;

    let rows = winsize.ws_row as u16;
    let cols = winsize.ws_col as u16;

    if rows > 0 && cols > 0 {
        Some((rows, cols))
    } else {
        None
    }
}


