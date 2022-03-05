use alloc::vec::Vec;
use bitflags::bitflags;

const N_CYCLES: u8 = 8;

bitflags! {
    struct ControlRegister: u8 {
        const SOURCE = 0x01;
        const SPEED = 0x02;
        const UNUSED = 0x7C;
        const START = 0x80;
    }
}

impl Default for ControlRegister {
    fn default() -> Self {
        ControlRegister::UNUSED | ControlRegister::SPEED
    }
}

#[derive(Default)]
pub struct SerialPort {
    buffer: u8,
    control: ControlRegister,

    cycles: u8,
    print_buffer: Vec<u8>
}

impl SerialPort {
    pub fn clock(&mut self) -> bool {
        let interrupt = if self.control.contains(ControlRegister::START) {
            self.cycles += 1;

            if self.cycles == N_CYCLES {
                self.cycles = 0;

                if self.buffer == 10u8 {
                    self.print();
                } else {
                    self.print_buffer.push(self.buffer);
                }

                // send data and receive data

                self.control.remove(ControlRegister::START);
                true
            } else {
                false
            }
        } else {
            false
        };

        interrupt
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

    fn print(&mut self) {
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
}
