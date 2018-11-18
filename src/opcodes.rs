extern crate rand;

use rand::prelude::*;

use crate::errors::*;
use crate::system::SystemState;

pub trait IOpcodeRunner {
    fn run(&self, system: &mut SystemState, opcode: u16) -> Result;
}

pub struct OpcodeRunner;

impl IOpcodeRunner for OpcodeRunner {
    fn run(&self, system: &mut SystemState, opcode: u16) -> Result {
        match nibbles(opcode) {
            (0x0, 0x0, 0xE, 0x0) => op_cls(system),
            (0x0, 0x0, 0xE, 0xE) => op_ret(system),
            (0x0, _, _, _) => noop(), // machine call disabled
            (0x1, a, b, c) => op_jp(system, nnn(a, b, c)),
            (0x2, a, b, c) => op_call(system, nnn(a, b, c)),
            (0x3, a, b, c) => op_se_reg_byte(system, a, nn(b, c)),
            (0x4, a, b, c) => op_sne_reg_byte(system, a, nn(b, c)),
            (0x5, a, b, 0x0) => op_se_reg_reg(system, a, b),
            (0x6, a, b, c) => op_ld_reg_byte(system, a, nn(b, c)),
            (0x7, a, b, c) => op_add_reg_byte(system, a, nn(b, c)),
            (0x8, a, b, 0x0) => op_ld_reg_reg(system, a, b),
            (0x8, a, b, 0x1) => op_or(system, a, b),
            (0x8, a, b, 0x2) => op_and(system, a, b),
            (0x8, a, b, 0x3) => op_xor(system, a, b),
            (0x8, a, b, 0x4) => op_add_reg_reg(system, a, b),
            (0x8, a, b, 0x5) => op_sub_reg_reg(system, a, b),
            (0x8, a, b, 0x6) => op_shr(system, a, b),
            (0x8, a, b, 0x7) => op_subn(system, a, b),
            (0x8, a, b, 0xE) => op_shl(system, a, b),
            (0x9, a, b, 0x0) => op_sne_reg_reg(system, a, b),
            (0xA, a, b, c) => op_ld_i(system, nnn(a, b, c)),
            (0xB, a, b, c) => op_jp_v0_addr(system, nnn(a, b, c)),
            (0xC, a, b, c) => op_rnd_reg_byte(system, a, nn(b, c)),
            (0xD, a, b, c) => op_drw(system, a, b, c),
            (0xE, a, 0x9, 0xE) => op_skp(system, a),
            (0xE, a, 0xA, 0x1) => op_sknp(system, a),
            (0xF, a, 0x0, 0x7) => op_ld_reg_dt(system, a),
            (0xF, a, 0x0, 0xA) => op_ld_k(system, a),
            (0xF, a, 0x1, 0x5) => op_ld_dt_reg(system, a),
            (0xF, a, 0x1, 0x8) => op_ld_st(system, a),
            (0xF, a, 0x1, 0xE) => op_add_i(system, a),
            (0xF, a, 0x2, 0x9) => op_ld_f(system, a),
            (0xF, a, 0x3, 0x3) => op_ld_b(system, a),
            (0xF, a, 0x5, 0x5) => op_store_regs(system, a),
            (0xF, a, 0x6, 0x5) => op_load_regs(system, a),
            _ => err(&format!(
                "unknown opcode: 0x{:X}; pc=0x{:04X}, registers={:?}",
                opcode, system.program_counter, system.registers
            )),
        }
    }
}

fn nibbles(opcode: u16) -> (u8, u8, u8, u8) {
    (
        ((opcode & 0xF000) >> 12) as u8,
        ((opcode & 0x0F00) >> 8) as u8,
        ((opcode & 0x00F0) >> 4) as u8,
        (opcode & 0x000F) as u8,
    )
}

fn nnn(a: u8, b: u8, c: u8) -> u16 {
    (u16::from(a) << 8) | u16::from(nn(b, c))
}

fn nn(a: u8, b: u8) -> u8 {
    ((a & 0xF) << 4) | (b & 0xF)
}

fn bcd(val: u8) -> (u8, u8, u8) {
    ((val / 100) % 10, (val / 10) % 10, val % 10)
}

fn sanitize_addr(system: &SystemState, addr: u16, len: usize) -> std::result::Result<usize, Error> {
    let i = usize::from(addr);
    if i < 0x200 || i + len >= system.memory.len() {
        return err("invalid address");
    }
    Ok(i)
}

fn noop() -> Result {
    Ok(())
}

fn op_cls(system: &mut SystemState) -> Result {
    for row in system.display.iter_mut() {
        for val in row.iter_mut() {
            *val = false;
        }
    }
    Ok(())
}

fn op_ret(system: &mut SystemState) -> Result {
    if system.stack_pointer == 0 {
        return err("can't return from empty call stack");
    }
    system.stack_pointer -= 1;
    system.program_counter = system.stack[usize::from(system.stack_pointer)];
    Ok(())
}

fn op_jp(system: &mut SystemState, address: u16) -> Result {
    system.program_counter = address;
    Ok(())
}

fn op_call(system: &mut SystemState, address: u16) -> Result {
    if usize::from(system.stack_pointer + 1) > system.stack.len() {
        return err("stack overflow");
    }
    let address = sanitize_addr(system, address, 1)?;
    system.stack[usize::from(system.stack_pointer)] = system.program_counter;
    system.stack_pointer += 1;
    system.program_counter = address as u16;
    Ok(())
}

fn op_se_reg_byte(system: &mut SystemState, x: u8, byte: u8) -> Result {
    if system.registers[usize::from(x)] == byte {
        system.program_counter += 2;
    }
    Ok(())
}

fn op_sne_reg_byte(system: &mut SystemState, x: u8, byte: u8) -> Result {
    if system.registers[usize::from(x)] != byte {
        system.program_counter += 2;
    }
    Ok(())
}

fn op_se_reg_reg(system: &mut SystemState, x: u8, y: u8) -> Result {
    if system.registers[usize::from(x)] == system.registers[usize::from(y)] {
        system.program_counter += 2;
    }
    Ok(())
}

fn op_sne_reg_reg(system: &mut SystemState, x: u8, y: u8) -> Result {
    if system.registers[usize::from(x)] != system.registers[usize::from(y)] {
        system.program_counter += 2;
    }
    Ok(())
}

fn op_ld_reg_byte(system: &mut SystemState, x: u8, byte: u8) -> Result {
    system.registers[usize::from(x)] = byte;
    Ok(())
}

fn op_add_reg_byte(system: &mut SystemState, x: u8, byte: u8) -> Result {
    let idx = usize::from(x);
    // Silently overflow, unlike 8xy4 which sets the carry flag.
    let (result, _) = system.registers[idx].overflowing_add(byte);
    system.registers[idx] = result;
    Ok(())
}

fn op_ld_reg_reg(system: &mut SystemState, x: u8, y: u8) -> Result {
    system.registers[usize::from(x)] = system.registers[usize::from(y)];
    Ok(())
}

fn op_or(system: &mut SystemState, x: u8, y: u8) -> Result {
    system.registers[usize::from(x)] |= system.registers[usize::from(y)];
    Ok(())
}

fn op_and(system: &mut SystemState, x: u8, y: u8) -> Result {
    system.registers[usize::from(x)] &= system.registers[usize::from(y)];
    Ok(())
}

fn op_xor(system: &mut SystemState, x: u8, y: u8) -> Result {
    system.registers[usize::from(x)] ^= system.registers[usize::from(y)];
    Ok(())
}

fn op_add_reg_reg(system: &mut SystemState, x: u8, y: u8) -> Result {
    let (ix, iy) = (usize::from(x), usize::from(y));
    let (result, overflow) = system.registers[ix].overflowing_add(system.registers[iy]);
    system.registers[ix] = result;
    system.registers[0xF] = if overflow { 0 } else { 1 }; // VF = NOT borrow
    Ok(())
}

fn op_sub_reg_reg(system: &mut SystemState, x: u8, y: u8) -> Result {
    let (ix, iy) = (usize::from(x), usize::from(y));
    let (result, overflow) = system.registers[ix].overflowing_sub(system.registers[iy]);
    system.registers[ix] = result;
    system.registers[0xF] = if overflow { 0 } else { 1 }; // VF = NOT borrow
    Ok(())
}

fn op_shr(system: &mut SystemState, x: u8, y: u8) -> Result {
    let idx = usize::from(x);
    let vy = system.registers[usize::from(y)];
    system.registers[idx] = vy >> 1;
    system.registers[0xF] = vy & 0x01;
    Ok(())
}

fn op_subn(system: &mut SystemState, x: u8, y: u8) -> Result {
    let (ix, iy) = (usize::from(x), usize::from(y));
    let (result, overflow) = system.registers[iy].overflowing_sub(system.registers[ix]);
    system.registers[ix] = result;
    system.registers[0xF] = if overflow { 1 } else { 0 };
    Ok(())
}

fn op_shl(system: &mut SystemState, x: u8, y: u8) -> Result {
    let idx = usize::from(x);
    let vy = system.registers[usize::from(y)];
    system.registers[idx] = vy << 1;
    system.registers[0xF] = (vy >> 7) & 0x1;
    Ok(())
}

fn op_ld_i(system: &mut SystemState, addr: u16) -> Result {
    system.address_register = addr & 0xFFF;
    Ok(())
}

fn op_jp_v0_addr(system: &mut SystemState, addr: u16) -> Result {
    let v0 = system.registers[0];
    let (addr, overflow) = addr.overflowing_add(u16::from(v0));
    if overflow {
        return err("invalid address");
    }
    system.program_counter = sanitize_addr(system, addr, 2)? as u16;
    Ok(())
}

fn op_rnd_reg_byte(system: &mut SystemState, reg: u8, byte: u8) -> Result {
    system.registers[usize::from(reg)] = random::<u8>() & byte;
    Ok(())
}

fn op_drw(system: &mut SystemState, x: u8, y: u8, n: u8) -> Result {
    let vx = system.registers[usize::from(x)];
    let vy = system.registers[usize::from(y)];

    let i = usize::from(system.address_register);
    if i + usize::from(n) > system.memory.len() {
        return err("invalid address");
    }
    let mut collide = false;
    for (idx, byte) in system.memory[i..i + usize::from(n)].iter().enumerate() {
        for bit in 0..8 {
            let iy = (usize::from(vy) + idx) % system.display.len();
            let ix = (usize::from(vx + bit)) % system.display[0].len();
            let on = (byte & (1 << (7 - bit))) != 0;
            collide = collide || (on && system.display[iy][ix]);
            system.display[iy][ix] ^= on;
        }
    }
    system.registers[0xF] = if collide { 1 } else { 0 };
    Ok(())
}

fn op_skp(system: &mut SystemState, x: u8) -> Result {
    let vx = system.registers[usize::from(x)];
    if vx > 0xF {
        return err("invalid key value");
    }
    if system.keys[usize::from(vx)] {
        system.program_counter += 2;
    }
    Ok(())
}

fn op_sknp(system: &mut SystemState, x: u8) -> Result {
    let vx = system.registers[usize::from(x)];
    if vx > 0xF {
        return err("invalid key value");
    }
    if !system.keys[usize::from(vx)] {
        system.program_counter += 2;
    }
    Ok(())
}

fn op_ld_reg_dt(system: &mut SystemState, x: u8) -> Result {
    system.registers[usize::from(x)] = system.delay_timer;
    Ok(())
}

fn op_ld_k(system: &mut SystemState, x: u8) -> Result {
    if let Some(key) = system.pending_keypress.take() {
        system.waiting_for_key = false;
        system.registers[usize::from(x)] = key;
    } else {
        // Mark the system as waiting for a keypress, and reset the program counter so
        // that this instruction gets executed again when a key is pressed.
        // The intention is that the wrapping emulator will not attempt to execute this
        // instruction again until a key is pressed, but even if execution continues
        // it should work.
        system.waiting_for_key = true;
        system.program_counter -= 2;
    }
    Ok(())
}

fn op_ld_dt_reg(system: &mut SystemState, x: u8) -> Result {
    system.delay_timer = system.registers[usize::from(x)];
    Ok(())
}

fn op_ld_st(system: &mut SystemState, x: u8) -> Result {
    system.sound_timer = system.registers[usize::from(x)];
    Ok(())
}

fn op_add_i(system: &mut SystemState, x: u8) -> Result {
    let (addr, _) = system
        .address_register
        .overflowing_add(u16::from(system.registers[usize::from(x)]));
    system.address_register = addr;
    Ok(())
}

fn op_ld_f(system: &mut SystemState, x: u8) -> Result {
    let vx = system.registers[usize::from(x)];
    system.address_register = system.get_sprite_location(vx)?;
    Ok(())
}

fn op_ld_b(system: &mut SystemState, x: u8) -> Result {
    let vx = system.registers[usize::from(x)];
    let i = sanitize_addr(system, system.address_register, 3)?;
    let (hundreds, tens, ones) = bcd(vx);
    system.memory[i..(i + 3)].copy_from_slice(&[hundreds, tens, ones]);
    Ok(())
}

fn op_store_regs(system: &mut SystemState, x: u8) -> Result {
    let x = usize::from(x);
    let i = sanitize_addr(system, system.address_register, x + 1)?;
    system.memory[i..=i + x].copy_from_slice(&system.registers[0..=x]);
    system.address_register = (i + x + 1) as u16;
    Ok(())
}

fn op_load_regs(system: &mut SystemState, x: u8) -> Result {
    let x = usize::from(x);
    let i = sanitize_addr(system, system.address_register, x)?;
    system.registers[0..=x].copy_from_slice(&system.memory[i..=i + x]);
    system.address_register = (i + x + 1) as u16;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nibbles() {
        assert_eq!(nibbles(0x8765), (8, 7, 6, 5));
    }

    #[test]
    fn test_nnn() {
        assert_eq!(nnn(0xA, 0xB, 0xC), 0xABC);
    }

    #[test]
    fn test_nn() {
        assert_eq!(nn(0xF, 0xA), 0xFA);
    }

    #[test]
    fn test_bcd() {
        assert_eq!(bcd(45), (0, 4, 5));
        assert_eq!(bcd(3), (0, 0, 3));
        assert_eq!(bcd(213), (2, 1, 3));
    }

    #[test]
    fn test_op_ld_b() {
        let mut system = SystemState::default();
        system.address_register = 0x800;
        system.registers[8] = 123;
        op_ld_b(&mut system, 8).unwrap();
        assert_eq!(system.memory[0x800..0x803], [1, 2, 3]);
    }

    #[test]
    fn test_sanitize_addr() {
        let system = SystemState::default();
        assert!(sanitize_addr(&system, 0x100, 1).is_err());
        assert!(sanitize_addr(&system, 4100, 1).is_err());
        assert!(sanitize_addr(&system, 4000, 100).is_err());
        assert_eq!(sanitize_addr(&system, 4000, 95).unwrap(), 4000);
    }

    #[test]
    fn test_cls() {
        let mut system = SystemState::default();
        system.display[0][0..8].copy_from_slice(&random::<[bool; 8]>());
        system.display[1][0..8].copy_from_slice(&random::<[bool; 8]>());
        system.display[2][0..8].copy_from_slice(&random::<[bool; 8]>());
        op_cls(&mut system).unwrap();
        for row in system.display.iter() {
            for val in row.iter() {
                assert_eq!(false, *val);
            }
        }
    }

    #[test]
    fn test_drw() {
        let mut system = SystemState::default();
        system.memory[0x200] = 0b1001_0101;
        system.address_register = 0x200;
        system.registers[5..8].copy_from_slice(&[30, 20, 10]);
        op_drw(&mut system, 6, 5, 1).unwrap();
        assert_eq!(
            &system.display[30][20..28],
            &[true, false, false, true, false, true, false, true]
        );
        assert_eq!(system.registers[0xF], 0);

        system.memory[0x201] = 0b1110_0000;
        system.address_register = 0x201;
        op_drw(&mut system, 6, 5, 1).unwrap();
        assert_eq!(
            &system.display[30][20..28],
            &[false, true, true, true, false, true, false, true]
        );
        assert_eq!(system.registers[0xF], 1);

        system.address_register = 0x200;
        op_drw(&mut system, 6, 7, 2).unwrap();
        assert_eq!(
            &system.display[10][20..28],
            &[true, false, false, true, false, true, false, true]
        );
        assert_eq!(
            &system.display[11][20..28],
            &[true, true, true, false, false, false, false, false]
        );
        assert_eq!(system.registers[0xF], 0);
    }
}
