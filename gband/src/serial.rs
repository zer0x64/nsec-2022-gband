use alloc::vec::Vec;
use core::time::Duration;
use bitflags::bitflags;

// TODO: Socket PoC stuff, remove later
extern crate std;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::io;
use std::io::{Read, Write};

const N_BIT_CYCLES: u8 = 8;
const CPU_CYCLES: u8 = (4194304 / 4 / 8192) as u8;
const CPU_CYCLES_FAST: u8 = (4194304 / 4 / 262144) as u8;

bitflags! {
    struct ControlRegister: u8 {
        const MASTER = 0x01;
        const FAST = 0x02;
        const UNUSED = 0x7C;
        const START = 0x80;
    }
}

impl Default for ControlRegister {
    fn default() -> Self {
        ControlRegister::UNUSED | ControlRegister::FAST
    }
}

#[derive(Default)]
pub struct SerialPort {
    buffer: u8,
    control: ControlRegister,

    freq_downscale_cycle: u8,
    bit_cycle: u8,
    receive_latch: u8,

    // TODO: Socket PoC stuff, remove later
    skip_handshake: bool,
    print_buffer: Vec<u8>,
    socket_wrapper: SocketWrapper,
}

impl SerialPort {
    // TODO: Socket PoC stuff, remove later
    // Exists to make only the emu create it, not the tests and benchmark
    pub fn enable_socket(&mut self) {
        self.socket_wrapper.enable();
    }

    /// Clock the serial port module.
    /// Returns a bool indicating whether an interrupt is triggered or not
    pub fn clock(&mut self) -> bool {
        self.freq_downscale_cycle += 1;
        if self.freq_downscale_cycle == CPU_CYCLES {
            self.freq_downscale_cycle = 0;

            if self.control.contains(ControlRegister::START) {
                if self.socket_wrapper.is_enabled() {
                    self.run_socket()
                } else {
                    self.run_printer()
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    fn run_socket(&mut self) -> bool {
        if self.bit_cycle == 0 {
            if !self.socket_wrapper.is_connected() {
                self.socket_wrapper.try_connect();
            }

            if self.socket_wrapper.is_connected() {
                if self.control.contains(ControlRegister::MASTER) {
                    if !self.skip_handshake {
                        match self.socket_wrapper.send(self.buffer) {
                            Ok(_) => {}
                            Err(e) => {
                                log::error!("Master failed to send: {e}");
                                self.socket_wrapper.reset_socket();
                            }
                        }
                    }

                    match self.socket_wrapper.recv() {
                        Ok(received) => {
                            self.skip_handshake = false;
                            self.receive_latch = received;
                        }
                        Err(e) => match e.kind() {
                            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => {
                                self.skip_handshake = true;
                                return false;
                            }
                            _ => {
                                log::error!("Master failed to receive: {e}");
                                self.skip_handshake = false;
                                self.socket_wrapper.reset_socket();
                            }
                        }
                    }
                } else {
                    match self.socket_wrapper.recv() {
                        Ok(received) => {
                            self.receive_latch = received;
                        }
                        Err(e) => match e.kind() {
                            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => {
                                return false;
                            }
                            _ => {
                                log::error!("Slave failed to receive: {e}");
                                self.socket_wrapper.reset_socket();
                            }
                        }
                    }

                    match self.socket_wrapper.send(self.buffer) {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("Slave failed to send: {e}");
                            self.socket_wrapper.reset_socket();
                        }
                    }
                }
            } else {
                self.socket_wrapper.reset_socket();
            }
        }

        // Increment "bits transferred" cycles only if the connection is still active
        if self.socket_wrapper.is_connected() {
            self.bit_cycle += 1;
        } else {
            self.socket_wrapper.reset_socket();
            self.bit_cycle = 0;
        }

        if self.bit_cycle == N_BIT_CYCLES {
            self.bit_cycle = 0;
            self.buffer = self.receive_latch;
            self.control.remove(ControlRegister::START);
            true
        } else {
            false
        }
    }

    fn run_printer(&mut self) -> bool {
        if self.buffer != 10u8 {
            self.print_buffer.push(self.buffer);
        }

        if self.buffer == 10u8 || self.print_buffer.len() == 64 {
            if !self.print_buffer.is_empty() {
                log::info!(
                    "Serial port: {}",
                    self.print_buffer
                        .iter()
                        .flat_map(|c| (*c as char).escape_default())
                        .collect::<alloc::string::String>()
                );
                self.print_buffer.clear();
            }
        }

        false
    }

    pub fn set_buffer(&mut self, data: u8) {
        self.buffer = data;
    }

    pub fn get_buffer(&self) -> u8 {
        self.buffer
    }

    pub fn set_control(&mut self, data: u8) {
        self.control = ControlRegister::from_bits_truncate(data) | ControlRegister::UNUSED;
    }

    pub fn get_control(&self) -> u8 {
        self.control.bits()
    }
}

// TODO: Socket PoC stuff, remove later
#[derive(Default)]
struct SocketWrapper {
    socket: Option<TcpStream>,
    listener: Option<TcpListener>,
    enabled: bool,
}

impl SocketWrapper {
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn try_connect(&mut self) {
        let host_addr = SocketAddr::from(([127, 0, 0, 1], 8001));

        if self.listener.is_none() && self.socket.is_none() {
            // Try to bind as listener first. If the address is in use, then try as client
            match TcpListener::bind(&host_addr) {
                Ok(listener) => {
                    log::info!("Started listener on {}", host_addr);
                    listener.set_nonblocking(true).unwrap();
                    self.listener = Some(listener);
                }
                Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
                    log::info!("Connecting to {}", host_addr);
                    match TcpStream::connect_timeout(&host_addr, Duration::from_millis(100)) {
                        Ok(socket) => {
                            log::info!("Connected");
                            socket.set_nonblocking(true).unwrap();
                            self.socket = Some(socket)
                        },
                        Err(e) => log::error!("Failed to connect: {}", e)
                    }
                }
                Err(e) => {
                    log::error!("Unable to create listener: {}", e);
                }
            }
        }

        // If we are the listener, try to accept. If no one is there, try another time
        if self.socket.is_none() {
            if let Some(listener) = &self.listener {
                match listener.accept() {
                    Ok((socket, _)) => {
                        log::info!("Accepted");
                        socket.set_nonblocking(true).unwrap();
                        self.socket = Some(socket);
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // Do nothing, this is fine
                    }
                    Err(e) => {
                        log::error!("Accept failed: {}", e);
                    }
                }
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn is_connected(&mut self) -> bool {
        match &self.socket {
            Some(socket) => {
                let mut dummy = [0u8];
                let connected = match socket.peek(&mut dummy) {
                    Ok(_) => true,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => true,
                    Err(e) => {
                        log::error!("is_connected peek error: {}", e);
                        false
                    },
                };

                connected
            },
            None => false,
        }
    }

    pub fn reset_socket(&mut self) {
        self.socket = None;
    }

    pub fn send(&mut self, byte: u8) -> io::Result<()> {
        if let Some(socket) = &mut self.socket {
            let send_buf = [byte];
            socket.write(&send_buf)?;
            Ok(())
        } else {
            Err(io::Error::from(io::ErrorKind::NotConnected))
        }
    }

    pub fn recv(&mut self) -> io::Result<u8> {
        if let Some(socket) = &mut self.socket {
            let mut recv_buf = [0u8];
            socket.read(&mut recv_buf)?;
            Ok(recv_buf[0])
        } else {
            Err(io::Error::from(io::ErrorKind::NotConnected))
        }
    }
}
