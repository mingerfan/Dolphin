use crate::emulator::Emulator;

use super::Instruction;
use super::insts::*;

pub const RV_A: &[Instruction] = &[Instruction {
    mask: MASK_MUL,
    identifier: MATCH_MUL,
    name: "todo!",
    execute: |_emu: &mut Emulator, _inst: u32, _pc: u64| {
        todo!("Implement MUL instruction execution");
    },
}];
