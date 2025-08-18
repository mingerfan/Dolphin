use thiserror::Error;

/// 内存错误类型
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("内存访问越界: 地址 {addr:#x}, 大小 {size}")]
    OutOfBounds { addr: u64, size: usize },
    #[error("内存对齐错误: 地址 {addr:#x}, 对齐要求 {alignment}")]
    Misaligned { addr: u64, alignment: usize },
}

/// 内存管理结构
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

    /// 检查地址是否有效
    fn check_bounds(&self, addr: u64, size: usize) -> Result<(), MemoryError> {
        let end = addr
            .checked_add(size as u64)
            .ok_or(MemoryError::OutOfBounds { addr, size })?;

        if end > self.data.len() as u64 {
            return Err(MemoryError::OutOfBounds { addr, size });
        }
        Ok(())
    }

    /// 读取内存
    pub fn read(&self, addr: u64, size: usize) -> Result<Vec<u8>, MemoryError> {
        self.check_bounds(addr, size)?;
        let start = addr as usize;
        Ok(self.data[start..start + size].to_vec())
    }

    /// 写入内存
    pub fn write(&mut self, addr: u64, data: &[u8]) -> Result<(), MemoryError> {
        self.check_bounds(addr, data.len())?;
        let start = addr as usize;
        self.data[start..start + data.len()].copy_from_slice(data);
        Ok(())
    }

    /// 读取字节
    pub fn read_byte(&self, addr: u64) -> Result<u8, MemoryError> {
        self.check_bounds(addr, 1)?;
        Ok(self.data[addr as usize])
    }

    /// 读取半字
    pub fn read_halfword(&self, addr: u64) -> Result<u16, MemoryError> {
        if addr % 2 != 0 {
            return Err(MemoryError::Misaligned { addr, alignment: 2 });
        }
        self.check_bounds(addr, 2)?;
        let bytes = self.read(addr, 2)?;
        Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
    }

    /// 读取字
    pub fn read_word(&self, addr: u64) -> Result<u32, MemoryError> {
        if addr % 4 != 0 {
            return Err(MemoryError::Misaligned { addr, alignment: 4 });
        }
        self.check_bounds(addr, 4)?;
        let bytes = self.read(addr, 4)?;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
    }

    /// 读取双字
    pub fn read_doubleword(&self, addr: u64) -> Result<u64, MemoryError> {
        if addr % 8 != 0 {
            return Err(MemoryError::Misaligned { addr, alignment: 8 });
        }
        self.check_bounds(addr, 8)?;
        let bytes = self.read(addr, 8)?;
        Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
    }

    /// 写入字节
    pub fn write_byte(&mut self, addr: u64, value: u8) -> Result<(), MemoryError> {
        self.write(addr, &[value])
    }

    /// 写入半字
    pub fn write_halfword(&mut self, addr: u64, value: u16) -> Result<(), MemoryError> {
        if addr % 2 != 0 {
            return Err(MemoryError::Misaligned { addr, alignment: 2 });
        }
        self.write(addr, &value.to_le_bytes())
    }

    /// 写入字
    pub fn write_word(&mut self, addr: u64, value: u32) -> Result<(), MemoryError> {
        if addr % 4 != 0 {
            return Err(MemoryError::Misaligned { addr, alignment: 4 });
        }
        self.write(addr, &value.to_le_bytes())
    }

    /// 写入双字
    pub fn write_doubleword(&mut self, addr: u64, value: u64) -> Result<(), MemoryError> {
        if addr % 8 != 0 {
            return Err(MemoryError::Misaligned { addr, alignment: 8 });
        }
        self.write(addr, &value.to_le_bytes())
    }
}
