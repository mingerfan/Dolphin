use crate::emulator::Emulator;

use super::Instruction;
use super::insts::*;

pub const RV_M: &[Instruction] = &[Instruction {
    mask: MASK_MUL,
    identifier: MATCH_MUL,
    name: "mul",
    execute: |_emu: &mut Emulator, _inst: u32, _pc: u64| {
        todo!("Implement MUL instruction execution");
    },
}];
