//! 内存管理模块

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use thiserror::Error;
use mmio_trait::{MmioDevice, DeviceError};

use crate::const_values::EmuConfig;

/// 内存错误类型
#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("内存访问越界: 地址 {addr:#x}, 大小 {size}")]
    OutOfBounds { addr: u64, size: usize },
    #[error("内存对齐错误: 地址 {addr:#x}, 对齐要求 {alignment}")]
    Misaligned { addr: u64, alignment: usize },
    #[error("MMIO 区域重叠: 地址 {addr:#x}")]
    MmioOverlap { addr: u64 },
    #[error("设备错误: {0}")]
    Device(#[from] DeviceError),
}

/// MMIO 区域
pub struct MmioRegion {
    pub base: u64,
    pub size: u64,
    pub device: Arc<Mutex<dyn MmioDevice>>,
    pub name: String,
}

impl std::fmt::Debug for MmioRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MmioRegion")
            .field("base", &format_args!("{:#x}", self.base))
            .field("size", &format_args!("{:#x}", self.size))
            .field("name", &self.name)
            .finish()
    }
}

/// 内存管理结构
#[derive(Debug)]
pub struct Memory {
    /// 内存数据
    data: Vec<u8>,
    #[allow(unused)]
    config: Rc<EmuConfig>,
    /// 主内存基地址（来自设备配置文件）
    memory_base: u64,
    /// 主内存大小 (来自设备配置文件, 单位: 字节)
    memory_size: usize,
    /// MMIO 区域列表
    mmio_regions: Vec<MmioRegion>,
    /// is last mmio
    is_last_mmio: RefCell<bool>,
}

impl Memory {
    /// 使用主配置和设备配置创建内存实例
    pub fn new(config: Rc<EmuConfig>, device_file: &crate::const_values::DeviceFile) -> Result<Self, MemoryError> {
        let size = device_file.memory.memory_size * 1024 * 1024; // 转换为字节
        if !size.is_power_of_two() {
            return Err(MemoryError::Misaligned { addr: 0, alignment: 2 });
        }
        Ok(Self {
            data: vec![0; size],
            config,
            memory_base: device_file.memory.memory_base,
            memory_size: device_file.memory.memory_size * 1024 * 1024,
            mmio_regions: Vec::new(),
            is_last_mmio: RefCell::new(false),
        })
    }

    /// 映射 MMIO 设备
    pub fn map_mmio(
        &mut self,
        base: u64,
        size: u64,
        device: Arc<Mutex<dyn MmioDevice>>,
        name: String,
    ) -> Result<(), MemoryError> {
        let new_end = base + size;

        // 检查地址重叠
        for region in &self.mmio_regions {
            let region_end = region.base + region.size;

            if base < region_end && new_end > region.base {
                return Err(MemoryError::MmioOverlap { addr: base });
            }
        }

        if base < (self.memory_base + self.memory_size as u64) && new_end > self.memory_base {
            return Err(MemoryError::MmioOverlap { addr: base });
        }

        self.mmio_regions.push(MmioRegion {
            base,
            size,
            device,
            name,
        });

        Ok(())
    }

    /// 排序 MMIO 区域
    pub fn sort_mmio_regions(&mut self) {
        self.mmio_regions.sort_by_key(|region| region.base);
    }

    /// 查找覆盖指定地址的 MMIO 区域
    #[inline(always)]
    fn find_mmio_region(&self, addr: u64) -> Option<&MmioRegion> {
        // self.mmio_regions
        //     .iter()
        //     .find(|region| addr >= region.base && addr < region.base + region.size)
        self.mmio_regions
            .binary_search_by(|region| {
                let start = region.base;
                let end = region.base + region.size;
                if addr < start {
                    std::cmp::Ordering::Greater
                } else if addr >= end {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Equal
                }
            }).ok().map(|index| &self.mmio_regions[index])
    }

    #[inline(always)]
    pub fn is_mem_region(&self, addr: u64) -> bool {
        addr >= self.memory_base && addr < self.memory_base + self.memory_size as u64
    }

    /// 检查给定地址范围是否完全在主内存区域内
    #[inline(always)]
    pub fn is_mem_region_range(&self, addr: u64, size: usize) -> bool {
        addr >= self.memory_base && 
        addr.saturating_add(size as u64) <= self.memory_base + self.memory_size as u64
    }

    /// 移除 MMIO 映射
    pub fn unmap_mmio(&mut self, base: u64) -> bool {
        if let Some(index) = self.mmio_regions.iter().position(|r| r.base == base) {
            self.mmio_regions.remove(index);
            true
        } else {
            false
        }
    }

    /// 转换并检查地址有效性和对齐
    #[inline(always)]
    fn translate_address(
        &self,
        addr: u64,
        size: usize,
        alignment: usize,
    ) -> Result<u64, MemoryError> {
        // 使用设备配置中的 memory_base 作为物理内存基地址
        let real_addr = addr.wrapping_sub(self.memory_base);

        if alignment > 1 && real_addr % alignment as u64 != 0 {
            return Err(MemoryError::Misaligned {
                addr: real_addr,
                alignment,
            });
        }

        let end = real_addr
            .checked_add(size as u64)
            .ok_or(MemoryError::OutOfBounds { addr, size })?;

        if end > self.data.len() as u64 {
            return Err(MemoryError::OutOfBounds { addr, size });
        }
        Ok(real_addr)
    }

    /// 读取内存
    #[inline(always)]
    pub fn read(&self, addr: u64, size: usize) -> Result<Vec<u8>, MemoryError> {
        if self.is_mem_region(addr) {
            // 普通内存访问 - 根据长度选择优化路径
            match size {
                1 => {
                    // 字节访问
                    if !self.is_mem_region_range(addr, 1) {
                        return Err(MemoryError::OutOfBounds { addr, size: 1 });
                    }
                    let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
                    let value = unsafe { self.read_byte_unsafe(real_addr) };
                    return Ok(vec![value]);
                }
                2 => {
                    // 半字访问
                    if !self.is_mem_region_range(addr, 2) {
                        return Err(MemoryError::OutOfBounds { addr, size: 2 });
                    }
                    let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
                    let value = unsafe { self.read_halfword_unsafe(real_addr) };
                    return Ok(value.to_le_bytes().to_vec());
                }
                4 => {
                    // 字访问
                    if !self.is_mem_region_range(addr, 4) {
                        return Err(MemoryError::OutOfBounds { addr, size: 4 });
                    }
                    let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
                    let value = unsafe { self.read_word_unsafe(real_addr) };
                    return Ok(value.to_le_bytes().to_vec());
                }
                8 => {
                    // 双字访问
                    if !self.is_mem_region_range(addr, 8) {
                        return Err(MemoryError::OutOfBounds { addr, size: 8 });
                    }
                    let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
                    let value = unsafe { self.read_doubleword_unsafe(real_addr) };
                    return Ok(value.to_le_bytes().to_vec());
                }
                _ => {
                    // 非标准长度，使用传统方法
                    let real_addr = self.translate_address(addr, size, 1)?;
                    let start = real_addr as usize;
                    return Ok(self.data[start..start + size].to_vec());
                }
            }
        }

        // 检查是否为 MMIO 访问
        if let Some(region) = self.find_mmio_region(addr) {
            let offset = addr - region.base;
            let mut device = region.device.lock().unwrap();
            let res = device.read(offset, size)?;
            *self.is_last_mmio.borrow_mut() = true;
            return Ok(res);
        }

        Err(MemoryError::OutOfBounds { addr, size })
    }

    /// 快速读取u32指令（unsafe版本，仅用于取指）
    /// 假设地址有效且在主内存范围内，跳过边界检查和MMIO检查以提高性能
    #[inline(always)]
    pub unsafe fn read_u32_fast(&self, addr: u64) -> u32 {
        let real_addr = addr.wrapping_sub(self.memory_base) as usize;
        // 直接从内存读取4字节并转换为u32
        // 假设地址已经过检查且对齐
        unsafe {
            let ptr = self.data.as_ptr().add(real_addr) as *const u32;
            ptr.read_unaligned().to_le()
        }
    }

    /// 快速读取字节（unsafe版本）
    #[inline(always)]
    unsafe fn read_byte_unsafe(&self, real_addr: usize) -> u8 {
        unsafe { *self.data.get_unchecked(real_addr) }
    }

    /// 快速读取半字（unsafe版本）
    #[inline(always)]
    unsafe fn read_halfword_unsafe(&self, real_addr: usize) -> u16 {
        unsafe {
            let ptr = self.data.as_ptr().add(real_addr) as *const u16;
            ptr.read_unaligned().to_le()
        }
    }

    /// 快速读取字（unsafe版本）
    #[inline(always)]
    unsafe fn read_word_unsafe(&self, real_addr: usize) -> u32 {
        unsafe {
            let ptr = self.data.as_ptr().add(real_addr) as *const u32;
            ptr.read_unaligned().to_le()
        }
    }

    /// 快速读取双字（unsafe版本）
    #[inline(always)]
    unsafe fn read_doubleword_unsafe(&self, real_addr: usize) -> u64 {
        unsafe {
            let ptr = self.data.as_ptr().add(real_addr) as *const u64;
            ptr.read_unaligned().to_le()
        }
    }

    /// 快速写入字节（unsafe版本）
    #[inline(always)]
    unsafe fn write_byte_unsafe(&mut self, real_addr: usize, value: u8) {
        unsafe { *self.data.get_unchecked_mut(real_addr) = value; }
    }

    /// 快速写入半字（unsafe版本）
    #[inline(always)]
    unsafe fn write_halfword_unsafe(&mut self, real_addr: usize, value: u16) {
        unsafe {
            let ptr = self.data.as_mut_ptr().add(real_addr) as *mut u16;
            ptr.write_unaligned(value.to_le());
        }
    }

    /// 快速写入字（unsafe版本）
    #[inline(always)]
    unsafe fn write_word_unsafe(&mut self, real_addr: usize, value: u32) {
        unsafe {
            let ptr = self.data.as_mut_ptr().add(real_addr) as *mut u32;
            ptr.write_unaligned(value.to_le());
        }
    }

    /// 快速写入双字（unsafe版本）
    #[inline(always)]
    unsafe fn write_doubleword_unsafe(&mut self, real_addr: usize, value: u64) {
        unsafe {
            let ptr = self.data.as_mut_ptr().add(real_addr) as *mut u64;
            ptr.write_unaligned(value.to_le());
        }
    }

    /// 写入内存
    #[inline(always)]
    pub fn write(&mut self, addr: u64, data: &[u8]) -> Result<(), MemoryError> {
        if self.is_mem_region(addr) {
            // 普通内存访问 - 根据长度选择优化路径
            match data.len() {
                1 => {
                    // 字节访问
                    if !self.is_mem_region_range(addr, 1) {
                        return Err(MemoryError::OutOfBounds { addr, size: 1 });
                    }
                    let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
                    unsafe { self.write_byte_unsafe(real_addr, data[0]); }
                }
                2 => {
                    // 半字访问
                    if !self.is_mem_region_range(addr, 2) {
                        return Err(MemoryError::OutOfBounds { addr, size: 2 });
                    }
                    let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
                    let value = u16::from_le_bytes([data[0], data[1]]);
                    unsafe { self.write_halfword_unsafe(real_addr, value); }
                }
                4 => {
                    // 字访问
                    if !self.is_mem_region_range(addr, 4) {
                        return Err(MemoryError::OutOfBounds { addr, size: 4 });
                    }
                    let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
                    let value = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                    unsafe { self.write_word_unsafe(real_addr, value); }
                }
                8 => {
                    // 双字访问
                    if !self.is_mem_region_range(addr, 8) {
                        return Err(MemoryError::OutOfBounds { addr, size: 8 });
                    }
                    let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
                    let value = u64::from_le_bytes([
                        data[0], data[1], data[2], data[3],
                        data[4], data[5], data[6], data[7],
                    ]);
                    unsafe { self.write_doubleword_unsafe(real_addr, value); }
                }
                _ => {
                    // 非标准长度，使用传统方法
                    let real_addr = self.translate_address(addr, data.len(), 1)?;
                    let start = real_addr as usize;
                    self.data[start..start + data.len()].copy_from_slice(data);
                }
            }
            return Ok(())
        }

        // 检查是否为 MMIO 访问
        if let Some(region) = self.find_mmio_region(addr) {
            let offset = addr - region.base;
            let mut device = region.device.lock().unwrap();
            device.write(offset, data)?;
            *self.is_last_mmio.borrow_mut() = true;
            return Ok(());
        }

        Err(MemoryError::OutOfBounds { addr, size: data.len() })
    }

    #[inline(always)]
    pub fn is_last_mmio(&self) -> bool {
        let res = *self.is_last_mmio.borrow();
        *self.is_last_mmio.borrow_mut() = false;
        res
    }

    /// 读取字节
    #[inline(always)]
    pub fn read_byte(&self, addr: u64) -> Result<u8, MemoryError> {
        if self.is_mem_region(addr) {
            // 主内存访问 - 直接使用unsafe版本
            if !self.is_mem_region_range(addr, 1) {
                return Err(MemoryError::OutOfBounds { addr, size: 1 });
            }
            let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
            return Ok(unsafe { self.read_byte_unsafe(real_addr) });
        }

        // MMIO访问 - 通过通用read方法
        if let Some(region) = self.find_mmio_region(addr) {
            let offset = addr - region.base;
            let mut device = region.device.lock().unwrap();
            let res = device.read(offset, 1)?;
            *self.is_last_mmio.borrow_mut() = true;
            return Ok(res[0]);
        }

        Err(MemoryError::OutOfBounds { addr, size: 1 })
    }

    /// 读取半字
    #[inline(always)]
    pub fn read_halfword(&self, addr: u64) -> Result<u16, MemoryError> {
        if self.is_mem_region(addr) {
            // 主内存访问 - 直接使用unsafe版本
            if !self.is_mem_region_range(addr, 2) {
                return Err(MemoryError::OutOfBounds { addr, size: 2 });
            }
            let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
            return Ok(unsafe { self.read_halfword_unsafe(real_addr) });
        }

        // MMIO访问 - 通过通用read方法
        if let Some(region) = self.find_mmio_region(addr) {
            let offset = addr - region.base;
            let mut device = region.device.lock().unwrap();
            let res = device.read(offset, 2)?;
            *self.is_last_mmio.borrow_mut() = true;
            return Ok(u16::from_le_bytes([res[0], res[1]]));
        }

        Err(MemoryError::OutOfBounds { addr, size: 2 })
    }

    /// 读取字
    #[inline(always)]
    pub fn read_word(&self, addr: u64) -> Result<u32, MemoryError> {
        if self.is_mem_region(addr) {
            // 主内存访问 - 直接使用unsafe版本
            if !self.is_mem_region_range(addr, 4) {
                return Err(MemoryError::OutOfBounds { addr, size: 4 });
            }
            let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
            return Ok(unsafe { self.read_word_unsafe(real_addr) });
        }

        // MMIO访问 - 通过通用read方法
        if let Some(region) = self.find_mmio_region(addr) {
            let offset = addr - region.base;
            let mut device = region.device.lock().unwrap();
            let res = device.read(offset, 4)?;
            *self.is_last_mmio.borrow_mut() = true;
            return Ok(u32::from_le_bytes([res[0], res[1], res[2], res[3]]));
        }

        Err(MemoryError::OutOfBounds { addr, size: 4 })
    }

    /// 读取双字
    #[inline(always)]
    pub fn read_doubleword(&self, addr: u64) -> Result<u64, MemoryError> {
        if self.is_mem_region(addr) {
            // 主内存访问 - 直接使用unsafe版本
            if !self.is_mem_region_range(addr, 8) {
                return Err(MemoryError::OutOfBounds { addr, size: 8 });
            }
            let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
            return Ok(unsafe { self.read_doubleword_unsafe(real_addr) });
        }

        // MMIO访问 - 通过通用read方法
        if let Some(region) = self.find_mmio_region(addr) {
            let offset = addr - region.base;
            let mut device = region.device.lock().unwrap();
            let res = device.read(offset, 8)?;
            *self.is_last_mmio.borrow_mut() = true;
            return Ok(u64::from_le_bytes([
                res[0], res[1], res[2], res[3],
                res[4], res[5], res[6], res[7],
            ]));
        }

        Err(MemoryError::OutOfBounds { addr, size: 8 })
    }

    /// 写入字节
    #[inline(always)]
    pub fn write_byte(&mut self, addr: u64, value: u8) -> Result<(), MemoryError> {
        if self.is_mem_region(addr) {
            // 主内存访问 - 直接使用unsafe版本
            if !self.is_mem_region_range(addr, 1) {
                return Err(MemoryError::OutOfBounds { addr, size: 1 });
            }
            let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
            unsafe { self.write_byte_unsafe(real_addr, value); }
            return Ok(());
        }

        // MMIO访问 - 通过通用write方法
        if let Some(region) = self.find_mmio_region(addr) {
            let offset = addr - region.base;
            let mut device = region.device.lock().unwrap();
            device.write(offset, &[value])?;
            *self.is_last_mmio.borrow_mut() = true;
            return Ok(());
        }

        Err(MemoryError::OutOfBounds { addr, size: 1 })
    }

    /// 写入半字
    #[inline(always)]
    pub fn write_halfword(&mut self, addr: u64, value: u16) -> Result<(), MemoryError> {
        if self.is_mem_region(addr) {
            // 主内存访问 - 直接使用unsafe版本
            if !self.is_mem_region_range(addr, 2) {
                return Err(MemoryError::OutOfBounds { addr, size: 2 });
            }
            let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
            unsafe { self.write_halfword_unsafe(real_addr, value); }
            return Ok(());
        }

        // MMIO访问 - 通过通用write方法
        if let Some(region) = self.find_mmio_region(addr) {
            let offset = addr - region.base;
            let mut device = region.device.lock().unwrap();
            device.write(offset, &value.to_le_bytes())?;
            *self.is_last_mmio.borrow_mut() = true;
            return Ok(());
        }

        Err(MemoryError::OutOfBounds { addr, size: 2 })
    }

    /// 写入字
    #[inline(always)]
    pub fn write_word(&mut self, addr: u64, value: u32) -> Result<(), MemoryError> {
        if self.is_mem_region(addr) {
            // 主内存访问 - 直接使用unsafe版本
            if !self.is_mem_region_range(addr, 4) {
                return Err(MemoryError::OutOfBounds { addr, size: 4 });
            }
            let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
            unsafe { self.write_word_unsafe(real_addr, value); }
            return Ok(());
        }

        // MMIO访问 - 通过通用write方法
        if let Some(region) = self.find_mmio_region(addr) {
            let offset = addr - region.base;
            let mut device = region.device.lock().unwrap();
            device.write(offset, &value.to_le_bytes())?;
            *self.is_last_mmio.borrow_mut() = true;
            return Ok(());
        }

        Err(MemoryError::OutOfBounds { addr, size: 4 })
    }

    /// 写入双字
    #[inline(always)]
    pub fn write_doubleword(&mut self, addr: u64, value: u64) -> Result<(), MemoryError> {
        if self.is_mem_region(addr) {
            // 主内存访问 - 直接使用unsafe版本
            if !self.is_mem_region_range(addr, 8) {
                return Err(MemoryError::OutOfBounds { addr, size: 8 });
            }
            let real_addr = (addr.wrapping_sub(self.memory_base)) as usize;
            unsafe { self.write_doubleword_unsafe(real_addr, value); }
            return Ok(());
        }

        // MMIO访问 - 通过通用write方法
        if let Some(region) = self.find_mmio_region(addr) {
            let offset = addr - region.base;
            let mut device = region.device.lock().unwrap();
            device.write(offset, &value.to_le_bytes())?;
            *self.is_last_mmio.borrow_mut() = true;
            return Ok(());
        }

        Err(MemoryError::OutOfBounds { addr, size: 8 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use crate::const_values::{EmuConfig, MemoryConfig, InstSetConfig, DebugConfig, OthersConfig};

    // 模拟 UART 设备
    struct MockUart {
        data: Vec<u8>,
    }

    impl MockUart {
        fn new() -> Self {
            Self { data: Vec::new() }
        }
    }

    impl mmio_trait::MmioDevice for MockUart {
        fn read(&mut self, _offset: u64, size: usize) -> Result<Vec<u8>, mmio_trait::DeviceError> {
            Ok(vec![0x01; size]) // 返回状态就绪
        }

        fn write(&mut self, _offset: u64, data: &[u8]) -> Result<(), mmio_trait::DeviceError> {
            self.data.extend_from_slice(data);
            Ok(())
        }

        fn name(&self) -> &str {
            "mock_uart"
        }
    }

    fn create_test_config() -> (Rc<EmuConfig>, crate::const_values::DeviceFile) {
        let config = Rc::new(EmuConfig {
            memory: MemoryConfig {
                boot_pc: 0x8000_0000,
            },
            inst_set: InstSetConfig {
                m_ext: false,
                a_ext: false,
                c_ext: false,
            },
            debug: DebugConfig {
                event_list_size: 64,
                #[cfg(feature = "tracer")]
                instruction_tracer_list_size: 64,
            },
            others: OthersConfig {
                decoder_cache_size: 1024,
            },
        });

        let device_file = crate::const_values::DeviceFile {
            memory: crate::const_values::DeviceFileMemory {
                memory_base: 0x8000_0000,
                memory_size: 128,
            },
            devices: Vec::new(),
        };

        (config, device_file)
    }

    #[test]
    fn test_memory_creation() {
        let (config, device_file) = create_test_config();
        let memory = Memory::new(config, &device_file).unwrap();
        assert_eq!(memory.data.len(), 128 * 1024 * 1024);
    }

    #[test]
    fn test_mmio_mapping() {
        let (config, device_file) = create_test_config();
        let mut memory = Memory::new(config, &device_file).unwrap();

        let uart = Arc::new(Mutex::new(MockUart::new()));
        let result = memory.map_mmio(0x1000_0000, 0x100, uart, "test_uart".to_string());
        assert!(result.is_ok());

        assert_eq!(memory.mmio_regions.len(), 1);
        assert_eq!(memory.mmio_regions[0].base, 0x1000_0000);
        assert_eq!(memory.mmio_regions[0].size, 0x100);
        assert_eq!(memory.mmio_regions[0].name, "test_uart");
    }

    #[test]
    fn test_mmio_overlap_detection() {
        let (config, device_file) = create_test_config();
        let mut memory = Memory::new(config, &device_file).unwrap();

        let uart1 = Arc::new(Mutex::new(MockUart::new()));
        memory.map_mmio(0x1000_0000, 0x100, uart1, "uart1".to_string()).unwrap();

        let uart2 = Arc::new(Mutex::new(MockUart::new()));
        let result = memory.map_mmio(0x1000_0050, 0x100, uart2, "uart2".to_string());
        assert!(matches!(result, Err(MemoryError::MmioOverlap { .. })));
    }

    #[test]
    fn test_mmio_read_write() {
        let (config, device_file) = create_test_config();
        let mut memory = Memory::new(config, &device_file).unwrap();

        let uart = Arc::new(Mutex::new(MockUart::new()));
        memory.map_mmio(0x1000_0000, 0x100, uart.clone(), "test_uart".to_string()).unwrap();

        // 测试写入
        memory.write_byte(0x1000_0000, b'H').unwrap();
        memory.write_byte(0x1000_0001, b'i').unwrap();

        // 验证数据被写入设备
        let device = uart.lock().unwrap();
        assert_eq!(device.data, vec![b'H', b'i']);

        // 测试读取
        drop(device);
        let data = memory.read_byte(0x1000_0000).unwrap();
        assert_eq!(data, 0x01); // MockUart 返回 0x01
    }

    #[test]
    fn test_regular_memory_access() {
        let (config, device_file) = create_test_config();
        let mut memory = Memory::new(config, &device_file).unwrap();

        // 测试普通内存读写
        let addr = 0x8000_1000;
        memory.write_byte(addr, 0x42).unwrap();
        let data = memory.read_byte(addr).unwrap();
        assert_eq!(data, 0x42);

        // 测试多字节访问
        memory.write_word(addr + 4, 0x12345678).unwrap();
        let word = memory.read_word(addr + 4).unwrap();
        assert_eq!(word, 0x12345678);
    }

    #[test]
    fn test_fast_u32_read() {
        let (config, device_file) = create_test_config();
        let mut memory = Memory::new(config, &device_file).unwrap();

        // 测试快速u32读取
        let addr = 0x8000_1000;
        let test_value = 0x12345678u32;
        
        // 写入测试值
        memory.write_word(addr, test_value).unwrap();
        
        // 使用普通方法读取
        let normal_read = memory.read_word(addr).unwrap();
        
        // 使用快速方法读取
        let fast_read = unsafe { memory.read_u32_fast(addr) };
        
        // 验证两种方法读取的结果相同
        assert_eq!(normal_read, fast_read);
        assert_eq!(fast_read, test_value);
    }

    #[test]
    fn test_mem_region_range_check() {
        let (config, device_file) = create_test_config();
        let memory = Memory::new(config, &device_file).unwrap();

        // 测试有效地址范围
        assert!(memory.is_mem_region_range(0x8000_0000, 4));
        assert!(memory.is_mem_region_range(0x8000_1000, 4));
        
        // 测试边界情况
        let last_valid_addr = 0x8000_0000 + (128 * 1024 * 1024) - 4;
        assert!(memory.is_mem_region_range(last_valid_addr, 4));
        
        // 测试越界情况
        let overflow_addr = 0x8000_0000 + (128 * 1024 * 1024) - 3;
        assert!(!memory.is_mem_region_range(overflow_addr, 4));
        
        // 测试完全超出范围的地址
        assert!(!memory.is_mem_region_range(0x9000_0000, 4));
        assert!(!memory.is_mem_region_range(0x7000_0000, 4));
    }
}
