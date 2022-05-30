// A basic debugging backend
// I stole this from the PPC backend for it's socket, so it's going to look similar

use ironic_core::bus::*;
use crate::back::*;

use std::env::temp_dir;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::thread;
use std::sync::{Arc, RwLock, mpsc::Sender};
use std::os::unix::net::{UnixStream, UnixListener};
use std::net::Shutdown;
use std::io::{Read, Write};
use std::convert::TryInto;

extern crate pretty_hex;
use pretty_hex::*;

macro_rules! extract_packet_and_reply {
    ($debug_recv: expr; $client: expr) => {
        let value = $debug_recv.recv().unwrap().value.unwrap().to_le();
        let bytes: [u8;4] = unsafe {std::mem::transmute(value)};
        $client.write(&bytes).unwrap();
    };
}

#[derive(Debug)]
pub struct DebugPacket {
    pub write: Option<bool>, // True if we are writing, false if we are reading
    pub addr: Option<u32>, // If we are operating on a memory address
    pub reg: Option<u32>,  // If we are operating on registers
    pub value: Option<u32>, //The value to write
    pub new_step: Option<u32>, // CPU/Bus steps
}

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

pub const IPC_SOCK: &str = "ironic-debug.sock";
pub const BUF_LEN: usize = 0x10000;

pub struct DebugBackend {
    // Thread IPC
    dbg_send: Sender<DebugPacket>,
    dbg_recv: Receiver<DebugPacket>,
    /// Input buffer for the socket.
    pub ibuf: [u8; BUF_LEN],
    /// Output buffer for the socket.
    pub obuf: [u8; BUF_LEN],
}
impl DebugBackend {
    pub fn new(dbg_send: Sender<DebugPacket>, dbg_recv: Receiver<DebugPacket>) -> Self {
        DebugBackend {
            dbg_send,
            dbg_recv,
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

    fn resolve_socket_path() -> PathBuf {
        let mut dir = temp_dir();
        dir.push(IPC_SOCK);
        dir
    }

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
        self.dbg_send.send(DebugPacket { write: Some(false), addr: None, reg: Some(req.addr), value: None, new_step: None }).expect("PeekReg send");
        extract_packet_and_reply!(self.dbg_recv; client);
    }

    fn handle_cmd_pokereg(&mut self, client: &mut UnixStream, req: SocketReq) {
        println!("[DEBUG] Command PokeReg");
        self.dbg_send.send(DebugPacket { write: Some(true), addr: None, reg: Some(req.addr), value: Some(req.len), new_step: None }).expect("PokeReg send");
        extract_packet_and_reply!(self.dbg_recv; client);
    }

    fn handle_cmd_peekaddr(&mut self, client: &mut UnixStream, req: SocketReq) {
        println!("[DEBUG] Command PeekAddr");
        self.dbg_send.send(DebugPacket { write: Some(false), addr: Some(req.addr), reg: None, value: None, new_step: None }).expect("PeekAddr send");
        extract_packet_and_reply!(self.dbg_recv; client);
    }

    fn handle_cmd_pokeaddr(&mut self, client: &mut UnixStream, req: SocketReq) {
        println!("[DEBUG] Command PokeAddr");
        self.dbg_send.send(DebugPacket { write: Some(true), addr: Some(req.addr), reg: None, value: Some(req.len), new_step: None }).expect("PokeAddr send");
        extract_packet_and_reply!(self.dbg_recv; client);
    }

    fn handle_cmd_step(&mut self, client: &mut UnixStream, req: SocketReq) {
        println!("[DEBUG] Command Step");
        todo!();
    }

}


impl Backend for DebugBackend {
    fn run(&mut self) {
        println!("[DEBUG] DEBUG backend thread started");

        // Try binding to the socket
        let res = std::fs::remove_file(DebugBackend::resolve_socket_path());
        match res {
            Ok(_) => {},
            Err(e) => {},
        }
        let res = UnixListener::bind(DebugBackend::resolve_socket_path());
        let sock = match res {
            Ok(sock) => Some(sock),
            Err(e) => {
                println!("[DEBUG] Couldn't bind to {},\n{:?}", DebugBackend::resolve_socket_path().to_string_lossy(), e);
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

