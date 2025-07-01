use crate::emulator::{Emulator, Exception::*};

use super::insts::*;
use super::*;


pub const RV_I: &[Instruction] = &[
    Instruction {
        mask: MASK_LUI,
        identifier: MATCH_LUI,
        name: "lui",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let u = parse_format_u(inst);
            emu.set_reg(u.rd, u.imm)
        },
    },
    Instruction {
        mask: MASK_AUIPC,
        identifier: MATCH_AUIPC,
        name: "auipc",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let u = parse_format_u(inst);
            let pc = emu.get_pc();
            emu.set_reg(u.rd, pc.wrapping_add(u.imm))
        },
    },
    Instruction {
        mask: MASK_JAL,
        identifier: MATCH_JAL,
        name: "jal",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let j = parse_format_j(inst);
            let pc = emu.get_pc();
            emu.set_reg(j.rd, pc.wrapping_add(4))?;
            let target = pc.wrapping_add(j.imm);
            if is_inst_addr_misaligned(target) {
                emu.execption = Some(InstructionAddressMisaligned { addr: target });
                return Ok(());
            }
            emu.set_pc(target);
            Ok(())
        },
    },
    Instruction {
        mask: MASK_JALR,
        identifier: MATCH_JALR,
        name: "jalr",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let pc = emu.get_pc();
            let target = (emu.get_reg(i.rs1)?).wrapping_add(i.imm) & !1u64;
            if is_inst_addr_misaligned(target) {
                emu.execption = Some(InstructionAddressMisaligned { addr: target });
                return Ok(());
            }
            emu.set_pc(target);
            emu.set_reg(i.rd, pc.wrapping_add(4))
        },
    },
    Instruction {
        mask: MASK_BEQ,
        identifier: MATCH_BEQ,
        name: "beq",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let b = parse_format_b(inst);
            let lhs = emu.get_reg(b.rs1)?;
            let rhs = emu.get_reg(b.rs2)?;
            if lhs == rhs {
                let pc = emu.get_pc();
                let target = pc.wrapping_add(b.imm);
                if is_inst_addr_misaligned(target) {
                    emu.execption = Some(InstructionAddressMisaligned { addr: target });
                    return Ok(());
                }
                emu.set_pc(target);
            }
            Ok(())
        }
    },
    Instruction {
        mask: MASK_BNE,
        identifier: MATCH_BNE,
        name: "bne",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let b = parse_format_b(inst);
            let lhs = emu.get_reg(b.rs1)?;
            let rhs = emu.get_reg(b.rs2)?;
            if lhs != rhs {
                let pc = emu.get_pc();
                let target = pc.wrapping_add(b.imm);
                if is_inst_addr_misaligned(target) {
                    emu.execption = Some(InstructionAddressMisaligned { addr: target });
                    return Ok(());
                }
                emu.set_pc(target);
            }
            Ok(())
        }
    },
    Instruction {
        mask: MASK_BLT,
        identifier: MATCH_BLT,
        name: "blt",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let b = parse_format_b(inst);
            let lhs = emu.get_reg(b.rs1)?;
            let rhs = emu.get_reg(b.rs2)?;
            if (lhs as i64) < (rhs as i64) {
                let pc = emu.get_pc();
                let target = pc.wrapping_add(b.imm);
                if is_inst_addr_misaligned(target) {
                    emu.execption = Some(InstructionAddressMisaligned { addr: target });
                    return Ok(());
                }
                emu.set_pc(target);
            }
            Ok(())
        }
    },
    Instruction {
        mask: MASK_BGE,
        identifier: MATCH_BGE,
        name: "bge",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let b = parse_format_b(inst);
            let lhs = emu.get_reg(b.rs1)?;
            let rhs = emu.get_reg(b.rs2)?;
            if (lhs as i64) >= (rhs as i64) {
                let pc = emu.get_pc();
                let target = pc.wrapping_add(b.imm);
                if is_inst_addr_misaligned(target) {
                    emu.execption = Some(InstructionAddressMisaligned { addr: target });
                    return Ok(());
                }
                emu.set_pc(target);
            }
            Ok(())
        }
    },
    Instruction {
        mask: MASK_BLTU,
        identifier: MATCH_BLTU,
        name: "bltu",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let b = parse_format_b(inst);
            let lhs = emu.get_reg(b.rs1)?;
            let rhs = emu.get_reg(b.rs2)?;
            if lhs < rhs {
                let pc = emu.get_pc();
                let target = pc.wrapping_add(b.imm);
                if is_inst_addr_misaligned(target) {
                    emu.execption = Some(InstructionAddressMisaligned { addr: target });
                    return Ok(());
                }
                emu.set_pc(target);
            }
            Ok(())
        }
    },
    Instruction {
        mask: MASK_BGEU,
        identifier: MATCH_BGEU,
        name: "bgeu",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let b = parse_format_b(inst);
            let lhs = emu.get_reg(b.rs1)?;
            let rhs = emu.get_reg(b.rs2)?;
            if lhs >= rhs {
                let pc = emu.get_pc();
                let target = pc.wrapping_add(b.imm);
                if is_inst_addr_misaligned(target) {
                    emu.execption = Some(InstructionAddressMisaligned { addr: target });
                    return Ok(());
                }
                emu.set_pc(target);
            }
            Ok(())
        }
    },
    // Instruction {
    //     mask: MASK_LB,
    //     identifier: MATCH_LB,
    //     name: "lb",
    //     execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            
    //     }
    // },
];
