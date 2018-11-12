use crate::system::System;

pub fn run(system: &mut System, opcode: u16) {
    let nibbles = (
        (opcode & 0xF000 >> 12) as u8,
        (opcode & 0x0F00 >> 8) as u8,
        (opcode & 0x00F0 >> 4) as u8,
        (opcode & 0x000F) as u8,
    );
    match nibbles {
        (0x0, 0x0, 0xE, 0x0) => op_cls(system),
        (0x0, 0x0, 0xE, 0xE) => op_ret(system),
        (0x0, _, _, _) => op_sys(system, nnn(nibbles.1, nibbles.2, nibbles.3)),
        _ => panic!("unknown opcode {}", opcode),
    }
}

fn nnn(a: u8, b: u8, c: u8) -> u16 {
    ((a as u16) << 8) + ((b as u16) << 4) + (c as u16)
}

fn op_sys(_system: &mut System, _address: u16) {}

fn op_cls(_system: &mut System) {}

fn op_ret(_system: &mut System) {}

mod tests {
    use super::*;

    #[test]
    fn test_nnn() {
        assert_eq!(nnn(0xA, 0xB, 0xC), 0xABC);
    }
}
