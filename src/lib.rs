mod errors;
mod opcodes;
mod sprites;
mod system;

pub use crate::errors::*;
use crate::opcodes::{IOpcodeRunner, OpcodeRunner};
use crate::system::SystemState;

pub struct Emulator {
    system: SystemState,
    opcode_runner: Box<dyn IOpcodeRunner>,
}

impl Default for Emulator {
    fn default() -> Emulator {
        Emulator::new(Box::new(OpcodeRunner))
    }
}

impl Emulator {
    fn new(opcode_runner: Box<dyn IOpcodeRunner>) -> Emulator {
        Emulator {
            system: SystemState::new(),
            opcode_runner,
        }
    }

    pub fn load_program(&mut self, program: &[u8]) -> Result {
        self.system.load_program(program)
    }

    pub fn execute_cycle(&mut self) -> Result {
        let opcode = self.system.next_opcode();
        self.opcode_runner.run(&mut self.system, opcode)?;
        self.system.tick_timers();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct MockOpcodeRunner {
        last_opcode: Rc<RefCell<Option<u16>>>,
    }

    impl IOpcodeRunner for MockOpcodeRunner {
        fn run(&self, _: &mut SystemState, opcode: u16) -> Result {
            *self.last_opcode.borrow_mut() = Some(opcode);
            Ok(())
        }
    }

    #[test]
    fn execute_cycle() {
        let last_opcode: Rc<RefCell<Option<u16>>> = Default::default();
        let mock_runner = MockOpcodeRunner {
            last_opcode: Rc::clone(&last_opcode),
        };
        let mut emulator = Emulator::new(Box::new(mock_runner));
        emulator
            .system
            .load_program(&[0x01, 0x02, 0x03, 0x04])
            .unwrap();
        emulator.system.delay_timer = 10;

        emulator.execute_cycle().unwrap();
        assert_eq!(*last_opcode.borrow(), Some(0x0102));
        assert_eq!(emulator.system.delay_timer, 9);

        emulator.execute_cycle().unwrap();
        assert_eq!(*last_opcode.borrow(), Some(0x0304));
        assert_eq!(emulator.system.delay_timer, 8);
    }
}
