//! 模拟器核心模块

mod state;
pub mod execute;
mod exception;
mod memory;

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use anyhow::{Result, Context};
use crate::debugger::{Debugger, ExecutionState};
pub use state::State;
pub use execute::Execute;
pub use exception::Exception;
pub use memory::{Memory, MemoryError};

/// 模拟器结构体
pub struct Emulator {
    /// CPU状态（包含内存）
    state: Arc<RwLock<State>>,
    /// 调试器
    debugger: Option<Debugger>,
}

impl Emulator {
    /// 创建新的模拟器实例
    pub fn new(memory_size: usize) -> Result<Self> {
        let state = State::new(memory_size)?;
        Ok(Self {
            state: Arc::new(RwLock::new(state)),
            debugger: None,
        })
    }

    /// 加载ELF文件
    pub fn load_elf(&mut self, path: &str) -> Result<()> {
        use crate::utils::load_elf;
        
        // 获取状态的可变引用
        let mut state = self.state.write()
            .expect("Failed to acquire state write lock");
        
        // 使用工具模块加载ELF
        load_elf(&mut state, path)
            .with_context(|| format!("Failed to load ELF file from '{}'", path))?;
        
        Ok(())
    }

    /// 启用调试模式
    pub fn enable_debug(&mut self, port: u16) -> Result<()> {
        use std::net::SocketAddr;
        
        // 创建调试器并启动GDB服务器
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let mut debugger = Debugger::new(self.state.clone());
        debugger.start_server(addr)?;
        
        // 保存调试器实例
        self.debugger = Some(debugger);
        Ok(())
    }

    /// 运行模拟器
    pub fn run(&mut self) -> Result<()> {
        // 如果启用了调试器，等待调试器准备就绪
        if let Some(debugger) = &self.debugger {
            println!("Waiting for debugger...");
            let control = debugger.get_control();

            loop {
                // 获取当前PC
                let pc = self.state.read()
                    .expect("Failed to acquire state read lock")
                    .get_pc();

                // 检查执行状态
                match *control.state.lock()
                    .expect("Failed to acquire execution control lock") {
                    ExecutionState::Stopped => {
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    },
                    ExecutionState::Running => {
                        // 取指
                        let instruction = {
                            let state = self.state.read()
                                .expect("Failed to acquire state read lock");
                            state.fetch_instruction(pc)
                                .with_context(|| format!("Failed to fetch instruction at PC {:#x}", pc))?
                        };
                        
                        // 创建指令执行器
                        let mut executor = execute::RV64I::new(instruction);
                        
                        // 执行指令
                        {
                            let mut state = self.state.write()
                                .expect("Failed to acquire state write lock");
                            executor.execute(&mut state)
                                .with_context(|| format!("Failed to execute instruction {:#x} at PC {:#x}", 
                                    instruction, pc))?;
                            // 更新PC（默认递增4字节）
                            state.set_pc(pc + 4);
                        }
                    },
                    ExecutionState::Quit => {
                        break Ok(())
                    }
                }
            }
        } else {
            // 无调试器，直接运行
            loop {
                // 获取PC和指令
                let (pc, instruction) = {
                    let state = self.state.read()
                        .expect("Failed to acquire state read lock");
                    let pc = state.get_pc();
                    let instruction = state.fetch_instruction(pc)
                        .with_context(|| format!("Failed to fetch instruction at PC {:#x}", pc))?;
                    (pc, instruction)
                };
                
                // 执行指令
                let mut executor = execute::RV64I::new(instruction);
                let mut state = self.state.write()
                    .expect("Failed to acquire state write lock");
                
                executor.execute(&mut state)
                    .with_context(|| format!("Failed to execute instruction {:#x} at PC {:#x}", 
                        instruction, pc))?;
                state.set_pc(pc + 4);
            }
        }
    }

    /// 获取处理器状态引用
    pub fn get_state(&self) -> Arc<RwLock<State>> {
        self.state.clone()
    }
}
