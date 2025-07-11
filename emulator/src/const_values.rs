/// 内存基地址
pub const MEMORY_BASE: u64 = 0x8000_0000;
/// 事件列表大小
pub const EVENT_LIST_SIZE: usize = 64;

/// Decoer LRU缓存大小
pub const DECODER_LRU_CACHE_SIZE: usize = 1024;

#[cfg(feature = "tracer")]
pub const INSTRUCTION_TRACER_LIST_SIZE: usize = 64;

