use crate::emulator::Emulator;

use super::insts::*;
use super::Instruction;

pub const RV_A: &[Instruction] = &[
    Instruction {
        mask: MASK_MUL,
        identifier: MATCH_MUL,
        name: "todo!",
        execute: |emu: &mut Emulator, inst: u32, pc: u64| {
            todo!("Implement MUL instruction execution");
        },
    }
];