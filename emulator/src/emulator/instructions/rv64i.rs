use crate::emulator::{Emulator, Exception::*, state::Event};

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
        execute: |emu: &mut Emulator, inst: u32, pc: u64| {
            let u = parse_format_u(inst);
            emu.set_reg(u.rd, pc.wrapping_add(u.imm))
        },
    },
    Instruction {
        mask: MASK_JAL,
        identifier: MATCH_JAL,
        name: "jal",
        execute: |emu: &mut Emulator, inst: u32, pc: u64| {
            let j = parse_format_j(inst);
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
        execute: |emu: &mut Emulator, inst: u32, pc: u64| {
            let i = parse_format_i(inst);
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
        execute: |emu: &mut Emulator, inst: u32, pc: u64| {
            let b = parse_format_b(inst);
            let lhs = emu.get_reg(b.rs1)?;
            let rhs = emu.get_reg(b.rs2)?;
            if lhs == rhs {
                let target = pc.wrapping_add(b.imm);
                if is_inst_addr_misaligned(target) {
                    emu.execption = Some(InstructionAddressMisaligned { addr: target });
                    return Ok(());
                }
                emu.set_pc(target);
            }
            Ok(())
        },
    },
    Instruction {
        mask: MASK_BNE,
        identifier: MATCH_BNE,
        name: "bne",
        execute: |emu: &mut Emulator, inst: u32, pc: u64| {
            let b = parse_format_b(inst);
            let lhs = emu.get_reg(b.rs1)?;
            let rhs = emu.get_reg(b.rs2)?;
            if lhs != rhs {
                let target = pc.wrapping_add(b.imm);
                if is_inst_addr_misaligned(target) {
                    emu.execption = Some(InstructionAddressMisaligned { addr: target });
                    return Ok(());
                }
                emu.set_pc(target);
            }
            Ok(())
        },
    },
    Instruction {
        mask: MASK_BLT,
        identifier: MATCH_BLT,
        name: "blt",
        execute: |emu: &mut Emulator, inst: u32, pc: u64| {
            let b = parse_format_b(inst);
            let lhs = emu.get_reg(b.rs1)?;
            let rhs = emu.get_reg(b.rs2)?;
            if (lhs as i64) < (rhs as i64) {
                let target = pc.wrapping_add(b.imm);
                if is_inst_addr_misaligned(target) {
                    emu.execption = Some(InstructionAddressMisaligned { addr: target });
                    return Ok(());
                }
                emu.set_pc(target);
            }
            Ok(())
        },
    },
    Instruction {
        mask: MASK_BGE,
        identifier: MATCH_BGE,
        name: "bge",
        execute: |emu: &mut Emulator, inst: u32, pc: u64| {
            let b = parse_format_b(inst);
            let lhs = emu.get_reg(b.rs1)?;
            let rhs = emu.get_reg(b.rs2)?;
            if (lhs as i64) >= (rhs as i64) {
                let target = pc.wrapping_add(b.imm);
                if is_inst_addr_misaligned(target) {
                    emu.execption = Some(InstructionAddressMisaligned { addr: target });
                    return Ok(());
                }
                emu.set_pc(target);
            }
            Ok(())
        },
    },
    Instruction {
        mask: MASK_BLTU,
        identifier: MATCH_BLTU,
        name: "bltu",
        execute: |emu: &mut Emulator, inst: u32, pc: u64| {
            let b = parse_format_b(inst);
            let lhs = emu.get_reg(b.rs1)?;
            let rhs = emu.get_reg(b.rs2)?;
            if lhs < rhs {
                let target = pc.wrapping_add(b.imm);
                if is_inst_addr_misaligned(target) {
                    emu.execption = Some(InstructionAddressMisaligned { addr: target });
                    return Ok(());
                }
                emu.set_pc(target);
            }
            Ok(())
        },
    },
    Instruction {
        mask: MASK_BGEU,
        identifier: MATCH_BGEU,
        name: "bgeu",
        execute: |emu: &mut Emulator, inst: u32, pc: u64| {
            let b = parse_format_b(inst);
            let lhs = emu.get_reg(b.rs1)?;
            let rhs = emu.get_reg(b.rs2)?;
            if lhs >= rhs {
                let target = pc.wrapping_add(b.imm);
                if is_inst_addr_misaligned(target) {
                    emu.execption = Some(InstructionAddressMisaligned { addr: target });
                    return Ok(());
                }
                emu.set_pc(target);
            }
            Ok(())
        },
    },
    Instruction {
        mask: MASK_LB,
        identifier: MATCH_LB,
        name: "lb",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let addr = emu.get_reg(i.rs1)?.wrapping_add(i.imm);
            let raw = emu.state.memory.read_byte(addr)?;
            let value = sign_extend_64(raw as u64, 8);
            emu.set_reg(i.rd, value)
        },
    },
    Instruction {
        mask: MASK_LH,
        identifier: MATCH_LH,
        name: "lh",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let addr = emu.get_reg(i.rs1)?.wrapping_add(i.imm);
            let raw = emu.state.memory.read_halfword(addr)?;
            let value = sign_extend_64(raw as u64, 16);
            emu.set_reg(i.rd, value)
        },
    },
    Instruction {
        mask: MASK_LW,
        identifier: MATCH_LW,
        name: "lw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let addr = emu.get_reg(i.rs1)?.wrapping_add(i.imm);
            let raw = emu.state.memory.read_word(addr)?;
            let value = sign_extend_64(raw as u64, 32);
            emu.set_reg(i.rd, value)
        },
    },
    Instruction {
        mask: MASK_LBU,
        identifier: MATCH_LBU,
        name: "lbu",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let addr = emu.get_reg(i.rs1)?.wrapping_add(i.imm);
            let raw = emu.state.memory.read_byte(addr)?;
            emu.set_reg(i.rd, raw as u64)
        },
    },
    Instruction {
        mask: MASK_LHU,
        identifier: MATCH_LHU,
        name: "lhu",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let addr = emu.get_reg(i.rs1)?.wrapping_add(i.imm);
            let raw = emu.state.memory.read_halfword(addr)?;
            emu.set_reg(i.rd, raw as u64)
        },
    },
    Instruction {
        mask: MASK_SB,
        identifier: MATCH_SB,
        name: "sb",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let s = parse_format_s(inst);
            let addr = emu.get_reg(s.rs1)?.wrapping_add(s.imm);
            let value = emu.get_reg(s.rs2)?;
            emu.state.memory.write_byte(addr, (value & 0xFF) as u8)?;
            Ok(())
        },
    },
    Instruction {
        mask: MASK_SH,
        identifier: MATCH_SH,
        name: "sh",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let s = parse_format_s(inst);
            let addr = emu.get_reg(s.rs1)?.wrapping_add(s.imm);
            let value = emu.get_reg(s.rs2)?;
            emu.state
                .memory
                .write_halfword(addr, (value & 0xFFFF) as u16)?;
            Ok(())
        },
    },
    Instruction {
        mask: MASK_SW,
        identifier: MATCH_SW,
        name: "sw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let s = parse_format_s(inst);
            let addr = emu.get_reg(s.rs1)?.wrapping_add(s.imm);
            let value = emu.get_reg(s.rs2)?;
            emu.state
                .memory
                .write_word(addr, (value & 0xFFFFFFFF) as u32)?;
            Ok(())
        },
    },
    Instruction {
        mask: MASK_ADDI,
        identifier: MATCH_ADDI,
        name: "addi",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let lhs = emu.get_reg(i.rs1)?;
            tracing::info!("i.rs1: {}, i.imm: 0x{:x}", i.rs1, i.imm);
            emu.set_reg(i.rd, lhs.wrapping_add(i.imm))
        },
    },
    Instruction {
        mask: MASK_SLTI,
        identifier: MATCH_SLTI,
        name: "slti",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let lhs = emu.get_reg(i.rs1)?;
            let result = if (lhs as i64) < (i.imm as i64) { 1 } else { 0 };
            emu.set_reg(i.rd, result as u64)
        },
    },
    Instruction {
        mask: MASK_SLTIU,
        identifier: MATCH_SLTIU,
        name: "sltiu",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let lhs = emu.get_reg(i.rs1)?;
            let result = if lhs < i.imm { 1 } else { 0 };
            emu.set_reg(i.rd, result as u64)
        },
    },
    Instruction {
        mask: MASK_XORI,
        identifier: MATCH_XORI,
        name: "xori",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let lhs = emu.get_reg(i.rs1)?;
            emu.set_reg(i.rd, lhs ^ i.imm)
        },
    },
    Instruction {
        mask: MASK_ORI,
        identifier: MATCH_ORI,
        name: "ori",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let lhs = emu.get_reg(i.rs1)?;
            emu.set_reg(i.rd, lhs | i.imm)
        },
    },
    Instruction {
        mask: MASK_ANDI,
        identifier: MATCH_ANDI,
        name: "andi",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let lhs = emu.get_reg(i.rs1)?;
            emu.set_reg(i.rd, lhs & i.imm)
        },
    },
    Instruction {
        mask: MASK_SLLI,
        identifier: MATCH_SLLI,
        name: "slli",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let lhs = emu.get_reg(i.rs1)?;
            let shamt = (i.imm & 0x3F) as u64; // 确保移位量在0-63范围内
            emu.set_reg(i.rd, lhs << shamt)
        },
    },
    Instruction {
        mask: MASK_SRLI,
        identifier: MATCH_SRLI,
        name: "srli",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let lhs = emu.get_reg(i.rs1)?;
            let shamt = (i.imm & 0x3F) as u64; // 确保移位量在0-63范围内
            emu.set_reg(i.rd, lhs >> shamt)
        },
    },
    Instruction {
        mask: MASK_SRAI,
        identifier: MATCH_SRAI,
        name: "srai",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let lhs = emu.get_reg(i.rs1)?;
            let shamt = (i.imm & 0x3F) as u64; // 确保移位量在0-63范围内
            emu.set_reg(i.rd, (lhs as i64 >> shamt) as u64)
        },
    },
    Instruction {
        mask: MASK_ADD,
        identifier: MATCH_ADD,
        name: "add",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?;
            let rhs = emu.get_reg(r.rs2)?;
            emu.set_reg(r.rd, lhs.wrapping_add(rhs))
        },
    },
    Instruction {
        mask: MASK_SUB,
        identifier: MATCH_SUB,
        name: "sub",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?;
            let rhs = emu.get_reg(r.rs2)?;
            emu.set_reg(r.rd, lhs.wrapping_sub(rhs))
        },
    },
    Instruction {
        mask: MASK_SLL,
        identifier: MATCH_SLL,
        name: "sll",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?;
            let rhs = emu.get_reg(r.rs2)?;
            let shamt = (rhs & 0x3F) as u64; // 确保移位量在0-63范围内
            emu.set_reg(r.rd, lhs << shamt)
        },
    },
    Instruction {
        mask: MASK_SLT,
        identifier: MATCH_SLT,
        name: "slt",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?;
            let rhs = emu.get_reg(r.rs2)?;
            let result = if (lhs as i64) < (rhs as i64) { 1 } else { 0 };
            emu.set_reg(r.rd, result as u64)
        },
    },
    Instruction {
        mask: MASK_SLTU,
        identifier: MATCH_SLTU,
        name: "sltu",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?;
            let rhs = emu.get_reg(r.rs2)?;
            let result = if lhs < rhs { 1 } else { 0 };
            emu.set_reg(r.rd, result as u64)
        },
    },
    Instruction {
        mask: MASK_XOR,
        identifier: MATCH_XOR,
        name: "xor",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?;
            let rhs = emu.get_reg(r.rs2)?;
            emu.set_reg(r.rd, lhs ^ rhs)
        },
    },
    Instruction {
        mask: MASK_SRL,
        identifier: MATCH_SRL,
        name: "srl",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?;
            let rhs = emu.get_reg(r.rs2)?;
            let shamt = (rhs & 0x3F) as u64; // 确保移位量在0-63范围内
            emu.set_reg(r.rd, lhs >> shamt)
        },
    },
    Instruction {
        mask: MASK_SRA,
        identifier: MATCH_SRA,
        name: "sra",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?;
            let rhs = emu.get_reg(r.rs2)?;
            let shamt = (rhs & 0x3F) as u64; // 确保移位量在0-63范围内
            emu.set_reg(r.rd, (lhs as i64 >> shamt) as u64)
        },
    },
    Instruction {
        mask: MASK_OR,
        identifier: MATCH_OR,
        name: "or",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?;
            let rhs = emu.get_reg(r.rs2)?;
            emu.set_reg(r.rd, lhs | rhs)
        },
    },
    Instruction {
        mask: MASK_AND,
        identifier: MATCH_AND,
        name: "and",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?;
            let rhs = emu.get_reg(r.rs2)?;
            emu.set_reg(r.rd, lhs & rhs)
        },
    },
    Instruction {
        mask: MASK_FENCE,
        identifier: MATCH_FENCE,
        name: "fence",
        execute: |_emu: &mut Emulator, _inst: u32, _pc: u64| {
            // FENCE 指令不做任何操作
            tracing::warn!("执行FENCE指令, 但是目前不做任何操作");
            todo!("Implement FENCE handling");
        },
    },
    Instruction {
        mask: MASK_ECALL,
        identifier: MATCH_ECALL,
        name: "ecall",
        execute: |_emu: &mut Emulator, _inst: u32, _pc: u64| {
            // 处理 ECALL 指令
            tracing::warn!("执行 ECALL 指令, 但目前未实现系统调用处理");
            todo!("Implement ECALL handling");
            // Ok(())
        },
    },
    Instruction {
        mask: MASK_EBREAK,
        identifier: MATCH_EBREAK,
        name: "ebreak",
        execute: |emu: &mut Emulator, _inst: u32, _pc: u64| {
            emu.event = Event::Halted;
            tracing::info!("执行 EBREAK 指令, 触发 CPU 停止事件");
            Ok(())
        },
    },
    Instruction {
        mask: MASK_ADDIW,
        identifier: MATCH_ADDIW,
        name: "addiw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let lhs = emu.get_reg(i.rs1)?;
            let result = lhs.wrapping_add(i.imm).bit_range(0..32);
            emu.set_reg(i.rd, sign_extend_64(result, 32))
        },
    },
    Instruction {
        mask: MASK_SLLIW,
        identifier: MATCH_SLLIW,
        name: "slliw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let lhs = emu.get_reg(i.rs1)?;
            let shamt = (i.imm & 0x1F) as u64; // 确保移位量在0-31范围内
            let result = (lhs << shamt).bit_range(0..32);
            emu.set_reg(i.rd, sign_extend_64(result, 32))
        },
    },
    Instruction {
        mask: MASK_SRLIW,
        identifier: MATCH_SRLIW,
        name: "srliw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let lhs = emu.get_reg(i.rs1)?.bit_range(0..32);
            let shamt = (i.imm & 0x1F) as u64;
            let result = lhs >> shamt;
            emu.set_reg(i.rd, sign_extend_64(result, 32))
        },
    },
    Instruction {
        mask: MASK_SRAIW,
        identifier: MATCH_SRAIW,
        name: "sraiw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let lhs = emu.get_reg(i.rs1)?.bit_range(0..32);
            let shamt = (i.imm & 0x1F) as u64;
            let result = lhs as i32 >> shamt;
            emu.set_reg(i.rd, sign_extend_64(result as u32 as u64, 32))
        },
    },
    Instruction {
        mask: MASK_ADDW,
        identifier: MATCH_ADDW,
        name: "addw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?.bit_range(0..32);
            let rhs = emu.get_reg(r.rs2)?.bit_range(0..32);
            let result = lhs.wrapping_add(rhs);
            emu.set_reg(r.rd, sign_extend_64(result, 32))
        },
    },
    Instruction {
        mask: MASK_SUBW,
        identifier: MATCH_SUBW,
        name: "subw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?.bit_range(0..32);
            let rhs = emu.get_reg(r.rs2)?.bit_range(0..32);
            let result = lhs.wrapping_sub(rhs);
            emu.set_reg(r.rd, sign_extend_64(result, 32))
        },
    },
    Instruction {
        mask: MASK_SLLW,
        identifier: MATCH_SLLW,
        name: "sllw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?;
            let rhs = emu.get_reg(r.rs2)?;
            let shamt = (rhs & 0x1F) as u64;
            let result = (lhs << shamt).bit_range(0..32);
            emu.set_reg(r.rd, sign_extend_64(result, 32))
        },
    },
    Instruction {
        mask: MASK_SRLW,
        identifier: MATCH_SRLW,
        name: "srlw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?.bit_range(0..32);
            let rhs = emu.get_reg(r.rs2)?;
            let shamt = (rhs & 0x1F) as u64;
            let result = (lhs >> shamt).bit_range(0..32);
            emu.set_reg(r.rd, sign_extend_64(result, 32))
        },
    },
    Instruction {
        mask: MASK_SRAW,
        identifier: MATCH_SRAW,
        name: "sraw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?.bit_range(0..32);
            let rhs = emu.get_reg(r.rs2)?;
            let shamt = (rhs & 0x1F) as u64;
            let result = (lhs as i32 >> shamt) as u32;
            emu.set_reg(r.rd, sign_extend_64(result as u64, 32))
        },
    },
    Instruction {
        mask: MASK_LD,
        identifier: MATCH_LD,
        name: "ld",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let rhs = emu.get_reg(i.rs1)?;
            let addr = rhs.wrapping_add(i.imm);
            let raw = emu.state.memory.read_doubleword(addr)?;
            emu.set_reg(i.rd, raw)
        },
    },
    Instruction {
        mask: MASK_LWU,
        identifier: MATCH_LWU,
        name: "lwu",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let i = parse_format_i(inst);
            let rhs = emu.get_reg(i.rs1)?;
            let addr = rhs.wrapping_add(i.imm);
            let raw = emu.state.memory.read_word(addr)?;
            emu.set_reg(i.rd, raw as u64)
        },
    },
    Instruction {
        mask: MASK_SD,
        identifier: MATCH_SD,
        name: "sd",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let s = parse_format_s(inst);
            let addr = emu.get_reg(s.rs1)?.wrapping_add(s.imm);
            tracing::info!(
                "lhs: 0x{:x}, s.imm: 0x{:x}, addr: 0x{:x}",
                emu.get_reg(s.rs1)?,
                s.imm,
                addr
            );
            let value = emu.get_reg(s.rs2)?;
            emu.state.memory.write_doubleword(addr, value)?;
            Ok(())
        },
    },
];
