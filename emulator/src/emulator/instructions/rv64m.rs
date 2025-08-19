use std::ops::Mul;
use std::u64;

use crate::emulator::Emulator;
use crate::emulator::instructions::parse_format_r;
use crate::utils::bit_utils::BitSlice;

use super::Instruction;
use super::insts::*;

// ┌───────────────────────┬──────────┬─────────┬──────────┬───────────┬──────────┬──────────┐
// │ Condition             │ Dividend │  Dvisor │  DIVU[W] │   REMU[W] │   DIV[W ]│  REM[W]  │
// ├───────────────────────┼──────────┼─────────┼──────────┼───────────┼──────────┼──────────┤
// │ Division by zero      │    X     │    0    │   2^L-1  │     X     │    -1    │    X     │
// ├───────────────────────┼──────────┼─────────┼──────────┼───────────┼──────────┼──────────┤
// │ Overflow (signed only)│ -2^(L-1) │   -1    │     -    │      -    │  -2^(L-1)│    0     │
// └───────────────────────┴──────────┴─────────┴──────────┴───────────┴──────────┴──────────┘
//  Table 7.1: Semantics for division by zero and division overflow. L is the width of the operation in
//  bits: XLEN for DIV[U] and REM[U], or 32 for DIV[U]W and REM[U]W.

pub const RV_M: &[Instruction] = &[
    Instruction {
        mask: MASK_MUL,
        identifier: MATCH_MUL,
        name: "mul",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?;
            let rhs = emu.get_reg(r.rs2)?;
            let res = lhs.wrapping_mul(rhs);
            emu.set_reg(r.rd, res.bit_range(0..64))
        },
    },
    Instruction {
        mask: MASK_MULH,
        identifier: MATCH_MULH,
        name: "mulh",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)? as i64 as i128;
            let rhs = emu.get_reg(r.rs2)? as i64 as i128;
            let res = lhs.mul(rhs);
            emu.set_reg(r.rd, (res >> 64) as u64)
        },
    },
    Instruction {
        mask: MASK_MULHSU,
        identifier: MATCH_MULHSU,
        name: "mulhsu",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)? as i64 as i128;
            let rhs = emu.get_reg(r.rs2)? as u128;
            let res = lhs.mul(rhs as i128);
            emu.set_reg(r.rd, (res >> 64) as u64)
        },
    },
    Instruction {
        mask: MASK_MULHU,
        identifier: MATCH_MULHU,
        name: "mulhu",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)? as u128;
            let rhs = emu.get_reg(r.rs2)? as u128;
            let res = lhs.mul(rhs);
            emu.set_reg(r.rd, (res >> 64) as u64)
        },
    },
    Instruction {
        mask: MASK_DIV,
        identifier: MATCH_DIV,
        name: "div",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)? as i64;
            let rhs = emu.get_reg(r.rs2)? as i64;
            if rhs == 0 {
                return emu.set_reg(r.rd, -1i64 as u64);
            }
            let res = lhs.wrapping_div(rhs);
            emu.set_reg(r.rd, res as u64)
        },
    },
    Instruction {
        mask: MASK_DIVU,
        identifier: MATCH_DIVU,
        name: "divu",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?;
            let rhs = emu.get_reg(r.rs2)?;
            if rhs == 0 {
                return emu.set_reg(r.rd, u64::MAX);
            }
            let res = lhs.wrapping_div(rhs);
            emu.set_reg(r.rd, res)
        },
    },
    Instruction {
        mask: MASK_REM,
        identifier: MATCH_REM,
        name: "rem",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)? as i64;
            let rhs = emu.get_reg(r.rs2)? as i64;
            if rhs == 0 {
                return emu.set_reg(r.rd, lhs as u64);
            }
            let res = lhs.wrapping_rem(rhs);
            emu.set_reg(r.rd, res as u64)
        },
    },
    Instruction {
        mask: MASK_REMU,
        identifier: MATCH_REMU,
        name: "remu",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?;
            let rhs = emu.get_reg(r.rs2)?;
            if rhs == 0 {
                return emu.set_reg(r.rd, lhs);
            }
            let res = lhs.wrapping_rem(rhs);
            emu.set_reg(r.rd, res)
        },
    },
    Instruction {
        mask: MASK_MULW,
        identifier: MATCH_MULW,
        name: "mulw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?.bit_range(0..32) as i32;
            let rhs = emu.get_reg(r.rs2)?.bit_range(0..32) as i32;
            let res = lhs.wrapping_mul(rhs);
            emu.set_reg(r.rd, res as i64 as u64)
        },
    },
    Instruction {
        mask: MASK_DIVW,
        identifier: MATCH_DIVW,
        name: "divw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?.bit_range(0..32) as i32;
            let rhs = emu.get_reg(r.rs2)?.bit_range(0..32) as i32;
            if rhs == 0 {
                return emu.set_reg(r.rd, -1i64 as u64);
            }
            let res = lhs.wrapping_div(rhs);
            emu.set_reg(r.rd, res as i64 as u64)
        },
    },
    Instruction {
        mask: MASK_DIVUW,
        identifier: MATCH_DIVUW,
        name: "divuw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?.bit_range(0..32) as u32;
            let rhs = emu.get_reg(r.rs2)?.bit_range(0..32) as u32;
            if rhs == 0 {
                return emu.set_reg(r.rd, u64::MAX);
            }
            let res = lhs.wrapping_div(rhs);
            emu.set_reg(r.rd, res as i32 as i64 as u64)
        },
    },
    Instruction {
        mask: MASK_REMW,
        identifier: MATCH_REMW,
        name: "remw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?.bit_range(0..32) as i32;
            let rhs = emu.get_reg(r.rs2)?.bit_range(0..32) as i32;
            if rhs == 0 {
                return emu.set_reg(r.rd, rhs as i64 as u64);
            }
            let res = lhs.wrapping_rem(rhs);
            emu.set_reg(r.rd, res as i64 as u64)
        },
    },
    Instruction {
        mask: MASK_REMUW,
        identifier: MATCH_REMUW,
        name: "remuw",
        execute: |emu: &mut Emulator, inst: u32, _pc: u64| {
            let r = parse_format_r(inst);
            let lhs = emu.get_reg(r.rs1)?.bit_range(0..32) as u32;
            let rhs = emu.get_reg(r.rs2)?.bit_range(0..32) as u32;
            if rhs == 0 {
                return emu.set_reg(r.rd, rhs as i32 as i64 as u64);
            }
            let res = lhs.wrapping_rem(rhs);
            emu.set_reg(r.rd, res as i32 as i64 as u64)
        },
    },
];
