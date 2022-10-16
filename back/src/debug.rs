// A basic debugging backend
// I stole this from the PPC backend for it's socket, so it's going to look similar

use crate::back::*;

use std::env::temp_dir;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::net::Shutdown;
use std::io::{Read, Write};
use std::convert::TryInto;

#[cfg(target_family = "unix")]
use std::os::unix::net::{UnixStream, UnixListener};
#[cfg(target_family = "windows")]
use uds_windows::{UnixStream, UnixListener};

macro_rules! extract_packet_and_reply {
    ($debug_recv: expr; $client: expr) => {
        let reply: DebugPacket = $debug_recv.recv().map_err(|e| e.to_string())?;
        assert!(reply.command == DebugCommand::Reply);
        let value = reply.op1.to_le();
        let bytes: [u8;4] = unsafe {std::mem::transmute(value)};

        $client.write(&bytes).map_err(|e| e.to_string())?;
        return Ok(());
    };
}

#[derive(Debug, PartialEq)]
pub struct DebugPacket {
    pub command: DebugCommand,
    pub op1:u32,
    pub op2:u32,
}

/// A type of command sent over the socket.
#[derive(Debug, PartialEq)]
#[repr(u32)]
pub enum DebugCommand {
    PeekReg,
    PokeReg,
    PeekPAddr,
    PokePAddr,
    Step,
    Reply,
    Unimpl,
}
impl DebugCommand {
    fn from_u32(x: u32) -> Self {
        match x {
            1 => Self::PeekReg,
            2 => Self::PokeReg,
            3 => Self::PeekPAddr,
            4 => Self::PokePAddr,
            5 => Self::Step,
            6 => Self::Reply,
            _ => Self::Unimpl,
        }
    }
}

/// A request packet from the socket.
#[repr(C)]
pub struct SocketReq {
    pub cmd: DebugCommand,
    pub addr: u32,
    pub len: u32,
}
impl SocketReq {
    pub fn from_buf(s: &[u8; 0xc]) -> Self {
        let cmd = DebugCommand::from_u32(
            u32::from_le_bytes(s[0..4].try_into().unwrap())
        );
        assert_ne!(cmd, DebugCommand::Reply);
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
    pub fn server_loop(&mut self, sock: UnixListener) -> Result<(), String> {
        loop {
            let (mut client, _) = sock.accept().map_err(|e| e.to_string())?;

            loop {
                println!("[DEBUG] waiting for command ...");

                let res = self.wait_for_request(&mut client);
                let req = if res.is_none() { break; } else { res.unwrap() };
                match req.cmd {
                    DebugCommand::PeekReg  => self.handle_cmd_peekreg(&mut client, req).map_err(|e| e.to_string())?,
                    DebugCommand::PokeReg  => self.handle_cmd_pokereg(&mut client, req).map_err(|e| e.to_string())?,
                    DebugCommand::PeekPAddr => self.handle_cmd_peekaddr(&mut client, req).map_err(|e| e.to_string())?,
                    DebugCommand::PokePAddr => self.handle_cmd_pokeaddr(&mut client, req).map_err(|e| e.to_string())?,
                    DebugCommand::Step     => self.handle_cmd_step(&mut client, req).map_err(|e| e.to_string())?,
                    DebugCommand::Reply    => { return Err(format!("Unsupported")); },
                    DebugCommand::Unimpl => break,
                }
            }
            return client.shutdown(Shutdown::Both).map_err(|e| e.to_string());
        };
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

    fn handle_cmd_peekreg(&mut self, client: &mut UnixStream, req: SocketReq) -> Result<(), String> {
        println!("[DEBUG] Command PeekReg");
        self.dbg_send.send(DebugPacket { command: DebugCommand::PeekReg, op1: req.addr, op2: 0 }).map_err(|e| e.to_string())?;
        extract_packet_and_reply!(self.dbg_recv; client);
    }

    fn handle_cmd_pokereg(&mut self, client: &mut UnixStream, req: SocketReq) -> Result<(), String> {
        println!("[DEBUG] Command PokeReg");
        self.dbg_send.send(DebugPacket { command: DebugCommand::PeekReg, op1: req.addr, op2: req.len }).map_err(|e| e.to_string())?;
        extract_packet_and_reply!(self.dbg_recv; client);
    }

    fn handle_cmd_peekaddr(&mut self, client: &mut UnixStream, req: SocketReq) -> Result<(), String> {
        println!("[DEBUG] Command PeekAddr");
        self.dbg_send.send(DebugPacket { command: DebugCommand::PeekPAddr, op1: req.addr, op2: 0 }).map_err(|e| e.to_string())?;
        extract_packet_and_reply!(self.dbg_recv; client);
    }

    fn handle_cmd_pokeaddr(&mut self, client: &mut UnixStream, req: SocketReq) -> Result<(), String> {
        println!("[DEBUG] Command PokeAddr");
        self.dbg_send.send(DebugPacket { command: DebugCommand::PokePAddr, op1: req.addr, op2: req.len }).map_err(|e| e.to_string())?;
        extract_packet_and_reply!(self.dbg_recv; client);
    }

    fn handle_cmd_step(&mut self, client: &mut UnixStream, req: SocketReq) -> Result<(), String> {
        println!("[DEBUG] Command Step");
        self.dbg_send.send(DebugPacket { command: DebugCommand::Step, op1: req.addr, op2: 0 }).map_err(|e| e.to_string())?;
        extract_packet_and_reply!(self.dbg_recv; client);
    }

}


impl Backend for DebugBackend {
    fn run(&mut self) -> Result<(), String> {
        println!("[DEBUG] DEBUG backend thread started");
        let path = DebugBackend::resolve_socket_path();
        // Try binding to the socket
        let _ = std::fs::remove_file(&path);

        let res = UnixListener::bind(&path);
        let sock = match res {
            Ok(sock) => Some(sock),
            Err(e) => {
                println!("[DEBUG] Couldn't bind to {},\n{:?}", &path.to_string_lossy(), e);
                None
            }
        };

        // If we successfully bind, run the server until it exits
        if let Some(sock) = sock {
            self.server_loop(sock)?;
        }
        println!("[DEBUG] thread exited");
        Ok(())
    }
}

