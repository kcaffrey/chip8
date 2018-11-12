use crate::opcodes;
use crate::sprites;

pub struct System {
    memory: [u8; 4096],
    registers: [u8; 16],
    address_register: u16,
    delay_timer: u8,
    sound_timer: u8,
    program_counter: u16,
    stack_pointer: u8,
    stack: [u8; 16],
}

impl System {
    pub fn new() -> System {
        let mut system = System {
            memory: [0; 4096],
            registers: [0; 16],
            address_register: 0,
            delay_timer: 0,
            sound_timer: 0,
            program_counter: 0,
            stack_pointer: 0,
            stack: [0; 16],
        };
        system.load_sprites();
        system
    }

    pub fn execute_cycle(&mut self) {
        let opcode: u16 = (self.memory[self.program_counter as usize] as u16) << 8
            | (self.memory[(self.program_counter + 1) as usize] as u16);
        opcodes::run(self, opcode);
        self.program_counter += 2;
    }

    fn load_sprites(&mut self) {
        let fonts = &mut self.memory[0..80];
        fonts.copy_from_slice(&sprites::HEX_DIGITS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_has_sprites() {
        let system = System::new();
        assert_eq!(&system.memory[0..80], &sprites::HEX_DIGITS[..]);
    }
}
