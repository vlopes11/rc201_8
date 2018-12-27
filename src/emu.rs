use crate::mem::{Mem, MemError, MemErrorVariant};
use std::ops::Bound::*;
use std::ops::RangeBounds;
use std::slice::SliceIndex;

const MEM_SIZE: usize = 4096_usize;

/// 
/// Main emulator structure
///
/// # Example
///
/// ```
/// use rc201_8::emu::Emu;
/// use rc201_8::mem::{Mem, MemError};
///
/// fn try_main() -> Result<(), MemError> {
///     let mut emu = Emu::new();
///     emu.mem_put(&0, 1).unwrap();
///     emu.mem_write(4090..4096, &[3, 4]).unwrap();
///     emu.mem_write(0..1, &[3]).unwrap();
///     println!("{:?}", emu.mem_read(..).unwrap());
///     println!("{:?}", emu.mem_read(1..3).unwrap());
///     println!("{:?}", emu.mem_read(..3).unwrap());
///     println!("{:?}", emu.mem_read(1..4096).unwrap());
///     Ok(())
/// }
///
/// fn main() {
///     try_main().unwrap();
/// }
/// ```
///
pub struct Emu {
    mem: [u8; MEM_SIZE],
}

impl Emu {
    /// 
    /// Returns a new Emu instance
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// let mut emu = Emu::new();
    /// ```
    ///
    pub fn new() -> Emu {
        Emu { mem: [0; MEM_SIZE] }
    }
}

impl Mem for Emu {
    /// Returns the maximum memory size
    fn max_size(&self) -> usize {
        MEM_SIZE
    }
    
    /// Returns a tuple (start, end) for a given range
    fn range_get_start_end<T: RangeBounds<usize> + SliceIndex<[u8]> + Clone>(
        &self,
        range: T,
    ) -> (usize, usize) {
        let start = match range.start_bound() {
            Included(i) => i.clone(),
            Excluded(i) => i + 1,
            Unbounded => 0,
        };
        let end = match range.end_bound() {
            Included(i) => i.clone(),
            Excluded(i) => i + 1,
            Unbounded => self.max_size(),
        };
        (start, end)
    }
    
    /// Validates if a given index belongs to the memory range
    fn validate_index(&self, index: &usize) -> bool {
        index >= &0 && index < &MEM_SIZE
    }
    
    /// Validates if a given range belongs to the memory range
    fn validate_range<T: RangeBounds<usize> + SliceIndex<[u8]> + Clone>(&self, range: T) -> bool {
        let (start, end) = self.range_get_start_end(range);
        start < MEM_SIZE && end <= MEM_SIZE + 1 && end >= start
    }

    /// Get the memory content of a given index
    fn mem_get(&self, index: &usize) -> Result<&u8, MemError> {
        if self.validate_index(index) {
            Ok(&self.mem[*index])
        } else {
            Err(MemError::new(MemErrorVariant::AccessViolation(
                index.clone(),
            )))
        }
    }

    /// Put a given value in a given index of the memory range
    fn mem_put(&mut self, index: &usize, value: u8) -> Result<(), MemError> {
        if self.validate_index(index) {
            self.mem[*index] = value;
            Ok(())
        } else {
            Err(MemError::new(MemErrorVariant::AccessViolation(
                index.clone(),
            )))
        }
    }

    /// Return a memory slice for a given range
    fn mem_read<T: RangeBounds<usize> + SliceIndex<[u8]> + Clone>(
        &self,
        range: T,
    ) -> Result<&<T as SliceIndex<[u8]>>::Output, MemError> {
        if self.validate_range(range.clone()) {
            Ok(&self.mem[range])
        } else {
            let (s, e) = self.range_get_start_end(range);
            Err(MemError::new(MemErrorVariant::AccessRangeViolation(s, e)))
        }
    }

    /// Replace the given range with a given slice
    fn mem_write<T: RangeBounds<usize> + SliceIndex<[u8]> + Clone>(
        &mut self,
        range: T,
        slice: &[u8],
    ) -> Result<(), MemError> {
        let start = match range.start_bound() {
            Included(i) => i.clone(),
            Excluded(i) => i + 1,
            Unbounded => 0,
        };
        let end = match range.end_bound() {
            Included(i) => i.clone(),
            Excluded(i) => i + 1,
            Unbounded => 0,
        };
        if start < MEM_SIZE && end <= MEM_SIZE + 1 && end >= start {
            for (i, v) in slice.iter().enumerate() {
                self.mem[start + i] = v.clone();
            }
            Ok(())
        } else {
            Err(MemError::new(MemErrorVariant::AccessRangeViolation(
                start, end,
            )))
        }
    }
}
