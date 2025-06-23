//! 模拟器核心模块

mod exception;
pub mod execute;
mod memory;
mod state;
mod gdb;

use anyhow::{Context, Result};

pub use exception::Exception;
pub use execute::Execute;
pub use memory::{Memory, MemoryError};
pub use state::State;
use std::collections::HashSet;
use nohash_hasher::{self, BuildNoHashHasher};


type NoHashHashSet<T> = HashSet<T, BuildNoHashHasher<T>>;
/// 模拟器结构体
pub struct Emulator {
    /// CPU状态（包含内存）
    state: State,
    /// 调试器（可选）
    debugger: bool,
    exec_state: ExecState,
    event: Event,
    breakpoints: NoHashHashSet<u64>,
    watchpoints: NoHashHashSet<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ExecState {
    #[default]
    Idle,
    Running,
    Stopped,
    End,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Event {
    #[default]
    None,
    IncomingData,
    DoneStep,
    Halted,
    Break,
    WatchWrite(u64),
    WatchRead(u64),
}

impl Emulator {
    /// 创建新的模拟器实例
    pub fn new(memory_size: usize) -> Result<Self> {
        let state = State::new(memory_size)?;
        Ok(Self {
            state,
            debugger: false,
            exec_state: ExecState::Idle,
            event: Event::None,
            breakpoints: NoHashHashSet::default(),
            watchpoints: NoHashHashSet::default(),
        })
    }

    /// 加载ELF文件
    pub fn load_elf(&mut self, path: &str) -> Result<()> {
        use crate::utils::load_elf;

        // 使用工具模块加载ELF
        load_elf(&mut self.state, path)
            .with_context(|| format!("无法从 '{}' 加载ELF文件", path))?;

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
                let pc = self.state.get_pc();
                let instruction = self.state
                    .fetch_instruction(pc)
                    .with_context(|| format!("无法从PC {:#x} 处读取指令", pc))?;
                (pc, instruction)
            };

            // 执行指令
            let mut executor = execute::RV64I::new(instruction);

            executor.execute(&mut self.state).with_context(|| {
                format!(
                    "无法执行PC {:#x} 处的指令 {:#x}",
                    pc, instruction
                )
            })?;
            self.state.set_pc(pc + 4);
        }
    }

    /// 获取处理器状态引用
    pub fn get_state(&self) -> State {
        self.state.clone()
    }

    pub fn get_state_ref(&self) -> &State {
        &self.state
    }
}
