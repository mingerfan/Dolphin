//! Bit slice operations similar to Chisel's bit slicing

/// Trait for bit-level operations on integers
pub trait BitSlice {
    /// Get a single bit at position (0-based, LSB is 0)
    /// Panics if pos >= 64
    fn bit(&self, pos: usize) -> bool;
    
    /// Get a range of bits [start..end) (0-based, LSB is 0)
    /// Panics if range is out of bounds or start > end
    fn bit_range(&self, range: std::ops::Range<usize>) -> u64;
    
    /// Set a single bit at position (0-based, LSB is 0)
    /// Panics if pos >= 64
    fn set_bit(&mut self, pos: usize, value: bool);
    
    /// Set a range of bits [start..end) (0-based, LSB is 0)
    /// Panics if range is out of bounds or start > end
    fn set_bit_range(&mut self, range: std::ops::Range<usize>, value: u64);
}

impl BitSlice for u64 {
    #[inline(always)]
    fn bit(&self, pos: usize) -> bool {
        assert!(pos < 64, "Bit position out of bounds");
        (self & (1 << pos)) != 0
    }

    #[inline(always)]
    fn bit_range(&self, range: std::ops::Range<usize>) -> u64 {
        assert!(range.end <= 64, "Bit range end out of bounds");
        assert!(range.start <= range.end, "Invalid bit range");
        
        if range.start == range.end {
            return 0;
        }

        let mask = if range.end == 64 {
            u64::MAX
        } else {
            (1 << range.end) - 1
        };
        
        (self & mask) >> range.start
    }

    #[inline(always)]
    fn set_bit(&mut self, pos: usize, value: bool) {
        assert!(pos < 64, "Bit position out of bounds");
        if value {
            *self |= 1 << pos;
        } else {
            *self &= !(1 << pos);
        }
    }

    #[inline(always)]
    fn set_bit_range(&mut self, range: std::ops::Range<usize>, value: u64) {
        assert!(range.end <= 64, "Bit range end out of bounds");
        assert!(range.start <= range.end, "Invalid bit range");
        
        let width = range.end - range.start;
        let value_mask = if width == 64 {
            u64::MAX
        } else {
            (1 << width) - 1
        };
        
        assert!(value <= value_mask, "Value too large for bit range");
        
        let mask = if range.end == 64 {
            u64::MAX
        } else {
            (1 << range.end) - 1
        } ^ ((1 << range.start) - 1);
        
        *self = (*self & !mask) | ((value << range.start) & mask);
    }
}

impl BitSlice for u32 {
    #[inline(always)]
    fn bit(&self, pos: usize) -> bool {
        assert!(pos < 32, "Bit position out of bounds");
        (self & (1 << pos)) != 0
    }

    #[inline(always)]
    fn bit_range(&self, range: std::ops::Range<usize>) -> u64 {
        assert!(range.end <= 32, "Bit range end out of bounds");
        assert!(range.start <= range.end, "Invalid bit range");

        if range.start == range.end {
            return 0;
        }

        let mask = if range.end == 32 {
            u32::MAX
        } else {
            (1 << range.end) - 1
        };

        ((self & mask) >> range.start) as u64
    }

    #[inline(always)]
    fn set_bit(&mut self, pos: usize, value: bool) {
        assert!(pos < 32, "Bit position out of bounds");
        if value {
            *self |= 1 << pos;
        } else {
            *self &= !(1 << pos);
        }
    }

    #[inline(always)]
    fn set_bit_range(&mut self, range: std::ops::Range<usize>, value: u64) {
        assert!(range.end <= 32, "Bit range end out of bounds");
        assert!(range.start <= range.end, "Invalid bit range");

        let width = range.end - range.start;
        let value_mask = if width == 32 {
            u32::MAX as u64
        } else {
            (1u64 << width) - 1
        };

        assert!(value <= value_mask, "Value too large for bit range");

        let mask = if range.end == 32 {
            u32::MAX
        } else {
            (1 << range.end) - 1
        } ^ ((1 << range.start) - 1);

        *self = (*self & !mask) | (((value as u32) << range.start) & mask);
    }
}

#[inline(always)]
pub fn sign_extend_64(value: u64, num_bits: u64) -> u64 {
    let shift_amount = 64 - num_bits; // 扩展到64位
    // 将符号位移到最高位，然后算术右移
    ((value << shift_amount) as i64 >> shift_amount) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_operations_u32() {
        let mut x = 0u32;

        // Test single bit operations
        x.set_bit(3, true);
        assert!(x.bit(3));
        assert_eq!(x, 0b1000);

        x.set_bit(3, false);
        assert!(!x.bit(3));
        assert_eq!(x, 0);

        // Test bit range operations
        x.set_bit_range(4..8, 0b1010);
        assert_eq!(x.bit_range(4..8), 0b1010);
        assert_eq!(x, 0b10100000);

        // Test overlapping ranges
        x.set_bit_range(6..10, 0b1100);
        assert_eq!(x.bit_range(6..10), 0b1100);
        assert_eq!(x, 0b1100100000);
    }

    #[test]
    #[should_panic(expected = "Bit position out of bounds")]
    fn test_bit_out_of_bounds_u32() {
        0u32.bit(32);
    }

    #[test]
    #[should_panic(expected = "Bit range end out of bounds")]
    fn test_bit_range_out_of_bounds_u32() {
        0u32.bit_range(30..33);
    }

    #[test]
    #[should_panic(expected = "Value too large for bit range")]
    fn test_value_too_large_u32() {
        let mut x = 0u32;
        x.set_bit_range(0..4, 16); // 16 needs 5 bits
    }

    #[test]
    fn test_bit_boundaries_u32() {
        let mut x = 0u32;
        x.set_bit(0, true);
        x.set_bit(31, true);
        assert!(x.bit(0));
        assert!(x.bit(31));
        x.set_bit(0, false);
        x.set_bit(31, false);
        assert!(!x.bit(0));
        assert!(!x.bit(31));
    }

    #[test]
    fn test_bit_range_boundaries_u32() {
        let mut x = 0u32;
        x.set_bit_range(0..0, 0);
        assert_eq!(x.bit_range(0..0), 0);
        x.set_bit_range(31..32, 1);
        assert_eq!(x.bit_range(31..32), 1);
        x.set_bit_range(0..32, u32::MAX as u64);
        assert_eq!(x.bit_range(0..32), u32::MAX as u64);
    }

    #[test]
    fn test_repeat_set_clear_u32() {
        let mut x = 0u32;
        x.set_bit(5, true);
        x.set_bit(5, true);
        assert!(x.bit(5));
        x.set_bit(5, false);
        x.set_bit(5, false);
        assert!(!x.bit(5));
    }

    #[test]
    fn test_repeat_set_bit_range_u32() {
        let mut x = 0u32;
        x.set_bit_range(10..20, 0b1010101010);
        assert_eq!(x.bit_range(10..20), 0b1010101010);
        x.set_bit_range(12..18, 0b111111);
        assert_eq!(x.bit_range(10..20), 0b1011111110);
    }

    #[test]
    fn test_all_ones_and_zeros_u32() {
        let mut x = 0u32;
        x.set_bit_range(0..32, u32::MAX as u64);
        for i in 0..32 {
            assert!(x.bit(i));
        }
        x.set_bit_range(0..32, 0);
        for i in 0..32 {
            assert!(!x.bit(i));
        }
    }

    #[test]
    fn test_empty_and_full_range_u32() {
        let x = 0x12345678u32;
        assert_eq!(x.bit_range(10..10), 0);
        assert_eq!(x.bit_range(0..32), x as u64);
    }

    #[test]
    fn test_set_bit_range_zero_and_max_u32() {
        let mut x = 0u32;
        x.set_bit_range(10..20, 0);
        assert_eq!(x.bit_range(10..20), 0);
        x.set_bit_range(10..20, (1 << 10) - 1);
        assert_eq!(x.bit_range(10..20), (1 << 10) - 1);
    }

    #[test]
    fn test_combined_operations_u32() {
        let mut x = 0u32;
        x.set_bit_range(8..16, 0b10101010);
        x.set_bit(10, false);
        assert_eq!(x.bit_range(8..16), 0b10101010); // 第10位本来就是0
        x.set_bit(10, true);
        assert_eq!(x.bit_range(8..16), 0b10101110);
        x.set_bit(12, true);
        assert_eq!(x.bit_range(8..16), 0b10111110);
        assert_eq!(x, 0b10111110 << 8);
    }

    #[test]
    fn test_extreme_values_u32() {
        let mut x = u32::MAX;
        x.set_bit(0, false);
        x.set_bit(31, false);
        assert!(!x.bit(0));
        assert!(!x.bit(31));
        assert_eq!(x.bit_range(1..31), (1u64 << 30) - 1);
        x.set_bit_range(0..32, 0);
        assert_eq!(x, 0);
        x.set_bit_range(0..32, u32::MAX as u64);
        assert_eq!(x, u32::MAX);
    }

    #[test]
    fn test_bit_operations() {
        let mut x = 0u64;
        
        // Test single bit operations
        x.set_bit(3, true);
        assert!(x.bit(3));
        assert_eq!(x, 0b1000);
        
        x.set_bit(3, false);
        assert!(!x.bit(3));
        assert_eq!(x, 0);
        
        // Test bit range operations
        x.set_bit_range(4..8, 0b1010);
        assert_eq!(x.bit_range(4..8), 0b1010);
        assert_eq!(x, 0b10100000);
        
        // Test overlapping ranges
        x.set_bit_range(6..10, 0b1100);
        assert_eq!(x.bit_range(6..10), 0b1100);
        assert_eq!(x, 0b1100100000);
    }

    #[test]
    #[should_panic(expected = "Bit position out of bounds")]
    fn test_bit_out_of_bounds() {
        0u64.bit(64);
    }

    #[test]
    #[should_panic(expected = "Bit range end out of bounds")]
    fn test_bit_range_out_of_bounds() {
        0u64.bit_range(60..65);
    }

    #[test]
    #[should_panic(expected = "Value too large for bit range")]
    fn test_value_too_large() {
        let mut x = 0u64;
        x.set_bit_range(0..4, 16); // 16 needs 5 bits
    }

    #[test]
    fn test_bit_boundaries() {
        let mut x = 0u64;
        x.set_bit(0, true);
        x.set_bit(63, true);
        assert!(x.bit(0));
        assert!(x.bit(63));
        x.set_bit(0, false);
        x.set_bit(63, false);
        assert!(!x.bit(0));
        assert!(!x.bit(63));
    }

    #[test]
    fn test_bit_range_boundaries() {
        let mut x = 0u64;
        x.set_bit_range(0..0, 0);
        assert_eq!(x.bit_range(0..0), 0);
        x.set_bit_range(63..64, 1);
        assert_eq!(x.bit_range(63..64), 1);
        assert!(x.bit(63));
        x.set_bit_range(5..6, 1);
        assert_eq!(x.bit_range(5..6), 1);
        assert!(x.bit(5));
        x.set_bit_range(0..64, u64::MAX);
        assert_eq!(x.bit_range(0..64), u64::MAX);
    }

    #[test]
    fn test_repeat_set_clear() {
        let mut x = 0u64;
        x.set_bit(5, true);
        x.set_bit(5, true);
        assert!(x.bit(5));
        x.set_bit(5, false);
        x.set_bit(5, false);
        assert!(!x.bit(5));
    }

    #[test]
    fn test_repeat_set_bit_range() {
        let mut x = 0u64;
        x.set_bit_range(10..20, 0b1010101010);
        assert_eq!(x.bit_range(10..20), 0b1010101010);
        x.set_bit_range(12..18, 0b111111);
        assert_eq!(x.bit_range(10..20), 0b1011111110);
    }

    #[test]
    fn test_all_ones_and_zeros() {
        let mut x = 0u64;
        x.set_bit_range(0..64, u64::MAX);
        for i in 0..64 {
            assert!(x.bit(i));
        }
        x.set_bit_range(0..64, 0);
        for i in 0..64 {
            assert!(!x.bit(i));
        }
    }

    #[test]
    fn test_empty_and_full_range() {
        let x = 0x1234567890abcdefu64;
        assert_eq!(x.bit_range(10..10), 0);
        assert_eq!(x.bit_range(0..64), x);
    }

    #[test]
    fn test_set_bit_range_zero_and_max() {
        let mut x = 0u64;
        x.set_bit_range(10..20, 0);
        assert_eq!(x.bit_range(10..20), 0);
        x.set_bit_range(10..20, (1 << 10) - 1);
        assert_eq!(x.bit_range(10..20), (1 << 10) - 1);
    }

    #[test]
    fn test_combined_operations() {
        let mut x = 0u64;
        x.set_bit_range(8..16, 0b10101010);
        x.set_bit(10, false);
        assert_eq!(x.bit_range(8..16), 0b10101010); // 第10位本来就是0
        x.set_bit(10, true);
        assert_eq!(x.bit_range(8..16), 0b10101110);
        x.set_bit(12, true);
        assert_eq!(x.bit_range(8..16), 0b10111110);
        assert_eq!(x, 0b10111110 << 8);
    }

    // #[test]
    // #[should_panic(expected = "Invalid bit range")]
    // fn test_invalid_range_start_gt_end() {
    //     let _ = 0u64.bit_range(3..2);
    // }

    // #[test]
    // #[should_panic(expected = "Invalid bit range")]
    // fn test_set_bit_range_start_gt_end() {
    //     let mut x = 0u64;
    //     x.set_bit_range(3..2, 0);
    // }

    #[test]
    fn test_sign_extend_64() {
        // 8位正数
        assert_eq!(sign_extend_64(0x7F, 8), 0x7F);
        // 8位负数
        assert_eq!(sign_extend_64(0x80, 8), 0xFFFFFFFFFFFFFF80);
        // 12位正数
        assert_eq!(sign_extend_64(0x7FF, 12), 0x7FF);
        // 12位负数
        assert_eq!(sign_extend_64(0x800, 12), 0xFFFFFFFFFFFFF800);
        // 16位正数
        assert_eq!(sign_extend_64(0x7FFF, 16), 0x7FFF);
        // 16位负数
        assert_eq!(sign_extend_64(0x8000, 16), 0xFFFFFFFFFFFF8000);
        // 32位正数
        assert_eq!(sign_extend_64(0x7FFFFFFF, 32), 0x7FFFFFFF);
        // 32位负数
        assert_eq!(sign_extend_64(0x80000000, 32), 0xFFFFFFFF80000000);
        // 63位正数
        assert_eq!(sign_extend_64(0x7FFFFFFFFFFFFFFF, 63), 0xFFFFFFFFFFFFFFFF);
        // 63位负数
        assert_eq!(sign_extend_64(0x4000000000000000, 63), 0xC000000000000000);
        // 64位正数
        assert_eq!(sign_extend_64(0x7FFFFFFFFFFFFFFF, 64), 0x7FFFFFFFFFFFFFFF);
        // 64位（不变）
        assert_eq!(sign_extend_64(0xFFFFFFFFFFFFFFFF, 64), 0xFFFFFFFFFFFFFFFF);
        assert_eq!(sign_extend_64(0x0, 64), 0x0);
    }

    #[test]
    fn test_extreme_values() {
        let mut x = u64::MAX;
        x.set_bit(0, false);
        x.set_bit(63, false);
        assert!(!x.bit(0));
        assert!(!x.bit(63));
        assert_eq!(x.bit_range(1..63), (1u64 << 62) - 1);
        x.set_bit_range(0..64, 0);
        assert_eq!(x, 0);
        x.set_bit_range(0..64, u64::MAX);
        assert_eq!(x, u64::MAX);
    }
}
