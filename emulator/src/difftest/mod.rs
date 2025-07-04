use std::fmt::Display;

use rv64emu::{self, rv64core::cpu_core::CpuCore};

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
}

impl Difftest for Emulator {
    fn init(&mut self) {

    }

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
}

impl Difftest for CpuCore {
    fn init(&mut self) {

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
}
