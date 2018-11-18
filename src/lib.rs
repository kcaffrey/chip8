mod errors;
mod opcodes;
mod sprites;
mod system;

use std::time::Duration;

pub use crate::errors::*;
use crate::opcodes::{IOpcodeRunner, OpcodeRunner};
use crate::system::SystemState;

const TIMER_DELTA: Duration = Duration::from_nanos(16_666_666); // 60hz

/// A CHIP-8 emulator.
///
/// # Examples
///
///```
/// extern crate chip8;
///
/// use chip8::{Emulator, AudioHandler};
///
/// struct MyAudioHandler;
///
/// impl AudioHandler for MyAudioHandler {
///     fn start_sound(&mut self) {
///         // Start playing sound...
///     }
///     fn stop_sound(&mut self) {
///         // Stop playing sound...
///     }
/// }
///
/// fn main() {
///     let mut emulator = Emulator::default();
///     emulator.set_audio_handler(Box::new(MyAudioHandler));
/// }
/// ```
pub struct Emulator {
    system: SystemState,
    opcode_runner: Box<dyn IOpcodeRunner>,
    audio: Box<dyn AudioHandler>,
    sound_playing: bool,
    program_loaded: bool,
    delta_since_timers: Duration,
}

impl Default for Emulator {
    fn default() -> Emulator {
        Emulator::new(Box::new(OpcodeRunner))
    }
}

pub trait AudioHandler {
    fn start_sound(&mut self);
    fn stop_sound(&mut self);
}

impl Emulator {
    fn new(opcode_runner: Box<dyn IOpcodeRunner>) -> Emulator {
        Emulator {
            system: SystemState::new(),
            opcode_runner,
            audio: Box::new(NullAudio),
            sound_playing: false,
            program_loaded: false,
            delta_since_timers: Duration::from_micros(0),
        }
    }

    /// Loads a program into the emulator. If a program was previously loaded, the
    /// emulator must be reset first.
    ///
    /// # Errors
    ///
    /// If the program is invalid, such as being too long, an error is returned.
    /// The emulator will be left in the prior state after returning an error result.
    ///
    /// # Panics
    ///
    /// Panics if a program has already been loaded.
    pub fn load_program(&mut self, program: &[u8]) -> Result {
        assert!(!self.program_loaded);
        self.system.load_program(program)?;
        self.program_loaded = true;
        Ok(())
    }

    /// Perform a hard reset of the emulator state. A program must be reloaded
    /// before executing any cycles.
    pub fn reset(&mut self) {
        self.system = SystemState::default();
        self.program_loaded = false;
        self.sound_playing = false;
        self.audio.stop_sound();
    }

    /// Executes a single emulation cycle of executing an instruction
    /// and ticking timers.
    ///
    /// # Errors
    ///
    /// If the next instruction in the program is invalid for any reason,
    /// such as causing a stack overflow or invalid memory access, an error
    /// is returned. In this case, the instruction is consumed, and further
    /// execution may result in undefined behavior. The error should be reported
    /// to the user and execution halted.
    ///
    /// It is recommended to reset the emulator before executing any more cycles.
    ///
    /// # Panics
    ///
    /// Panics if a program has not been loaded, or if a reset has occurred and a
    /// new program has not been loaded.
    pub fn execute_cycle(&mut self, delta_time: Duration) -> Result {
        assert!(self.program_loaded);
        if !self.system.waiting_for_key {
            // Tick timers if necessary.
            self.delta_since_timers += delta_time;
            while self.delta_since_timers >= TIMER_DELTA {
                self.system.tick_timers();
                self.delta_since_timers -= TIMER_DELTA;
            }

            // Run opcode
            let opcode = self.system.next_opcode();
            self.opcode_runner.run(&mut self.system, opcode)?;

            // Play sounds
            if self.system.sound_timer > 0 && !self.sound_playing {
                self.sound_playing = true;
                self.audio.start_sound();
            } else if self.system.sound_timer == 0 && self.sound_playing {
                self.sound_playing = false;
                self.audio.stop_sound();
            }
        }
        Ok(())
    }

    /// Gets a reference to the system display
    pub fn get_display(&self) -> &[[bool; 64]] {
        &self.system.display
    }

    /// Set the audio handler that will receive callbacks for when to start and stop
    /// audio. CHIP-8 is a simple system and only supports a single tone which may only
    /// be started or stopped.
    pub fn set_audio_handler(&mut self, audio: Box<dyn AudioHandler>) {
        self.audio = audio;
    }

    /// Callback for keyboard input when a keypad key is pressed.
    ///
    /// # Panics
    ///
    /// Panics if key is greater than 15.
    pub fn on_key_down(&mut self, key: u8) {
        assert!(key <= 0xF);
        self.system.keys[usize::from(key)] = true;
        if self.system.waiting_for_key && self.system.pending_keypress.is_none() {
            self.system.pending_keypress = Some(key);
        }
        self.system.waiting_for_key = false;
    }

    /// Callback for keyboard input when a keypad key is released.
    ///
    /// # Panics
    ///
    /// Panics if key is greater than 15.
    pub fn on_key_up(&mut self, key: u8) {
        assert!(key <= 0xF);
        self.system.keys[usize::from(key)] = false;
    }
}

struct NullAudio;

impl AudioHandler for NullAudio {
    fn start_sound(&mut self) {}
    fn stop_sound(&mut self) {}
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
        emulator.load_program(&[0x01, 0x02, 0x03, 0x04]).unwrap();
        emulator.system.delay_timer = 10;

        emulator.execute_cycle(Duration::from_millis(17)).unwrap();
        assert_eq!(*last_opcode.borrow(), Some(0x0102));
        assert_eq!(emulator.system.delay_timer, 9);

        emulator.execute_cycle(Duration::from_millis(17)).unwrap();
        assert_eq!(*last_opcode.borrow(), Some(0x0304));
        assert_eq!(emulator.system.delay_timer, 8);
    }
}
