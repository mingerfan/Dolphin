mod breakpoints;

use std::collections::HashSet;
use nohash_hasher::{self, BuildNoHashHasher};
use crate::emulator::Emulator;
use anyhow::Result;
use gdbstub::target::ext::base::single_register_access::SingleRegisterAccess;
use gdbstub::target::ext::base::singlethread::{
    SingleThreadBase, SingleThreadRangeStepping, SingleThreadResume, SingleThreadSingleStep,
};
use gdbstub::target::{self, Target};
use gdbstub_arch::riscv::reg::id::RiscvRegId;
use std::net::{TcpListener, TcpStream};
use tracing::info;

use gdbstub::common::Signal;
use gdbstub::conn::{Connection, ConnectionExt};
use gdbstub::stub::{SingleThreadStopReason, run_blocking};
use gdbstub::target::ext::breakpoints::WatchKind;
use super::state::{Event, ExecMode, ExecState};

type NoHashHashSet<T> = HashSet<T, BuildNoHashHasher<T>>;

pub struct GdbData {
    pub breakpoints: NoHashHashSet<u64>,
    pub watchpoints: NoHashHashSet<u64>,
}

impl GdbData {
    pub fn new() -> Self {
        Self {
            breakpoints: NoHashHashSet::default(),
            watchpoints: NoHashHashSet::default(),
        }
    }
}

pub enum EmuGdbEventLoop {}

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
        let mode = target.get_exec_mode();
        let mut cnt = match mode {
            ExecMode::Step => 1,
            ExecMode::Continue => usize::MAX,
            ExecMode::RangeStep(start, end) => {
                if target.get_state_ref().get_pc() >= end {
                    return Ok(run_blocking::Event::TargetStopped(
                        SingleThreadStopReason::Exited(0),
                    ));
                }
                (end - start) as usize
            }
            _ => 1, // 默认单步执行
        };
        let mut delay_cycles = 0;
        while target.get_exec_state() != ExecState::End {
            if delay_cycles >= 1000 && conn.peek().is_ok() {
                let byte = conn
                    .read()
                    .map_err(run_blocking::WaitForStopReasonError::Connection)?;
                return Ok(run_blocking::Event::IncomingData(byte));
            }

            match target.step() {
                Ok(_) => match target.event {
                    Event::None => (),
                    Event::Halted => {
                        return Ok(run_blocking::Event::TargetStopped(
                            SingleThreadStopReason::Exited(0),
                        ));
                    }
                    Event::Break => {
                        return Ok(run_blocking::Event::TargetStopped(
                            SingleThreadStopReason::Terminated(Signal::SIGSTOP),
                        ));
                    }
                    Event::WatchWrite(addr) => {
                        return Ok(run_blocking::Event::TargetStopped(
                            SingleThreadStopReason::Watch {
                                tid: (),
                                kind: WatchKind::Write,
                                addr,
                            },
                        ));
                    }
                    Event::WatchRead(addr) => {
                        return Ok(run_blocking::Event::TargetStopped(
                            SingleThreadStopReason::Watch {
                                tid: (),
                                kind: WatchKind::Read,
                                addr,
                            },
                        ));
                    }
                },
                Err(e) => {
                    let error_msg = format!("gdb调试过程中出现执行错误: {}", e);
                    // 打印错误信息
                    tracing::error!("{}", error_msg);
                    tracing::error!("CPU状态:\n{}", target.get_state_ref());
                    return Err(run_blocking::WaitForStopReasonError::Target(error_msg));
                }
            }
            if mode != ExecMode::Continue {
                cnt -= 1;
                if cnt == 0 {
                    return Ok(run_blocking::Event::TargetStopped(
                        SingleThreadStopReason::DoneStep,
                    ));
                }
            }
            if delay_cycles >= 1000 {
                delay_cycles = 0; // 重置延迟计数器
            } else {
                delay_cycles += 1;
            }
        }
        Ok(run_blocking::Event::TargetStopped(
            SingleThreadStopReason::DoneStep,
        ))
    }

    fn on_interrupt(
        _target: &mut Self::Target,
    ) -> std::result::Result<Option<Self::StopReason>, <Self::Target as Target>::Error> {
        Ok(Some(SingleThreadStopReason::Signal(Signal::SIGINT)))
    }
}


pub fn wait_for_tcp(port: u16) -> Result<TcpStream> {
    let sock_addr = format!("localhost:{}", port);
    info!(port, "等待TCP连接: {}", sock_addr);
    let sock = TcpListener::bind(sock_addr)?;
    let (stream, addr) = sock.accept()?;
    info!(?addr, "TCP连接已建立");
    Ok(stream)
}

impl Target for Emulator {
    type Arch = gdbstub_arch::riscv::Riscv64;
    type Error = String;

    #[inline(always)]
    fn base_ops(&mut self) -> target::ext::base::BaseOps<'_, Self::Arch, Self::Error> {
        target::ext::base::BaseOps::SingleThread(self)
    }

    #[inline(always)]
    fn support_breakpoints(
        &mut self,
    ) -> Option<target::ext::breakpoints::BreakpointsOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadBase for Emulator {
    fn read_registers(
        &mut self,
        regs: &mut <Self::Arch as gdbstub::arch::Arch>::Registers,
    ) -> target::TargetResult<(), Self> {
        regs.pc = self.state.get_pc();
        regs.x = self.state.get_regs().to_owned();
        Ok(())
    }

    fn write_registers(
        &mut self,
        regs: &<Self::Arch as gdbstub::arch::Arch>::Registers,
    ) -> target::TargetResult<(), Self> {
        self.state.set_npc(regs.pc);
        self.state.sync_pc();
        for (i, &val) in regs.x.iter().enumerate() {
            self.state
                .set_reg(i, val)
                .map_err(|_| target::TargetError::NonFatal)?;
        }
        Ok(())
    }

    fn read_addrs(
        &mut self,
        start_addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
        data: &mut [u8],
    ) -> target::TargetResult<usize, Self> {
        for (addr, val) in (start_addr..).zip(data.iter_mut()) {
            match self.state.read_memory(addr, 1) {
                Ok(byte) => *val = byte[0],
                Err(_) => return Err(target::TargetError::NonFatal),
            }
        }
        Ok(data.len())
    }

    fn write_addrs(
        &mut self,
        start_addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
        data: &[u8],
    ) -> target::TargetResult<(), Self> {
        for (addr, &val) in (start_addr..).zip(data.iter()) {
            self.state
                .write_memory(addr, &[val])
                .map_err(|_| target::TargetError::NonFatal)?;
        }
        Ok(())
    }

    #[inline(always)]
    fn support_resume(
        &mut self,
    ) -> Option<target::ext::base::singlethread::SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }
}

impl SingleRegisterAccess<()> for Emulator {
    fn read_register(
        &mut self,
        _tid: (),
        reg_id: <Self::Arch as gdbstub::arch::Arch>::RegId,
        buf: &mut [u8],
    ) -> target::TargetResult<usize, Self> {
        match reg_id {
            RiscvRegId::Pc => {
                let pc = self.state.get_pc();
                buf.copy_from_slice(&pc.to_le_bytes());
                Ok(buf.len())
            }
            RiscvRegId::Gpr(reg) => {
                let reg_value = self
                    .state
                    .get_reg(reg as usize)
                    .map_err(|_| target::TargetError::NonFatal)?;
                buf.copy_from_slice(&reg_value.to_le_bytes());
                Ok(buf.len())
            }
            _ => {
                // 其他寄存器暂不支持
                Err(target::TargetError::NonFatal)
            }
        }
    }

    fn write_register(
        &mut self,
        _tid: (),
        reg_id: <Self::Arch as gdbstub::arch::Arch>::RegId,
        val: &[u8],
    ) -> target::TargetResult<(), Self> {
        match reg_id {
            RiscvRegId::Pc => {
                let pc =
                    u64::from_le_bytes(val.try_into().map_err(|_| target::TargetError::NonFatal)?);
                self.state.set_npc(pc);
                self.state.sync_pc();
                Ok(())
            }
            RiscvRegId::Gpr(reg) => {
                let reg_value =
                    u64::from_le_bytes(val.try_into().map_err(|_| target::TargetError::NonFatal)?);
                self.state
                    .set_reg(reg as usize, reg_value)
                    .map_err(|_| target::TargetError::NonFatal)?;
                Ok(())
            }
            _ => {
                // 其他寄存器暂不支持
                Err(target::TargetError::NonFatal)
            }
        }
    }
}

impl SingleThreadSingleStep for Emulator {
    fn step(
        &mut self,
        signal: Option<gdbstub::common::Signal>,
    ) -> std::result::Result<(), Self::Error> {
        if signal.is_some() {
            tracing::error!("带信号的single step不受支持");
            return Err("带信号的single step不受支持".to_string());
        }
        self.exec_mode = ExecMode::Step;
        Ok(())
    }
}

impl SingleThreadRangeStepping for Emulator {
    fn resume_range_step(
        &mut self,
        start: <Self::Arch as gdbstub::arch::Arch>::Usize,
        end: <Self::Arch as gdbstub::arch::Arch>::Usize,
    ) -> std::result::Result<(), Self::Error> {
        self.exec_mode = ExecMode::RangeStep(start, end);
        Ok(())
    }
}

impl SingleThreadResume for Emulator {
    fn resume(
        &mut self,
        signal: Option<gdbstub::common::Signal>,
    ) -> std::result::Result<(), Self::Error> {
        if signal.is_some() {
            tracing::error!("带信号的resume不受支持");
            return Err("带信号的resume不受支持".to_string());
        }

        self.exec_mode = ExecMode::Continue;
        Ok(())
    }

    #[inline(always)]
    fn support_single_step(
        &mut self,
    ) -> Option<target::ext::base::singlethread::SingleThreadSingleStepOps<'_, Self>> {
        Some(self)
    }

    #[inline(always)]
    fn support_range_step(
        &mut self,
    ) -> Option<target::ext::base::singlethread::SingleThreadRangeSteppingOps<'_, Self>> {
        Some(self)
    }
}
