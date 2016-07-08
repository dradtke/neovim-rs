use mpack::{Value, WriteError};
use mpack::rpc::{Client, RpcResult};

use std::env;
use std::error::Error;
use std::io::{self, Read, Stdin, Stdout, Write};
use std::net::{TcpStream, ToSocketAddrs, SocketAddr};
use std::process::{Command, Child, ChildStdin, ChildStdout, Stdio};
use std::sync::mpsc::Receiver;
use super::metadata::Metadata;

/// An active Neovim session.
pub struct Session {
    pub metadata: Metadata,
    conn: ClientConn,
}

impl Session {
    /// Connect to a Neovim instance over TCP.
    pub fn new_tcp<A: ToSocketAddrs>(addr: A) -> io::Result<Session> {
        let reader = try!(TcpStream::connect(&addr));
        let writer = try!(reader.try_clone());
        let addr = reader.peer_addr().unwrap();
        let mut client = Client::new(reader, writer);
        Ok(Session{
            metadata: try!(Session::get_vim_api_info(&mut client)),
            conn: ClientConn::Tcp(client, addr),
        })
    }

    /// Connect to a Neovim instance using this process' standard input
    /// and output. Useful if Neovim started this process.
    pub fn new_stdio() -> Session {
        let mut client = Client::new(io::stdin(), io::stdout());
        Session{
            metadata: Session::get_vim_api_info(&mut client).unwrap(),
            conn: ClientConn::Stdio(client),
        }
    }

    /// Connect to a Neovim instance by spawning a new one. Automatically passes `--embed`
    /// as a command-line parameter.
    ///
    /// Uses `nvim` as the default command for launching Neovim, but this can be overridden
    /// with the `NVIM_BIN` environment variable.
    pub fn new_child(args: &[String]) -> io::Result<Session> {
        let cmd = env::var("NVIM_BIN").unwrap_or(String::from("nvim"));
        let mut child = try!(Command::new(cmd).args(args).arg("--embed").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn());
        let mut client = Client::new(child.stdout.take().unwrap(), child.stdin.take().unwrap());
        Ok(Session{
            metadata: try!(Session::get_vim_api_info(&mut client)),
            conn: ClientConn::Child(client, child),
        })
    }

    /// Connect to a Neovim instance over a Unix socket. Currently unimplemented.
    pub fn new_socket() {
        unimplemented!()
    }

    /// Call a method over RPC.
    pub fn call(&mut self, method: String, params: Vec<Value>) -> Result<Receiver<RpcResult>, WriteError> {
        match self.conn {
            ClientConn::Tcp(ref mut client, _) => client.call(method, params),
            ClientConn::Stdio(ref mut client) => client.call(method, params),
            ClientConn::Child(ref mut client, _) => client.call(method, params),
        }
    }

    /// Call a method over RPC, synchronously.
    pub fn call_sync(&mut self, method: String, params: Vec<Value>) -> Result<RpcResult, WriteError> {
        match self.conn {
            ClientConn::Tcp(ref mut client, _) => client.call_sync(method, params),
            ClientConn::Stdio(ref mut client) => client.call_sync(method, params),
            ClientConn::Child(ref mut client, _) => client.call_sync(method, params),
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

    fn get_vim_api_info<R: Read + Send + 'static, W: Write + Send>(client: &mut Client<R, W>) -> io::Result<Metadata> {
        let api_info = match client.call_sync(String::from("vim_get_api_info"), vec![]) {
            Ok(result) => match result {
                Ok(api_info) => api_info,
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, "call to vim_get_api_info failed")),
            },
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e.description())),
        };
        Ok(Metadata::new(api_info.array().unwrap().get(1).unwrap().clone()).unwrap())
    }
}

enum ClientConn {
    Tcp(Client<TcpStream, TcpStream>, SocketAddr),
    Stdio(Client<Stdin, Stdout>),
    Child(Client<ChildStdout, ChildStdin>, Child),
}
