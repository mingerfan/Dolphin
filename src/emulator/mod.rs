//! 模拟器核心模块

mod exception;
pub mod execute;
mod memory;
mod state;

use anyhow::{Context, Result};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

pub use exception::Exception;
pub use execute::Execute;
pub use memory::{Memory, MemoryError};
pub use state::State;

/// 模拟器结构体
pub struct Emulator {
    /// CPU状态（包含内存）
    state: Arc<RwLock<State>>,
    /// 调试器（可选）
    debugger: bool,
}

impl Emulator {
    /// 创建新的模拟器实例
    pub fn new(memory_size: usize) -> Result<Self> {
        let state = State::new(memory_size)?;
        Ok(Self {
            state: Arc::new(RwLock::new(state)),
            debugger: false,
        })
    }

    /// 加载ELF文件
    pub fn load_elf(&mut self, path: &str) -> Result<()> {
        use crate::utils::load_elf;

        // 获取状态的可变引用
        let mut state = self
            .state
            .write()
            .expect("Failed to acquire state write lock");

        // 使用工具模块加载ELF
        load_elf(&mut state, path)
            .with_context(|| format!("Failed to load ELF file from '{}'", path))?;

        Ok(())
    }

    /// 启用调试模式
    pub fn enable_debug(&mut self, _port: u16) -> Result<()> {
        self.debugger = true;

        Ok(())
    }

    /// 运行模拟器
    pub fn run(&mut self) -> Result<()> {
        // TODO: 调试器支持

        // 无调试器，直接运行
        loop {
            // 获取PC和指令
            let (pc, instruction) = {
                let state = self
                    .state
                    .read()
                    .expect("Failed to acquire state read lock");
                let pc = state.get_pc();
                let instruction = state
                    .fetch_instruction(pc)
                    .with_context(|| format!("Failed to fetch instruction at PC {:#x}", pc))?;
                (pc, instruction)
            };

            // 执行指令
            let mut executor = execute::RV64I::new(instruction);
            let mut state = self
                .state
                .write()
                .expect("Failed to acquire state write lock");

            executor.execute(&mut state).with_context(|| {
                format!(
                    "Failed to execute instruction {:#x} at PC {:#x}",
                    instruction, pc
                )
            })?;
            state.set_pc(pc + 4);
        }
    }

    /// 获取处理器状态引用
    pub fn get_state(&self) -> Arc<RwLock<State>> {
        self.state.clone()
    }
}
