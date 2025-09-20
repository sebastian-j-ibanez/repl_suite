// Copyright (c) 2025 Sebastian Ibanez
// Author: Sebastian Ibanez
// Created: 2025-09-14

use std::{
    fmt::Display,
    io::{self, Read, Stdin, Stdout, Write},
    os::fd::{AsRawFd, RawFd},
    u8,
};

/// Error type for IO and UNIX errors.
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Errno(nix::errno::Errno),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<nix::errno::Errno> for Error {
    fn from(e: nix::errno::Errno) -> Self {
        Error::Errno(e)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Errno(e) => write!(f, "UNIX error: {}", e),
        }
    }
}

/// Manipulates terminal state via libc.
pub struct TermManager {
    stdin: Stdin,
    stdout: Stdout,
    fd: RawFd,
    original_termios: libc::termios,
}

impl TermManager {
    /// Create a new TermManager in raw mode.
    pub fn new() -> Result<TermManager, Error> {
        let stdin = io::stdin();
        let stdout = io::stdout();
        let fd = stdin.as_raw_fd();
        let original_termios = enable_raw_mode(fd)?;

        Ok(TermManager {
            stdin,
            stdout,
            fd,
            original_termios,
        })
    }

    pub fn get_stdin(&self) -> &Stdin {
        &self.stdin
    }

    pub fn get_stout(&self) -> &Stdout {
        &self.stdout
    }

    /// Flush stdout.
    pub fn flush(&mut self) -> Result<(), Error> {
        match self.stdout.flush() {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::Io(e)),
        }
    }

    /// Write buffer to stdout.
    pub fn write(&mut self, data: &[u8]) -> Result<(), Error> {
        match self.stdout.write(data) {
            Ok(r) => {
                if r <= 0 && data.len() <= 0 {
                    let msg = format!("only wrote {} of {} bytes", r, data.len());
                    return Err(Error::Io(io::Error::new(io::ErrorKind::WriteZero, msg)));
                }

                Ok(())
            }
            Err(e) => Err(Error::Io(e)),
        }
    }

    /// Read byte from stdin. Return io::ErrorKind::WriteZero if no byte read.
    pub fn read(&mut self, buf: &mut [u8; 1]) -> Result<usize, Error> {
        match self.stdin.read(buf) {
            Ok(0) => {
                return Err(Error::Io(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "read 0 bytes from stdin",
                )));
            }
            Ok(bytes_read) => Ok(bytes_read),
            Err(e) => Err(Error::Io(e)),
        }
    }
}

impl Drop for TermManager {
    fn drop(&mut self) {
        disable_raw_mode(self.fd, self.original_termios).unwrap();
    }
}

/// Enable raw mode by disabling canonical mode and echo.
fn enable_raw_mode(fd: RawFd) -> Result<libc::termios, Error> {
    let original_termios = get_termios(fd)?;
    let mut raw = original_termios;
    raw.c_lflag &= !(libc::ICANON | libc::ECHO);
    raw.c_cc[libc::VMIN] = 1;
    raw.c_cc[libc::VTIME] = 0;
    set_termios(fd, &raw)?;
    Ok(original_termios)
}

/// Disable raw mode and reset terminal interface.
fn disable_raw_mode(fd: RawFd, original_termios: libc::termios) -> Result<(), Error> {
    set_termios(fd, &original_termios)?;
    Ok(())
}

/// Get termios from raw file descriptors.
fn get_termios(fd: RawFd) -> Result<libc::termios, Error> {
    let mut termios = std::mem::MaybeUninit::uninit();
    let res = unsafe { libc::tcgetattr(fd, termios.as_mut_ptr()) };
    if res != 0 {
        return Err(Error::Io(io::Error::last_os_error()));
    }
    Ok(unsafe { termios.assume_init() })
}

/// Set termios settings from raw file descriptors.
fn set_termios(fd: RawFd, termios: &libc::termios) -> Result<(), Error> {
    let res = unsafe { libc::tcsetattr(fd, libc::TCSANOW, termios) };
    if res != 0 {
        return Err(Error::Io(io::Error::last_os_error()));
    }
    Ok(())
}
