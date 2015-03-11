use mpack::{Value, WriteError};
use mpack::rpc::{Client, RpcResult};

use std::io::{self, Stdin, Stdout};
use std::net::{TcpStream, ToSocketAddrs, SocketAddr};
use std::process::{Command, Child, ChildStdin, ChildStdout};
use std::sync::Future;
use super::metadata::Metadata;

/// Represents an error that occurred when trying to start
/// a child Neovim session.
pub enum ChildSessionError {
    Io(io::Error),
    NoStdin,
    NoStdout,
}

/// An active Neovim session.
pub struct Session<'c> {
    conn: ClientConn<'c>,
}

impl<'c> Session<'c> {
    /// Connect to a Neovim instance over TCP.
    pub fn new_tcp<A: ToSocketAddrs>(addr: A) -> io::Result<Session<'c>> {
        let reader = try!(TcpStream::connect(&addr));
        let writer = try!(reader.try_clone());
        let addr = try!(reader.socket_addr());
        Ok(Session{
            conn: ClientConn::Tcp(Client::new(reader, writer), addr),
        })
    }

    /// Connect to a Neovim instance using this process' standard input
    /// and output. Useful if Neovim started this process.
    pub fn new_stdio() -> Session<'c> {
        Session{
            conn: ClientConn::Stdio((Client::new(io::stdin(), io::stdout()))),
        }
    }

    /// Connect to a Neovim instance by spawning a new one.
    pub fn new_child(args: &[String]) -> Result<Session<'c>, ChildSessionError> {
        let mut child = match Command::new("nvim").args(args).spawn() {
            Ok(child) => child,
            Err(e) => return Err(ChildSessionError::Io(e)),
        };
        if child.stdout.is_none() {
            return Err(ChildSessionError::NoStdout);
        }
        if child.stdin.is_none() {
            return Err(ChildSessionError::NoStdin);
        }
        Ok(Session{
            conn: ClientConn::Child(Client::new(child.stdout.take().unwrap(), child.stdin.take().unwrap()), child),
        })
    }

    /// Connect to a Neovim instance over a Unix socket. Currently unimplemented.
    pub fn new_socket() {
        unimplemented!()
    }

    /// Call a method over RPC.
    pub fn call(&mut self, method: String, params: Vec<Value>) -> Result<Future<RpcResult>, WriteError> {
        match self.conn {
            ClientConn::Tcp(ref mut client, _) => client.call(method, params),
            ClientConn::Stdio(ref mut client) => client.call(method, params),
            ClientConn::Child(ref mut client, _) => client.call(method, params),
        }
    }

    /// Returns a reference to the TCP socket address used by this session.
    ///
    /// If the connection isn't over TCP, this method returns None.
    pub fn socket_addr(&self) -> Option<&SocketAddr> {
        match self.conn {
            ClientConn::Tcp(_, ref addr) => Some(addr),
            ClientConn::Stdio(..) | ClientConn::Child(..) => None,
        }
    }

    pub fn get_vim_api_info(&self) -> Future<Result<Metadata, ()>> {
        unimplemented!()
    }
}

enum ClientConn<'c> {
    Tcp(Client<'c, TcpStream, TcpStream>, SocketAddr),
    Stdio(Client<'c, Stdin, Stdout>),
    Child(Client<'c, ChildStdout, ChildStdin>, Child),
}
