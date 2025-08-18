//! 模拟器核心模块

mod exception;
mod instructions;
pub mod state;

#[cfg(feature = "gdb")] // 条件编译 GDB 模块
pub mod gdb;
#[cfg(feature = "tracer")] // 条件编译追踪器模块
pub mod tracer;

mod memory;

use std::path::PathBuf;
use std::rc::Rc;

use crate::emulator::instructions::is_compressed;
use crate::utils::disasm_riscv64_instruction;
use crate::{const_values, utils::ringbuf::RingBuffer};
use anyhow::{Context, Result};
pub use exception::Exception;

#[cfg(feature = "gdb")] // 条件编译 GDB 模块
pub use gdb::EmuGdbEventLoop;
pub use memory::{Memory, MemoryError};

#[cfg(feature = "difftest")]
use rv64emu::rv64core::{
    bus::{DeviceType},
    cpu_core::CpuCore
};
pub use state::State;
pub use state::{Event, ExecMode, ExecState};

/// 模拟器结构体
pub struct Emulator {
    /// CPU状态（包含内存）
    state: State,
    exec_state: ExecState,
    exec_mode: ExecMode,
    event: Event,
    execption: Option<Exception>,
    event_list: RingBuffer<Event>,
    decoder: instructions::InstDecoder,
    #[allow(unused)]
    config: Rc<const_values::EmuConfig>, // 模拟器配置
    #[cfg(feature = "gdb")] // 条件编译 GDB 相关
    gdb_data: gdb::GdbData,
    #[cfg(feature = "difftest")] // 条件编译 DiffTest 相关
    ref_emu: rv64emu::rv64core::cpu_core::CpuCore,
}

impl Emulator {
        /// 创建新的模拟器实例
        pub fn new(args: &crate::Args) -> Result<Self> {
        let prj_base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let arg_cfg_path = PathBuf::from(&args.config);
        let config_path = if arg_cfg_path.is_absolute() {
            arg_cfg_path
        } else {
            prj_base.join(&args.config)
        };
        let emu_config = Rc::new(const_values::EmuConfig::new(config_path)?);
        let state = State::new(emu_config.clone())?;
        let exec_mode = if cfg!(feature = "gdb") {
            ExecMode::Continue // 如果启用了GDB，默认执行模式为连续执行
        } else {
            ExecMode::None // 否则为无执行模式
        };
        #[cfg(feature = "difftest")]
        let ref_emu;
        #[cfg(feature = "difftest")]
        {
            use std::{cell::RefCell, rc::Rc};

            use rv64emu::{device::device_trait::DeviceBase, rv64core::{bus, cpu_core}};

            use crate::difftest::Difftest;
            let mut ref_config = rv64emu::config::Config::new();
            ref_config.set_decode_cache_size(1024);
            ref_config.set_mmu_type("bare");
            ref_config.set_isa("rv64imac");
            let bus = Rc::new(RefCell::new(bus::Bus::new()));
            let rc_config = Rc::new(ref_config);
            let mut in_core = cpu_core::CpuCoreBuild::new(bus.clone(), rc_config)
                .with_boot_pc(emu_config.memory.boot_pc)
                .with_smode(false)
                .build();
            let mem = rv64emu::device::device_memory::DeviceMemory::new(emu_config.memory.memory_size * 1024 * 1024);
            let device_name = mem.get_name();
            bus.borrow_mut().add_device(DeviceType {
                start: emu_config.memory.memory_base,
                len: mem.size() as u64,
                instance: Box::new(mem),
                name: device_name,
            });

            in_core.init();
            ref_emu = in_core;
        }

        Ok(Self {
            state,
            exec_state: ExecState::Idle,
            exec_mode,
            event: Event::None,
            execption: None,
            event_list: RingBuffer::new(emu_config.debug.event_list_size),
            decoder: instructions::InstDecoder::new(emu_config.clone()),
            config: emu_config,
            #[cfg(feature = "gdb")] // 条件编译 GDB 相关
            gdb_data: gdb::GdbData::new(),
            #[cfg(feature = "difftest")] // 条件编译 DiffTest 相关
            ref_emu,
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
            self.state.sync_pc();
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

        if is_compressed(instruction) {
            // 如果是压缩指令，PC需要加2
            self.state.set_npc(pc + 2);
        } else {
            // 否则PC加4
            self.state.set_npc(pc + 4);
        }

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

        #[cfg(feature = "difftest")] // 条件编译 DiffTest 相关
        if self.event != Event::Halted {
            use crate::difftest::Difftest;
            tracing::info!("check diff");

            Difftest::step(&mut self.ref_emu);
            let ref_state = self.ref_emu.self_state();
            if ref_state != self.self_state() {
                use anyhow::anyhow;

                return Err(anyhow!(
                    "Failed in difftest check, ref state: {}, self state: {}",
                    ref_state,
                    self.state
                ));
            }
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

            #[cfg(feature = "difftest")] // 条件编译 DiffTest 相关
            {
                use crate::difftest::Difftest;

                Difftest::step(&mut self.ref_emu);
                let ref_state = self.ref_emu.self_state();
                if ref_state != self.self_state() {
                    use anyhow::anyhow;

                    return Err(anyhow!(
                        "Failed in difftest check, ref state: {}, self state: {}",
                        ref_state,
                        self.state
                    ));
                }
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

    #[inline(always)]
    pub fn read_memory(&self, addr: u64, size: usize) -> Result<Vec<u8>> {
        self.state.read_memory(addr, size)
    }

    #[inline(always)]
    pub fn write_memory(&mut self, addr: u64, data: &[u8]) -> Result<()> {
        self.state.write_memory(addr, data)
    }

    #[inline(always)]
    pub fn get_reg(&self, reg: u64) -> Result<u64> {
        self.state.get_reg(reg)
    }

    #[inline(always)]
    pub fn set_reg(&mut self, reg: u64, value: u64) -> Result<()> {
        self.state.set_reg(reg, value)
    }

    #[inline(always)]
    pub fn get_pc(&self) -> u64 {
        self.state.get_pc()
    }

    #[inline(always)]
    pub fn set_npc(&mut self, pc: u64) {
        self.state.set_npc(pc)
    }

    #[inline(always)]
    pub fn sync_pc(&mut self) {
        self.state.sync_pc()
    }

    #[inline(always)]
    pub fn get_regs(&self) -> &[u64; 32] {
        self.state.get_regs()
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

    #[cfg(feature = "difftest")]
    pub fn get_ref_mut(&mut self) -> &mut CpuCore {
        &mut self.ref_emu
    }
}
