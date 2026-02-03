// KEY PATTERN: Apply no_std only when NOT testing
#![cfg_attr(not(test), no_std)]

// Conditional imports: std::Vec for tests, heapless::Vec for no_std
#[cfg(test)]
use std::vec::Vec;

#[cfg(not(test))]
use heapless::Vec;

/// Simple data buffer with different backing storage based on compilation mode
pub struct DataBuffer<const N: usize> {
    #[cfg(test)]
    data: Vec<u32>,  // Dynamic allocation in tests

    #[cfg(not(test))]
    data: Vec<u32, N>,  // Fixed-size for embedded
}

impl<const N: usize> DataBuffer<N> {
    pub const fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }

    pub fn push(&mut self, value: u32) -> bool {
        #[cfg(test)]
        {
            // Simulate capacity limit in test mode
            if self.data.len() >= N {
                return false;
            }
            self.data.push(value);
            true
        }

        #[cfg(not(test))]
        {
            self.data.push(value).is_ok()
        }
    }

    pub fn sum(&self) -> u32 {
        self.data.iter().sum()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer() {
        let mut buffer = DataBuffer::<3>::new();

        assert!(buffer.push(10));
        assert!(buffer.push(20));
        assert!(buffer.push(30));
        assert!(!buffer.push(40)); // Should fail - at capacity

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.sum(), 60);
    }
}