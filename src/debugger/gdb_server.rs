//! GDB远程调试服务器实现

use anyhow::Result;
use gdbstub::arch::Arch;
use gdbstub::common::Signal;
use gdbstub::conn::ConnectionExt;
use gdbstub::stub::{run_blocking, SingleThreadStopReason};
use gdbstub::target::ext::base::singlethread::{
    SingleThreadBase, SingleThreadResume, SingleThreadSingleStep,
};
use gdbstub::target::{Target, TargetError, TargetResult};
use gdbstub_arch::riscv::Riscv64;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, RwLock};

use super::{ExecutionControl, ExecutionState};
use crate::emulator::{Execute, State};

/// RISC-V调试目标
pub struct RiscvTarget {
    /// CPU状态
    state: Arc<RwLock<State>>,
    /// 执行控制
    control: ExecutionControl,
    /// 当前PC
    current_pc: u64,
}

impl RiscvTarget {
    /// 创建新的调试目标
    pub fn new(state: Arc<RwLock<State>>, control: ExecutionControl) -> Self {
        Self {
            state,
            control,
            current_pc: 0,
        }
    }

    pub fn quit(&mut self) {
        *self.control.state.lock().expect("Failed to lock execution state mutex") = ExecutionState::Stopped;
    }
}

impl Target for RiscvTarget {
    type Error = anyhow::Error;
    type Arch = Riscv64;

    fn base_ops(&mut self) -> gdbstub::target::ext::base::BaseOps<'_, Self::Arch, Self::Error> {
        gdbstub::target::ext::base::BaseOps::SingleThread(self)
    }

    fn guard_rail_implicit_sw_breakpoints(&self) -> bool {
        true
    }
}

impl SingleThreadBase for RiscvTarget {
    fn read_registers(
        &mut self,
        regs: &mut <Self::Arch as Arch>::Registers,
    ) -> TargetResult<(), Self> {
        let state = self.state.read().expect("Failed to acquire state read lock");

        // 复制通用寄存器 (x0-x31)
        for i in 0..32 {
            regs.x[i] = state.get_reg(i).map_err(|e| {
                tracing::error!("Failed to read register x{}: {}", i, e);
                TargetError::NonFatal
            })?;
        }
        
        // 设置程序计数器
        regs.pc = state.get_pc();

        Ok(())
    }

    fn write_registers(
        &mut self,
        regs: &<Self::Arch as Arch>::Registers,
    ) -> TargetResult<(), Self> {
        let mut state = self.state.write().expect("Failed to acquire state write lock");

        // 写入通用寄存器
        for i in 0..32 {
            state.set_reg(i, regs.x[i]).map_err(|e| {
                tracing::error!("Failed to write register x{}: {}", i, e);
                TargetError::NonFatal
            })?;
        }

        // 写入程序计数器
        state.set_pc(regs.pc);
        self.current_pc = regs.pc;

        Ok(())
    }

    fn read_addrs(&mut self, start_addr: u64, data: &mut [u8]) -> TargetResult<usize, Self> {
        let state = self.state.read().expect("Failed to acquire state read lock");
        let mem = state.read_memory(start_addr, data.len())
            .map_err(|e| {
                tracing::error!("Memory read error at {:#x}: {}", start_addr, e);
                TargetError::NonFatal
            })?;
        data.copy_from_slice(&mem);
        Ok(mem.len())
    }

    fn write_addrs(&mut self, start_addr: u64, data: &[u8]) -> TargetResult<(), Self> {
        let mut state = self.state.write().expect("Failed to acquire state write lock");
        state.write_memory(start_addr, data)
            .map_err(|e| {
                tracing::error!("Memory write error at {:#x}: {}", start_addr, e);
                TargetError::NonFatal
            })?;
        Ok(())
    }
}

impl SingleThreadResume for RiscvTarget {
    fn resume(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        // 设置为运行状态
        *self.control.state.lock().expect("Failed to lock execution state mutex") = ExecutionState::Running;
        Ok(())
    }

    fn support_single_step(
        &mut self,
    ) -> Option<&mut dyn SingleThreadSingleStep<Arch = Self::Arch, Error = Self::Error>> {
        Some(self)
    }
}

impl SingleThreadSingleStep for RiscvTarget {
    fn step(&mut self, _signal: Option<Signal>) -> Result<(), Self::Error> {
        // 设置为停止状态
        *self.control.state.lock().expect("Failed to lock execution state mutex") = ExecutionState::Stopped;

        // 执行一条指令
        let mut state = self.state.write().expect("Failed to acquire state write lock");
        let pc = state.get_pc();
        let instruction = state.fetch_instruction(pc).map_err(|e| {
            tracing::error!("Failed to fetch instruction at {:#x}: {}", pc, e);
            anyhow::anyhow!("Instruction fetch failed: {}", e)
        })?;

        use crate::emulator::execute::RV64I;
        let mut executor = RV64I::new(instruction);
        executor.execute(&mut state)?;

        state.set_pc(pc + 4);
        Ok(())
    }
}


enum MyGdbBlockingEventLoop {}

impl run_blocking::BlockingEventLoop for MyGdbBlockingEventLoop {
    type Target = RiscvTarget;
    type Connection = TcpStream;

    type StopReason = SingleThreadStopReason<u64>;

    fn wait_for_stop_reason(
        target: &mut Self::Target,
        conn: &mut Self::Connection,
    ) -> std::result::Result<
        run_blocking::Event<Self::StopReason>,
        run_blocking::WaitForStopReasonError<
            <Self::Target as Target>::Error,
            <Self::Connection as gdbstub::conn::Connection>::Error,
        >,
    > {
        let event = run_blocking::Event::TargetStopped(
            SingleThreadStopReason::Signal(Signal::SIG100).into(),
        );
        Ok(event)
    }

    fn on_interrupt(
        target: &mut Self::Target,
    ) -> std::result::Result<Option<Self::StopReason>, <Self::Target as Target>::Error> {
        target.quit();

        Ok(Some(SingleThreadStopReason::Signal(Signal::SIGINT).into()))
    }
}

/// GDB服务器
pub struct GdbServer {
    /// 调试目标
    target: RiscvTarget,
}

impl GdbServer {
    /// 创建新的GDB服务器
    pub fn new(state: Arc<RwLock<State>>, control: ExecutionControl) -> Self {
        Self {
            target: RiscvTarget::new(state, control),
        }
    }

    /// 启动服务器
    pub fn start(&mut self, port: u16) -> Result<()> {
        use gdbstub::stub::GdbStub;
        use std::net::TcpListener;

        // 创建TCP监听器
        let listener = TcpListener::bind(format!("localhost:{}", port))?;
        println!("GDB server listening on localhost:{}", port);

        // 等待客户端连接
        let (stream, addr) = listener.accept()?;
        println!("Client connected from {}", addr);

        // 创建GDB存根并运行
        let gdb = GdbStub::new(stream);

        match gdb.run_blocking::<MyGdbBlockingEventLoop>(&mut self.target) {
            Ok(reason) => {
                println!("GDB session ended: {:?}", reason);
            }
            Err(err) => {
                eprintln!("GDB session error: {}", err);
            }
        }

        Ok(())
    }

    /// 关闭服务器
    pub fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}
