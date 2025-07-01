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
    fn bit(&self, pos: usize) -> bool {
        assert!(pos < 64, "Bit position out of bounds");
        (self & (1 << pos)) != 0
    }

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

    fn set_bit(&mut self, pos: usize, value: bool) {
        assert!(pos < 64, "Bit position out of bounds");
        if value {
            *self |= 1 << pos;
        } else {
            *self &= !(1 << pos);
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

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
