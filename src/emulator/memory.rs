//! 内存管理模块

use thiserror::Error;
use crate::const_values::MEMORY_BASE;

/// 内存错误类型
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("内存访问越界: 地址 {addr:#x}, 大小 {size}")]
    OutOfBounds { addr: u64, size: usize },
    #[error("内存对齐错误: 地址 {addr:#x}, 对齐要求 {alignment}")]
    Misaligned { addr: u64, alignment: usize },
}

/// 内存管理结构
#[derive(Debug, Clone)]
pub struct Memory {
    /// 内存数据
    data: Vec<u8>,
}

impl Memory {
    /// 创建新的内存实例
    pub fn new(size: usize) -> Result<Self, MemoryError> {
        if !size.is_power_of_two() {
            return Err(MemoryError::OutOfBounds { addr: 0, size });
        }
        Ok(Self {
            data: vec![0; size],
        })
    }

    /// 转换并检查地址有效性和对齐
    fn translate_address(&self, addr: u64, size: usize, alignment: usize) -> Result<u64, MemoryError> {
        let real_addr = addr.wrapping_sub(MEMORY_BASE);
        
        if alignment > 1 && real_addr % alignment as u64 != 0 {
            return Err(MemoryError::Misaligned { addr: real_addr, alignment });
        }

        let end = real_addr.checked_add(size as u64)
            .ok_or(MemoryError::OutOfBounds { addr, size })?;
            
        if end > self.data.len() as u64 {
            return Err(MemoryError::OutOfBounds { addr, size });
        }
        Ok(real_addr)
    }

    /// 读取内存
    pub fn read(&self, addr: u64, size: usize) -> Result<Vec<u8>, MemoryError> {
        let real_addr = self.translate_address(addr, size, 1)?;
        let start = real_addr as usize;
        Ok(self.data[start..start + size].to_vec())
    }

    /// 写入内存
    pub fn write(&mut self, addr: u64, data: &[u8]) -> Result<(), MemoryError> {
        let real_addr = self.translate_address(addr, data.len(), 1)?;
        let start = real_addr as usize;
        self.data[start..start + data.len()].copy_from_slice(data);
        Ok(())
    }

    /// 读取字节
    pub fn read_byte(&self, addr: u64) -> Result<u8, MemoryError> {
        let real_addr = self.translate_address(addr, 1, 1)?;
        Ok(self.data[real_addr as usize])
    }

    /// 读取半字
    pub fn read_halfword(&self, addr: u64) -> Result<u16, MemoryError> {
        let real_addr = self.translate_address(addr, 2, 2)?;
        let bytes = self.data[real_addr as usize..(real_addr as usize + 2)].to_vec();
        Ok(u16::from_le_bytes(bytes.try_into().map_err(|_| 
            MemoryError::OutOfBounds { addr, size: 2 })?))
    }

    /// 读取字
    pub fn read_word(&self, addr: u64) -> Result<u32, MemoryError> {
        let real_addr = self.translate_address(addr, 4, 4)?;
        let bytes = self.data[real_addr as usize..(real_addr as usize + 4)].to_vec();
        Ok(u32::from_le_bytes(bytes.try_into().map_err(|_| 
            MemoryError::OutOfBounds { addr, size: 4 })?))
    }

    /// 读取双字
    pub fn read_doubleword(&self, addr: u64) -> Result<u64, MemoryError> {
        let real_addr = self.translate_address(addr, 8, 8)?;
        let bytes = self.data[real_addr as usize..(real_addr as usize + 8)].to_vec();
        Ok(u64::from_le_bytes(bytes.try_into().map_err(|_| 
            MemoryError::OutOfBounds { addr, size: 8 })?))
    }

    /// 写入字节
    pub fn write_byte(&mut self, addr: u64, value: u8) -> Result<(), MemoryError> {
        self.write(addr, &[value])
    }

    /// 写入半字
    pub fn write_halfword(&mut self, addr: u64, value: u16) -> Result<(), MemoryError> {
        let real_addr = self.translate_address(addr, 2, 2)?;
        let value_bytes = value.to_le_bytes();
        self.data[real_addr as usize..(real_addr as usize + 2)].copy_from_slice(&value_bytes);
        Ok(())
    }

    /// 写入字
    pub fn write_word(&mut self, addr: u64, value: u32) -> Result<(), MemoryError> {
        let real_addr = self.translate_address(addr, 4, 4)?;
        let value_bytes = value.to_le_bytes();
        self.data[real_addr as usize..(real_addr as usize + 4)].copy_from_slice(&value_bytes);
        Ok(())
    }

    /// 写入双字
    pub fn write_doubleword(&mut self, addr: u64, value: u64) -> Result<(), MemoryError> {
        let real_addr = self.translate_address(addr, 8, 8)?;
        let value_bytes = value.to_le_bytes();
        self.data[real_addr as usize..(real_addr as usize + 8)].copy_from_slice(&value_bytes);
        Ok(())
    }
}
