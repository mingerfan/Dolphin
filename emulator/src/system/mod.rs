//! 系统接口模块

mod memory;
mod syscall;

pub use memory::{Memory, MemoryError};
pub use syscall::{SyscallContext, SyscallError, handle_syscall};

/// 系统接口结构体
pub struct System {
    /// 内存管理
    memory: Memory,
}

impl System {
    /// 创建新的系统接口实例
    pub fn new(memory_size: usize) -> Result<Self, MemoryError> {
        Ok(Self {
            memory: Memory::new(memory_size)?,
        })
    }

    /// 读取内存
    pub fn read_memory(&self, addr: u64, size: usize) -> Result<Vec<u8>, MemoryError> {
        self.memory.read(addr, size)
    }

    /// 写入内存
    pub fn write_memory(&mut self, addr: u64, data: &[u8]) -> Result<(), MemoryError> {
        self.memory.write(addr, data)
    }

    /// 读取指令
    pub fn fetch_instruction(&self, pc: u64) -> Result<u32, MemoryError> {
        let bytes = self.read_memory(pc, 4)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    /// 处理系统调用
    pub fn handle_syscall(&mut self, context: SyscallContext) -> Result<u64, SyscallError> {
        handle_syscall(context, &mut self.memory)
    }
}
