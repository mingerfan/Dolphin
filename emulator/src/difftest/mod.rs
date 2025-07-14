use std::fmt::Display;

use rv64emu::{self, rv64core::cpu_core::{CpuCore, CpuState}};

use crate::emulator::Emulator;

pub enum DiffMode {
    Dut,
    Reference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffState {
    pub reg: [u64; 32],
    pub pc: u64,
}

impl Display for DiffState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PC: {:016x}\n", self.pc)?;
        for i in 0..32 {
            write!(f, "x{:02}: {:016x}\n", i, self.reg[i])?;
        }
        Ok(())
    }
}

pub trait Difftest {
    fn init(&mut self);
    #[allow(unused)]
    fn mode(&self) -> DiffMode;
    fn self_state(&self) -> DiffState;
    fn step(&mut self) -> bool;
    fn set_regs(&mut self, regs: [u64; 32]);
    fn set_pc(&mut self, pc: u64);
    fn get_mem(&mut self, addr: u64, size: usize) -> u64;
    fn set_mem(&mut self, addr: u64, data: u64, len: usize);
}

impl Difftest for Emulator {
    fn init(&mut self) {}

    fn mode(&self) -> DiffMode {
        DiffMode::Dut
    }

    fn self_state(&self) -> DiffState {
        DiffState {
            reg: self.get_regs().clone(),
            pc: self.get_pc(),
        }
    }

    fn step(&mut self) -> bool {
        self.steps(1).is_ok()
    }

    fn set_regs(&mut self, regs: [u64; 32]) {
        for i in 0..32 {
            let _ = self.set_reg(i, regs[i as usize]);
        }
    }

    fn set_pc(&mut self, pc: u64) {
        self.set_pc(pc);
    }

    fn get_mem(&mut self, addr: u64, size: usize) -> u64 {
        let mut data = 0u64.to_le_bytes();
        data[..size].copy_from_slice(
            &self.read_memory(addr, size).unwrap()[addr as usize..addr as usize + size],
        );
        u64::from_le_bytes(data)
    }

    fn set_mem(&mut self, addr: u64, data: u64, len: usize) {
        let data = data.to_le_bytes();
        self.write_memory(addr, &data[..len]).unwrap();
    }
}

impl Difftest for CpuCore {
    fn init(&mut self) {
        self.cpu_state = CpuState::Running;
    }

    fn mode(&self) -> DiffMode {
        DiffMode::Reference
    }

    fn self_state(&self) -> DiffState {
        let mut regs: Vec<u64> = Vec::new();
        for i in 0..32 {
            regs.push(self.gpr.read(i as u64));
        }
        DiffState {
            reg: regs.try_into().unwrap(),
            pc: self.pc,
        }
    }

    fn step(&mut self) -> bool {
        self.execute(1);
        true
    }

    fn set_regs(&mut self, regs: [u64; 32]) {
        for i in 0..32 {
            self.gpr.write(i, regs[i as usize]);
        }
    }

    fn set_pc(&mut self, pc: u64) {
        self.npc = pc;
    }

    fn get_mem(&mut self, addr: u64, size: usize) -> u64 {
        <CpuCore as rv64emu::difftest::difftest_trait::Difftest>::get_mem(self, addr, size)
    }

    fn set_mem(&mut self, addr: u64, data: u64, len: usize) {
        <CpuCore as rv64emu::difftest::difftest_trait::Difftest>::set_mem(self, addr, data, len);
    }
}
