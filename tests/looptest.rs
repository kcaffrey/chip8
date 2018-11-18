extern crate chip8;

use std::time::Duration;

use chip8::Emulator;

#[test]
fn test_loop() {
    let program = [
        0x60, 0x00, 0x61, 0x00, 0x6E, 0x1E, 0x8E, 0x17, 0x4F, 0x01, 0x12, 0x12, 0xD0, 0x15, 0x71,
        0x05, 0x12, 0x04, 0x60, 0x05, 0x61, 0x00, 0x41, 0x1E, 0x12, 0x20, 0xD0, 0x15, 0x71, 0x05,
        0x12, 0x16, 0x60, 0x0A, 0x61, 0x00, 0x6E, 0x1E, 0x8E, 0x17, 0x4F, 0x01, 0x12, 0x32, 0xD0,
        0x15, 0x71, 0x05, 0x12, 0x24, 0x60, 0x0F, 0x61, 0x00, 0x41, 0x1E, 0x12, 0x40, 0xD0, 0x15,
        0x71, 0x05, 0x12, 0x36, 0x60, 0x14, 0x61, 0x00, 0x41, 0x19, 0x12, 0x56, 0xD0, 0x15, 0x71,
        0x05, 0x62, 0x05, 0x72, 0xFF, 0x32, 0x00, 0x12, 0x4E, 0x12, 0x44, 0xD0, 0x15, 0x12, 0x58,
    ];
    let mut emulator = Emulator::default();
    emulator.load_program(&program).expect("program is valid");
    for _ in 0..200 {
        emulator
            .execute_cycle(Duration::from_millis(17))
            .expect("shouldn't crash");
    }
}