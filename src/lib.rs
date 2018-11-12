mod opcodes;
mod sprites;
mod system;

use crate::system::System;

pub struct Emulator {
    system: System,
}

impl Emulator {
    pub fn new() -> Emulator {
        Emulator {
            system: System::new(),
        }
    }

    pub fn run(&mut self) {
        self.system.execute_cycle();
        println!("Hello, world");
    }
}
