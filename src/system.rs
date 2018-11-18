use crate::errors::*;
use crate::sprites;

pub struct SystemState {
    pub memory: [u8; 4096],
    pub registers: [u8; 16],
    pub address_register: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub stack: [u16; 16],
    pub display: [[bool; 64]; 32],
    pub keys: [bool; 16],
    pub waiting_for_key: bool,
    pub pending_keypress: Option<u8>,
}

impl Default for SystemState {
    fn default() -> SystemState {
        SystemState::new()
    }
}

impl SystemState {
    pub fn new() -> SystemState {
        let mut system = SystemState {
            memory: [0; 4096],
            registers: [0; 16],
            address_register: 0,
            delay_timer: 0,
            sound_timer: 0,
            program_counter: 0,
            stack_pointer: 0,
            stack: [0; 16],
            display: [[false; 64]; 32],
            keys: [false; 16],
            waiting_for_key: false,
            pending_keypress: None,
        };
        system.load_sprites();
        system
    }

    pub fn next_opcode(&mut self) -> u16 {
        let addr = self.program_counter as usize;
        let opcode = (u16::from(self.memory[addr]) << 8) | u16::from(self.memory[addr + 1]);
        self.program_counter += 2;
        opcode
    }

    pub fn load_program(&mut self, program: &[u8]) -> Result {
        if program.len() > self.memory.len() - 0x200 {
            return err("program too long");
        }
        self.memory[0x200..0x200 + program.len()].copy_from_slice(program);
        self.program_counter = 0x200;
        Ok(())
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn get_sprite_location(&self, digit: u8) -> std::result::Result<u16, Error> {
        if digit > 15 {
            return err(&format!("invalid sprite digit: {}", digit));
        }
        Ok(u16::from(digit * 5))
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
        let system = SystemState::new();
        assert_eq!(&system.memory[0..80], &sprites::HEX_DIGITS[..]);
    }

    #[test]
    fn sprite_locations() {
        let system = SystemState::new();
        assert_eq!(system.get_sprite_location(5).unwrap(), 25);
        assert!(system.get_sprite_location(20).is_err());
    }

    #[test]
    fn programs_load() {
        let mut system = SystemState::new();
        let program = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        system.load_program(&program).unwrap();
        assert_eq!(&system.memory[0x200..0x200 + program.len()], &program[..]);
        assert_eq!(system.program_counter, 0x200);
    }

    #[test]
    fn program_too_long() {
        let mut system = SystemState::new();
        assert!(
            system.load_program(&[0x55; 4000][..]).is_err(),
            "program should be too long"
        );
    }

    #[test]
    fn opcodes_read_correctly() {
        let mut system = SystemState::new();
        system.program_counter = 0x200;
        system.memory[0x200..0x202].copy_from_slice(&[0x12, 0x34]);
        assert_eq!(system.next_opcode(), 0x1234);
        assert_eq!(system.program_counter, 0x202);
    }

    #[test]
    fn tick_timers() {
        let mut system = SystemState::new();

        system.delay_timer = 5;
        system.sound_timer = 8;
        system.tick_timers();
        assert_eq!(system.delay_timer, 4);
        assert_eq!(system.sound_timer, 7);

        system.delay_timer = 10;
        system.sound_timer = 0;
        system.tick_timers();
        assert_eq!(system.delay_timer, 9);
        assert_eq!(system.sound_timer, 0);

        system.delay_timer = 0;
        system.sound_timer = 20;
        system.tick_timers();
        assert_eq!(system.delay_timer, 0);
        assert_eq!(system.sound_timer, 19);
    }
}
