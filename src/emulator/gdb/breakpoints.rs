use crate::emulator::Emulator;
use gdbstub::target;

impl target::ext::breakpoints::Breakpoints for Emulator {
    #[inline(always)]
    fn support_sw_breakpoint(
        &mut self,
    ) -> Option<target::ext::breakpoints::SwBreakpointOps<'_, Self>> {
        Some(self)
    }

    #[inline(always)]
    fn support_hw_watchpoint(
        &mut self,
    ) -> Option<target::ext::breakpoints::HwWatchpointOps<'_, Self>> {
        Some(self)
    }
}

impl target::ext::breakpoints::SwBreakpoint for Emulator {
    fn add_sw_breakpoint(
        &mut self,
        addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
        _kind: <Self::Arch as gdbstub::arch::Arch>::BreakpointKind,
    ) -> target::TargetResult<bool, Self> {
        self.breakpoints.insert(addr);
        Ok(true)
    }

    fn remove_sw_breakpoint(
        &mut self,
        addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
        _kind: <Self::Arch as gdbstub::arch::Arch>::BreakpointKind,
    ) -> target::TargetResult<bool, Self> {
        Ok(self.breakpoints.remove(&addr))
    }
}

impl target::ext::breakpoints::HwWatchpoint for Emulator {
    fn add_hw_watchpoint(
        &mut self,
        addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
        len: <Self::Arch as gdbstub::arch::Arch>::Usize,
        _kind: target::ext::breakpoints::WatchKind,
    ) -> target::TargetResult<bool, Self> {
        for addr in addr..(addr + len) {
            self.watchpoints.insert(addr);
        }
        Ok(true)
    }

    fn remove_hw_watchpoint(
        &mut self,
        addr: <Self::Arch as gdbstub::arch::Arch>::Usize,
        len: <Self::Arch as gdbstub::arch::Arch>::Usize,
        _kind: target::ext::breakpoints::WatchKind,
    ) -> target::TargetResult<bool, Self> {
        for addr in addr..(addr + len) {
            if !self.watchpoints.remove(&addr) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
