use alloc::vec::Vec;
use core::time::Duration;
use bitflags::bitflags;

// TODO: Socket PoC stuff, remove later
extern crate std;
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::io;
use std::io::{Read, Write};

const N_CYCLES: u8 = 8;

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

    cycles: u8,
    print_buffer: Vec<u8>,
    receive_latch: u8,

    // TODO: Socket PoC stuff, remove later
    socket_wrapper: SocketWrapper,
}

impl SerialPort {
    // TODO: Socket PoC stuff, remove later
    // Exists to make only the emu create it, not the tests and benchmark
    pub fn enable_socket(&mut self) {
        self.socket_wrapper.enable();
    }

    pub fn clock(&mut self) -> bool {
        if self.control.contains(ControlRegister::START) {
            if self.socket_wrapper.is_enabled() {
                //log::info!("Run socket");
                self.run_socket()
            } else {
                //log::info!("Run printer");
                self.run_printer()
            }
        } else {
            false
        }
    }

    fn run_socket(&mut self) -> bool {
        if self.cycles == 0 {
            if !self.socket_wrapper.is_connected() {
                //log::info!("Socket not connected, trying to connect");
                self.socket_wrapper.try_connect();
            }

            if self.socket_wrapper.is_connected() {
                if self.control.contains(ControlRegister::MASTER) {
                    match self.socket_wrapper.send(self.buffer) {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("Master failed to send: {e}");
                            self.socket_wrapper.reset_socket();
                        }
                    }

                    match self.socket_wrapper.recv() {
                        Ok(received) => {
                            self.receive_latch = received;
                            log::info!("Master first cycle, sent {}, received {}", self.buffer, self.receive_latch);
                        }
                        Err(e) => {
                            log::error!("Master failed to receive: {e}");
                            self.socket_wrapper.reset_socket();
                        }
                    }
                } else {
                    match self.socket_wrapper.recv() {
                        Ok(received) => {
                            self.receive_latch = received;
                        }
                        Err(e) => {
                            log::error!("Slave failed to receive: {e}");
                            self.socket_wrapper.reset_socket();
                        }
                    }

                    match self.socket_wrapper.send(self.buffer) {
                        Ok(_) => {
                            log::info!("Slave first cycle, sent {}, received {}", self.buffer, self.receive_latch);
                        }
                        Err(e) => {
                            log::error!("Slave failed to send: {e}");
                            self.socket_wrapper.reset_socket();
                        }
                    }
                }
            } else {
                //log::info!("Should have send, but socket is not connected. Waiting");
                self.socket_wrapper.reset_socket();
            }
        }

        // Increment cycles only if the connection is active
        if self.socket_wrapper.is_connected() {
            self.cycles += 1;
        }

        if self.cycles == N_CYCLES {
            self.cycles = 0;
            self.buffer = self.receive_latch;
            self.control.remove(ControlRegister::START);
            log::info!("Serial last cycle -- SB: {:x}, SC: {:x}", self.buffer, self.control.bits());
            true
        } else {
            false
        }
    }

    fn run_printer(&mut self) -> bool {
        if self.buffer != 10u8 {
            self.print_buffer.push(self.buffer);
        }

        if self.buffer == 10u8 || self.print_buffer.len() == 128 {
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

        self.cycles += 1;
        if self.cycles == N_CYCLES {
            self.cycles = 0;
            //self.buffer = self.receive_latch;
            self.control.remove(ControlRegister::START);
            true
        } else {
            false
        }
    }

    /*fn master(&mut self) {
        if let Some(socket) = &self.socket {
            let addr = Self::get_remote_addr(&socket);
            log::info!("Other's addr: {addr}");

            let send_buf = [self.buffer];
            socket.send_to(&send_buf, addr).expect("Master failed to send");

            let mut recv_buf = [0u8, 1];
            let (_, other) = socket.recv_from(&mut recv_buf).expect("Master failed to receive");
            if other != addr {
                panic!("Master received a udp packet from someone else: {}", other);
            }

            self.receive_latch = recv_buf[0];
        }

        log::info!("Master first cycle, sent {}, received {}", self.buffer, self.receive_latch);

        /*self.cycles += 1;
        //log::info!("SB: {:x}, SC: {:x}", self.buffer, self.control.bits());

        if self.cycles == N_CYCLES {
            self.cycles = 0;

            /*if self.buffer == 10u8 {
                self.print();
            } else {
                self.print_buffer.push(self.buffer);
            }*/

            // send data and receive data

            self.control.remove(ControlRegister::START);
            log::info!("Sent -- SB: {:x}, SC: {:x}", self.buffer, self.control.bits());
            true
        } else {
            false
        }*/
    }

    fn slave(&mut self) {
        if let Some(socket) = &self.socket {
            let addr = Self::get_remote_addr(&socket);
            log::info!("Other's addr: {addr}");

            let mut recv_buf = [0u8, 1];
            let (_, other) = socket.recv_from(&mut recv_buf).expect("Slave failed to receive");
            if other != addr {
                panic!("Slave received a udp packet from someone else: {}", other);
            }

            self.receive_latch = recv_buf[0];

            let send_buf = [self.buffer];
            socket.send_to(&send_buf, addr).expect("Slave failed to send");
        }

        log::info!("Slave first cycle, sent {}, received {}", self.buffer, self.receive_latch);
    }*/

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
    //socket: Option<UdpSocket>,
    socket: Option<TcpStream>,
    listener: Option<TcpListener>,
    enabled: bool,
}

impl SocketWrapper {
    pub fn enable(&mut self) {
        self.enabled = true;
        /*let addresses = [
            SocketAddr::from(([127, 0, 0, 1], 8001)),
            SocketAddr::from(([127, 0, 0, 1], 8002)),
        ];

        match UdpSocket::bind(&addresses[..]) {
            Ok(socket) => {
                self.socket = Some(socket);
                self.connect();
            },
            Err(e) => {
                log::error!("Failed to create socket: {}", e);
            }
        }*/
    }

    // Old UDP code, kept for reference
    /*pub fn connect(&mut self) {
        let addresses = [
            SocketAddr::from(([127, 0, 0, 1], 8001)),
            SocketAddr::from(([127, 0, 0, 1], 8002)),
        ];

        if let Some(socket) = &self.socket {
            let addr = socket.local_addr().unwrap();
            let remote = if addresses[0] == addr {
                addresses[1]
            } else {
                addresses[0]
            };

            socket.connect(remote).unwrap();

            log::info!("Socket bound to {}", socket.local_addr().unwrap());
            log::info!("Socket connected to {}", socket.peer_addr().unwrap());
        }
    }*/

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
                            //socket.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
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
                //log::info!("Accepting");
                match listener.accept() {
                    Ok((socket, _)) => {
                        log::info!("Accepted");
                        socket.set_nonblocking(false).unwrap();
                        //socket.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
                        self.socket = Some(socket);
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // Do nothing, this is fine
                        //log::info!("No one is here");
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
                socket.set_nonblocking(true).unwrap();
                let connected = match socket.peek(&mut dummy) {
                    Ok(_) => true,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => true,
                    //Err(e) if e.kind() == io::ErrorKind::TimedOut => true,
                    Err(e) => {
                        log::error!("is_connected peek error: {}", e);
                        false
                    },
                };
                socket.set_nonblocking(false);

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
            //socket.send(&send_buf)?;
            socket.write(&send_buf)?;
            Ok(())
        } else {
            Err(io::Error::from(io::ErrorKind::NotConnected))
        }
    }

    pub fn recv(&mut self) -> io::Result<u8> {
        if let Some(socket) = &mut self.socket {
            let mut recv_buf = [0u8];
            //socket.recv(&mut recv_buf)?;
            socket.read(&mut recv_buf)?;
            Ok(recv_buf[0])
        } else {
            Err(io::Error::from(io::ErrorKind::NotConnected))
        }
    }
}
