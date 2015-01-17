#![allow(dead_code, unused_variables, unstable)]

use std::io::{IoError, IoResult, Stream};
use std::io::net::ip::ToSocketAddr;
use std::io::net::tcp::TcpStream;
use std::io::process::{Command, Process};
use std::io::stdio::{StdReader, StdWriter, stdin_raw, stdout_raw};
use std::path::BytesContainer;

/// Stores the means of communicating with Neovim.
enum SessionStream {
    Stream(Box<Stream + Send>),
    Stdio(StdReader, StdWriter),
    Child(Process),
}

/// Represents an error that occurred when trying to start
/// a child Neovim session.
pub enum ChildSessionError {
    Io(IoError),
    NoStdin,
    NoStdout,
}

/// An active Neovim session.
pub struct Session {
    stream: SessionStream,
}

impl Session {
    /// Connect to a Neovim instance over TCP.
    pub fn new_tcp<A: ToSocketAddr>(addr: A) -> IoResult<Session> {
        let stream = try!(TcpStream::connect(addr));
        Ok(Session {
            stream: SessionStream::Stream(Box::new(stream)),
        })
    }

    /// Connect to a Neovim instance over a Unix socket.
    pub fn new_socket<P: BytesContainer>(path: P) -> IoResult<Session> {
        let stream = try!(std::io::net::pipe::UnixStream::connect(Path::new(path)));
        Ok(Session {
            stream: SessionStream::Stream(Box::new(stream)),
        })
    }

    /// Create a new session using standard input/output.
    pub fn new_stdio() -> Session {
        Session {
            stream: SessionStream::Stdio(stdin_raw(), stdout_raw()),
        }
    }

    /// Create a new session using a new instance of Neovim.
    pub fn new_child(args: &[String]) -> Result<Session, ChildSessionError> {
        let mut cmd = Command::new("neovim");
        cmd.args(args);
        let p = match cmd.spawn() {
            Ok(p) => p,
            Err(e) => return Err(ChildSessionError::Io(e)),
        };
        // Make sure that standard input/output are available.
        if p.stdin.is_none() { return Err(ChildSessionError::NoStdin); }
        if p.stdout.is_none() { return Err(ChildSessionError::NoStdout); }
        Ok(Session {
            stream: SessionStream::Child(p),
        })
    }
}

impl Reader for Session {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        match self.stream {
            SessionStream::Stream(ref mut s) => s.read(buf),
            SessionStream::Stdio(ref mut r, _) => r.read(buf),
            SessionStream::Child(ref mut p) => p.stdin.as_mut().unwrap().read(buf),
        }
    }
}

impl Writer for Session {
    fn write(&mut self, buf: &[u8]) -> IoResult<()> {
        match self.stream {
            SessionStream::Stream(ref mut s) => s.write(buf),
            SessionStream::Stdio(_, ref mut w) => w.write(buf),
            SessionStream::Child(ref mut p) => p.stdout.as_mut().unwrap().write(buf),
        }
    }
}
