// A basic debugging backend
// I stole this from the PPC backend for it's socket, so it's going to look similar

use ironic_core::bus::*;
use crate::back::*;

use std::thread;
use std::sync::{Arc, RwLock};
use std::os::unix::net::{UnixStream, UnixListener};
use std::net::Shutdown;
use std::io::{Read, Write};
use std::convert::TryInto;

extern crate pretty_hex;
use pretty_hex::*;

/// A type of command sent over the socket.
#[derive(Debug)]
#[repr(u32)]
pub enum Command { 
    PeekReg, 
    PokeReg, 
    PeekAddr, 
    PokeAddr, 
    Step,
    Unimpl 
}
impl Command {
    fn from_u32(x: u32) -> Self {
        match x {
            1 => Self::PeekReg,
            2 => Self::PokeReg,
            3 => Self::PeekAddr,
            4 => Self::PokeAddr,
            5 => Self::Step,
            _ => Self::Unimpl,
        }
    }
}

/// A request packet from the socket.
#[repr(C)]
pub struct SocketReq {
    pub cmd: Command,
    pub addr: u32,
    pub len: u32,
}
impl SocketReq {
    pub fn from_buf(s: &[u8; 0xc]) -> Self {
        let cmd = Command::from_u32(
            u32::from_le_bytes(s[0..4].try_into().unwrap())
        );
        let addr = u32::from_le_bytes(s[0x4..0x8].try_into().unwrap());
        let len = u32::from_le_bytes(s[0x8..0xc].try_into().unwrap());
        SocketReq { cmd, addr, len }
    }
}

pub const IPC_SOCK: &str = "/tmp/ironic-debug.sock";
pub const BUF_LEN: usize = 0x10000;

pub struct DebugBackend {
    /// Reference to the system bus.
    pub bus: Arc<RwLock<Bus>>,
    /// Input buffer for the socket.
    pub ibuf: [u8; BUF_LEN],
    /// Output buffer for the socket.
    pub obuf: [u8; BUF_LEN],
}
impl DebugBackend {
    pub fn new(bus: Arc<RwLock<Bus>>) -> Self {
        DebugBackend {
            bus,
            ibuf: [0; BUF_LEN],
            obuf: [0; BUF_LEN],
        }
    }

    fn recv(&mut self, client: &mut UnixStream) -> Option<usize> {
        let res = client.read(&mut self.ibuf);
        match res {
            Ok(len) => if len == 0 { None } else { Some(len) },
            Err(_) => None
        }
    }
}


impl DebugBackend {

    /// Handle clients connected to the socket.
    pub fn server_loop(&mut self, sock: UnixListener) {
        loop {
            let res = sock.accept();
            let mut client = match res {
                Ok((stream, _)) => stream,
                Err(e) => { 
                    println!("[DEBUG] accept() error {:?}", e);
                    break;
                }
            };

            'handle_client: loop {
                println!("[DEBUG] waiting for command ...");

                let res = self.wait_for_request(&mut client);
                let req = if res.is_none() { break; } else { res.unwrap() };
                match req.cmd {
                    Command::PeekReg  => self.handle_cmd_peekreg(&mut client, req),
                    Command::PokeReg  => self.handle_cmd_pokereg(&mut client, req),
                    Command::PeekAddr => self.handle_cmd_peekaddr(&mut client, req),
                    Command::PokeAddr => self.handle_cmd_pokeaddr(&mut client, req),
                    Command::Step     => self.handle_cmd_step(&mut client, req),
                    Command::Unimpl => break,
                }
            }
            client.shutdown(Shutdown::Both).unwrap();
        }
    }

    /// Block until we receive some command message from a client.
    fn wait_for_request(&mut self, client: &mut UnixStream) -> Option<SocketReq> {
        let res = self.recv(client);
        if res.is_none() {
            return None;
        }
        let req = SocketReq::from_buf(
            &self.ibuf[0..0xc].try_into().unwrap()
        );
        if req.len as usize > BUF_LEN - 0xc {
            return None;
        }
        Some(req)
    }

    fn handle_cmd_peekreg(&mut self, client: &mut UnixStream, req: SocketReq) {
        println!("[DEBUG] Command PeekReg");
    }

    fn handle_cmd_pokereg(&mut self, client: &mut UnixStream, req: SocketReq) {
        println!("[DEBUG] Command PeekReg");
    }

    fn handle_cmd_peekaddr(&mut self, client: &mut UnixStream, req: SocketReq) {
        println!("[DEBUG] Command PeekReg");
    }

    fn handle_cmd_pokeaddr(&mut self, client: &mut UnixStream, req: SocketReq) {
        println!("[DEBUG] Command PeekReg");
    }

    fn handle_cmd_step(&mut self, client: &mut UnixStream, req: SocketReq) {
        println!("[DEBUG] Command PeekReg");
    }

}


impl Backend for DebugBackend {
    fn run(&mut self) {
        println!("[DEBUG] DEBUG backend thread started");

        // Try binding to the socket
        let res = std::fs::remove_file(IPC_SOCK);
        match res {
            Ok(_) => {},
            Err(e) => {},
        }
        let res = UnixListener::bind(IPC_SOCK);
        let sock = match res {
            Ok(sock) => Some(sock),
            Err(e) => {
                println!("[DEBUG] Couldn't bind to {},\n{:?}", IPC_SOCK, e);
                None
            }
        };

        // If we successfully bind, run the server until it exits
        if sock.is_some() {
            self.server_loop(sock.unwrap());
        }
        println!("[DEBUG] thread exited");
    }
}

