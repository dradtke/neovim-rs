use mpack::rpc::Client;
use std::io::{IoError, IoResult, Stream, LineBufferedWriter};
use std::io::net::ip::ToSocketAddr;
use std::io::net::tcp::TcpStream;
use std::io::pipe::PipeStream;
use std::io::process::{Command, Process};
use std::io::stdio;
use std::sync::Future;

#[cfg(unix)] use std::io::net::pipe::UnixStream;

/// Stores the means of communicating with Neovim.
enum SessionStream {
    Stream(Box<Stream + Send>),
    Stdio(stdio::StdReader, stdio::StdWriter),
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
pub struct Session<R, W>(Client<R, W>);

impl Session<TcpStream, TcpStream> {
    /// Connect to a Neovim instance over TCP.
    pub fn new_tcp<A: ToSocketAddr>(addr: A) -> IoResult<Session<TcpStream, TcpStream>> {
        let stream = try!(TcpStream::connect(addr));
        Ok(Session(Client::new_for_stream(stream)))
    }
}

#[cfg(unix)]
impl Session<UnixStream, UnixStream> {
    /// Connect to a Neovim instance over a Unix socket.
    pub fn new_socket<P: BytesContainer>(path: P) -> IoResult<Session<UnixStream, UnixStream>> {
        let stream = try!(UnixStream::connect(Path::new(path)));
        Ok(Session(Client::new_for_stream(stream)))
    }
}

// TODO: use the unbuffered versions instead, if possible
// or maybe just flush after every call
impl Session<stdio::StdinReader, LineBufferedWriter<stdio::StdWriter>> {
    /// Create a new session using standard input/output.
    pub fn new_stdio() -> Session<stdio::StdinReader, LineBufferedWriter<stdio::StdWriter>> {
        Session(Client::new(stdio::stdin(), stdio::stdout()))
    }
}

impl Session<PipeStream, PipeStream> {
    /// Create a new session using a new instance of Neovim.
    pub fn new_child(args: &[String]) -> Result<Session<PipeStream, PipeStream>, ChildSessionError> {
        let mut cmd = Command::new("nvim");
        cmd.args(args);
        let p = match cmd.spawn() {
            Ok(p) => p,
            Err(e) => return Err(ChildSessionError::Io(e)),
        };
        // Make sure that standard input/output are available.
        let procin = match p.stdin {
            Some(ref procin) => procin.clone(),
            None => return Err(ChildSessionError::NoStdin),
        };
        let procout = match p.stdout {
            Some(ref procout) => procout.clone(),
            None => return Err(ChildSessionError::NoStdin),
        };
        Ok(Session(Client::new(procin, procout)))
    }
}

impl<R, W> Session<R, W> where R: Reader + Send, W: Writer + Send {
    pub fn get_vim_api_info(&self) -> Future<Result<Metadata, ()>> {
    }
}
