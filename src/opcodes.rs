use crate::errors::*;
use crate::system::SystemState;

pub trait IOpcodeRunner {
    fn run(&self, system: &mut SystemState, opcode: u16) -> Result;
}

pub struct OpcodeRunner;

impl IOpcodeRunner for OpcodeRunner {
    fn run(&self, system: &mut SystemState, opcode: u16) -> Result {
        // println!(
        //    "opcode 0x{:04X}: pc=0x{:04X}, i=0x{:06x}, regs={:?}",
        //    opcode, system.program_counter, system.address_register, system.registers
        // );
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
            (0x8, a, _, 0x6) => op_shr(system, a),
            (0x8, a, b, 0x7) => op_subn(system, a, b),
            (0x8, a, _, 0xE) => op_shl(system, a),
            (0x9, a, b, 0x0) => op_sne_reg_reg(system, a, b),
            (0xA, a, b, c) => op_ld_i(system, nnn(a, b, c)),
            (0xB, a, b, c) => op_jp_v0_addr(system, nnn(a, b, c)),
            (0xC, a, b, c) => op_rnd_reg_byte(system, a, nn(b, c)),
            (0xD, a, b, c) => op_drw(system, a, b, c),
            // TODO: E and F opcodes
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

fn noop() -> Result {
    Ok(())
}

fn op_cls(_system: &mut SystemState) -> Result {
    // TODO: Implement
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
    if address < 0x200 || usize::from(address) >= system.memory.len() {
        return err("invalid address");
    }
    if usize::from(system.stack_pointer + 1) > system.stack.len() {
        return err("stack overflow");
    }
    system.stack[usize::from(system.stack_pointer)] = system.program_counter;
    system.stack_pointer += 1;
    system.program_counter = address;
    Ok(())
}

fn op_se_reg_byte(system: &mut SystemState, reg: u8, byte: u8) -> Result {
    if system.registers[usize::from(reg)] == byte {
        system.program_counter += 2;
    }
    Ok(())
}

fn op_sne_reg_byte(system: &mut SystemState, reg: u8, byte: u8) -> Result {
    if system.registers[usize::from(reg)] != byte {
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

fn op_ld_reg_byte(system: &mut SystemState, reg: u8, byte: u8) -> Result {
    system.registers[usize::from(reg)] = byte;
    Ok(())
}

fn op_add_reg_byte(system: &mut SystemState, reg: u8, byte: u8) -> Result {
    let idx = usize::from(reg);
    // Silently overflow, unlike 8xy4 which sets the carry flag.
    let (result, _) = system.registers[idx].overflowing_add(byte);
    system.registers[idx] = result;
    Ok(())
}

fn op_ld_reg_reg(system: &mut SystemState, a: u8, b: u8) -> Result {
    let (ia, ib) = (usize::from(a), usize::from(b));
    system.registers[ia] = system.registers[ib];
    Ok(())
}

fn op_or(system: &mut SystemState, a: u8, b: u8) -> Result {
    let (ia, ib) = (usize::from(a), usize::from(b));
    system.registers[ia] |= system.registers[ib];
    Ok(())
}

fn op_and(system: &mut SystemState, a: u8, b: u8) -> Result {
    let (ia, ib) = (usize::from(a), usize::from(b));
    system.registers[ia] &= system.registers[ib];
    Ok(())
}

fn op_xor(system: &mut SystemState, a: u8, b: u8) -> Result {
    let (ia, ib) = (usize::from(a), usize::from(b));
    system.registers[ia] ^= system.registers[ib];
    Ok(())
}

fn op_add_reg_reg(system: &mut SystemState, a: u8, b: u8) -> Result {
    let (ia, ib) = (usize::from(a), usize::from(b));
    let (result, overflow) = system.registers[ia].overflowing_add(system.registers[ib]);
    system.registers[ia] = result;
    system.registers[0xF] = if overflow { 1 } else { 0 };
    Ok(())
}

fn op_sub_reg_reg(system: &mut SystemState, a: u8, b: u8) -> Result {
    let (ia, ib) = (usize::from(a), usize::from(b));
    let (result, overflow) = system.registers[ia].overflowing_sub(system.registers[ib]);
    system.registers[ia] = result;
    system.registers[0xF] = if overflow { 1 } else { 0 };
    Ok(())
}

fn op_shr(system: &mut SystemState, a: u8) -> Result {
    let idx = usize::from(a);
    system.registers[0xF] = system.registers[idx] & 0x01;
    system.registers[idx] >>= 1;
    Ok(())
}

fn op_subn(system: &mut SystemState, a: u8, b: u8) -> Result {
    let (ia, ib) = (usize::from(a), usize::from(b));
    let (result, overflow) = system.registers[ib].overflowing_sub(system.registers[ia]);
    system.registers[ia] = result;
    system.registers[0xF] = if overflow { 1 } else { 0 };
    Ok(())
}

fn op_shl(system: &mut SystemState, a: u8) -> Result {
    let idx = usize::from(a);
    system.registers[0xF] = (system.registers[idx] & 0b1000_0000) >> 7;
    system.registers[idx] <<= 1;
    Ok(())
}

fn op_ld_i(_system: &mut SystemState, _addr: u16) -> Result {
    err("ld i not implemented")
}

fn op_jp_v0_addr(_system: &mut SystemState, _addr: u16) -> Result {
    err("jp v0 addr not implemented")
}

fn op_rnd_reg_byte(_system: &mut SystemState, _reg: u8, _byte: u8) -> Result {
    err("rnd not implemented")
}

fn op_drw(_system: &mut SystemState, _x: u8, _y: u8, _n: u8) -> Result {
    // TODO: Implement
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
}
