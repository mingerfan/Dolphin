//! 模拟器核心模块

mod exception;
pub mod execute;
mod gdb;
mod memory;
mod state;

use anyhow::{Context, Ok, Result};

use crate::utils::disasm_riscv64_instruction;
use crate::{const_values, utils::ringbuf::RingBuffer};
pub use exception::Exception;
pub use execute::Execute;
use gdbstub::conn::{Connection, ConnectionExt};
use gdbstub::stub::{SingleThreadStopReason, run_blocking};
use gdbstub::target::Target;
pub use memory::{Memory, MemoryError};
use nohash_hasher::{self, BuildNoHashHasher};
pub use state::State;
use std::collections::HashSet;

type NoHashHashSet<T> = HashSet<T, BuildNoHashHasher<T>>;
/// 模拟器结构体
pub struct Emulator {
    /// CPU状态（包含内存）
    state: State,
    /// 调试器（可选）
    debugger: bool,
    exec_state: ExecState,
    event: Event,
    event_list: RingBuffer<Event>,
    breakpoints: NoHashHashSet<u64>,
    watchpoints: NoHashHashSet<u64>,
}

enum EmuGdbEventLoop {}

impl run_blocking::BlockingEventLoop for EmuGdbEventLoop {
    type Target = Emulator;

    type Connection = Box<dyn ConnectionExt<Error = std::io::Error>>;

    type StopReason = SingleThreadStopReason<u64>;

    fn wait_for_stop_reason(
        target: &mut Self::Target,
        conn: &mut Self::Connection,
    ) -> std::result::Result<
        run_blocking::Event<Self::StopReason>,
        run_blocking::WaitForStopReasonError<
            <Self::Target as Target>::Error,
            <Self::Connection as Connection>::Error,
        >,
    > {
        todo!()
    }

    fn on_interrupt(
        target: &mut Self::Target,
    ) -> std::result::Result<Option<Self::StopReason>, <Self::Target as Target>::Error> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum ExecState {
    #[default]
    Idle,
    Running,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
            event_list: RingBuffer::new(const_values::EVENT_LIST_SIZE),
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
    pub fn enable_debug(&mut self) -> Result<()> {
        self.debugger = true;
        Ok(())
    }

    /// 运行模拟器
    pub fn step(&mut self, n: usize) -> Result<()> {
        self.exec_state = ExecState::Running;
        for _ in 0..n {
            self.event = Event::None; // 重置事件
            // 获取PC和指令
            let (pc, instruction) = {
                let pc = self.state.get_pc();
                let instruction = self
                    .state
                    .fetch_instruction(pc)
                    .with_context(|| format!("无法从PC {:#x} 处读取指令", pc))?;
                (pc, instruction)
            };

            // 执行指令
            let mut executor = execute::RV64I::new(instruction);

            executor.execute(&mut self.state).with_context(|| {
                let instruction_msg =
                    disasm_riscv64_instruction(instruction, pc).unwrap_or("未知指令".to_string());
                format!(
                    "无法执行PC {:#010x} 处的指令 {:#010x} ({})",
                    pc, instruction, instruction_msg
                )
            })?;
            self.state.set_pc(pc + 4);

            // 捕获除了None以外的event，放入事件列表
            if self.debugger && self.event != Event::None {
                self.event_list.push_overwrite(self.event);
            }
        }
        self.exec_state = ExecState::Idle;
        Ok(())
    }

    /// 获取处理器状态引用
    pub fn get_state(&self) -> State {
        self.state.clone()
    }

    pub fn get_state_ref(&self) -> &State {
        &self.state
    }

    // 返回事件列表
    pub fn get_events(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        while let Result::Ok(event) = self.event_list.pop() {
            events.push(event);
        }
        events
    }

    pub fn get_cur_event(&self) -> Event {
        self.event
    }
}
