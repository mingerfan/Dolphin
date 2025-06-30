//! 模拟器核心模块

mod exception;
pub mod execute;
mod instructions;
pub mod state;

#[cfg(feature = "gdb")] // 条件编译 GDB 模块
pub mod gdb;
#[cfg(feature = "tracer")] // 条件编译追踪器模块
pub mod tracer;

mod memory;

use crate::emulator::instructions::is_compressed;
use crate::utils::disasm_riscv64_instruction;
use crate::{const_values, utils::ringbuf::RingBuffer};
use anyhow::{Context, Result};
pub use exception::Exception;
pub use execute::Execute;

#[cfg(feature = "gdb")] // 条件编译 GDB 模块
pub use gdb::EmuGdbEventLoop;
pub use memory::{Memory, MemoryError};

pub use instructions::InstDecoderArgs;
pub use state::State;
pub use state::{Event, ExecMode, ExecState};

/// 模拟器结构体
pub struct Emulator {
    /// CPU状态（包含内存）
    state: State,
    exec_state: ExecState,
    exec_mode: ExecMode,
    event: Event,
    event_list: RingBuffer<Event>,
    decoder: instructions::InstDecoder,
    #[cfg(feature = "gdb")] // 条件编译 GDB 相关
    gdb_data: gdb::GdbData,
}

impl Emulator {
    /// 创建新的模拟器实例
    pub fn new(args: &crate::Args) -> Result<Self> {
        let state = State::new(args.memory * 1024 * 1024)?;
        let exec_mode = if cfg!(feature = "gdb") {
            ExecMode::Continue // 如果启用了GDB，默认执行模式为连续执行
        } else {
            ExecMode::None // 否则为无执行模式
        };
        Ok(Self {
            state,
            exec_state: ExecState::Idle,
            exec_mode,
            event: Event::None,
            event_list: RingBuffer::new(const_values::EVENT_LIST_SIZE),
            decoder: instructions::InstDecoder::new(&args.inst_decoder_args),
            #[cfg(feature = "gdb")] // 条件编译 GDB 相关
            gdb_data: gdb::GdbData::new(),
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

    #[inline(always)]
    fn step_internal(&mut self) -> Result<()> {
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
        // let mut executor = execute::RV64I::new(instruction);

        // let event = executor.execute(&mut self.state).with_context(|| {
        //     let instruction_msg =
        //         disasm_riscv64_instruction(instruction, pc).unwrap_or("未知指令".to_string());
        //     format!(
        //         "无法执行PC {:#010x} 处的指令 {:#010x} ({}), cpu状态:\n{}",
        //         pc, instruction, instruction_msg, self.state
        //     )
        // })?;
        let inst = self.decoder.fast_path(instruction).with_context(|| {
            let instruction_msg =
                disasm_riscv64_instruction(instruction, pc).unwrap_or("未知指令".to_string());
            format!(
                "无法解码PC {:#010x} 处的指令 {:#010x} ({}), cpu状态:\n{}",
                pc, instruction, instruction_msg, self.state
            )
        })?;

        (inst.execute)(self, instruction, pc).with_context(|| {
            let instruction_msg =
                disasm_riscv64_instruction(instruction, pc).unwrap_or("未知指令".to_string());
            format!(
                "无法执行PC {:#010x} 处的指令 {:#010x} ({}), cpu状态:\n{}",
                pc, instruction, instruction_msg, self.state
            )
        })?;

        if self.event == Event::Halted {
            self.exec_state = ExecState::End; // 结束执行状态
        }
        #[cfg(feature = "tracer")] // 条件编译追踪器相关
        tracer::global_trace(self);

        if is_compressed(instruction) {
            // 如果是压缩指令，PC需要加2
            self.state.set_pc(pc + 2);
        } else {
            // 否则PC加4
            self.state.set_pc(pc + 4);
        }
        Ok(())
    }

    /// 执行单步指令
    #[inline(always)]
    pub fn step(&mut self) -> Result<()> {
        self.exec_state = ExecState::Running;
        self.event = Event::None; // 重置事件

        self.step_internal()?;

        // 捕获除了None以外的event，放入事件列表
        #[cfg(feature = "gdb")] // 条件编译 GDB 相关
        if self.event != Event::None {
            self.event_list.push_overwrite(self.event);
        }

        if self.exec_state != ExecState::End {
            self.exec_state = ExecState::Idle;
        }
        Ok(())
    }

    /// 运行模拟器
    pub fn steps(&mut self, n: usize) -> Result<()> {
        self.exec_state = ExecState::Running;
        for _ in 0..n {
            self.event = Event::None; // 重置事件

            self.step_internal()?;

            // 捕获除了None以外的event，放入事件列表
            #[cfg(feature = "gdb")] // 条件编译 GDB 相关
            if self.event != Event::None {
                self.event_list.push_overwrite(self.event);
            }

            if self.exec_state == ExecState::End {
                break;
            }
        }
        if self.exec_state != ExecState::End {
            self.exec_state = ExecState::Idle;
        }
        Ok(())
    }

    /// 获取处理器状态引用
    #[inline(always)]
    pub fn get_state(&self) -> State {
        self.state.clone()
    }

    #[inline(always)]
    pub fn get_state_ref(&self) -> &State {
        &self.state
    }

    #[inline(always)]
    pub fn get_exec_state(&self) -> ExecState {
        self.exec_state
    }

    #[inline(always)]
    pub fn get_exec_mode(&self) -> ExecMode {
        self.exec_mode
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
