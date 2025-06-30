use crate::emulator::Emulator;

use super::Instruction;
use super::insts::*;


pub const RV_I: &[Instruction] = &[
    Instruction {
        mask: MASK_LUI,
        identifier: MATCH_LUI,
        name: "lui",
        execute: |emu: &mut Emulator, inst: u32, pc: u64| {
            todo!("Implement LUI instruction execution");
        },
    }
];
