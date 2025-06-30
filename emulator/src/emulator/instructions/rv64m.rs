
use crate::emulator::Emulator;

use super::insts::*;
use super::Instruction;

pub const RV_M: &[Instruction] = &[
    Instruction {
        mask: MASK_MUL,
        identifier: MATCH_MUL,
        name: "mul",
        execute: |emu: &mut Emulator, inst: u32, pc: u64| {
            todo!("Implement MUL instruction execution");
        },
    }
];