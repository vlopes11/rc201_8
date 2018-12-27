use std::fmt;
use std::ops::RangeBounds;
use std::slice::SliceIndex;

/// 
/// Encapsulation of memory interactions
///
pub trait Mem {
    ///
    /// Returns the maximum memory size
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::mem::Mem;
    ///
    /// let emu = Emu::new();
    ///
    /// println!("The number of memory slots is {}", emu.max_size());
    /// ```
    ///
    fn max_size(&self) -> usize;

    /// 
    /// Returns a tuple (start, end) for a given range
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::mem::Mem;
    ///
    /// let emu = Emu::new();
    ///
    /// println!("The begining and end of this range is {:?}", emu.range_get_start_end(..));
    /// ```
    ///
    fn range_get_start_end<T: RangeBounds<usize> + SliceIndex<[u8]> + Clone>(
        &self,
        range: T,
    ) -> (usize, usize);

    /// 
    /// Validates if a given index belongs to the memory range
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::mem::Mem;
    ///
    /// let emu = Emu::new();
    /// let index = 15000;
    ///
    /// if emu.validate_index(&index) {
    ///     println!("{} belongs to the memory range", index);
    /// }
    /// else {
    ///     println!("{} dont belongs to the memory range", index);
    /// }
    /// ```
    ///
    fn validate_index(&self, index: &usize) -> bool;

    ///
    /// Validates if a given range belongs to the memory range
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::mem::Mem;
    ///
    /// let emu = Emu::new();
    ///
    /// if emu.validate_range(..15000) {
    ///     println!("The given range belongs to the memory range");
    /// }
    /// else {
    ///     println!("The given range dont belongs to the memory range");
    /// }
    /// ```
    ///
    fn validate_range<T: RangeBounds<usize> + SliceIndex<[u8]> + Clone>(&self, range: T) -> bool;

    /// 
    /// Get the memory content of a given index
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::mem::Mem;
    ///
    /// let emu = Emu::new();
    ///
    /// println!("{}", emu.mem_get(&2).unwrap());
    /// ```
    ///
    fn mem_get(&self, index: &usize) -> Result<&u8, MemError>;

    /// 
    /// Put a given value in a given index of the memory range
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::mem::Mem;
    ///
    /// let mut emu = Emu::new();
    /// let index = 15;
    /// let value = 25;
    ///
    /// emu.mem_put(&index, value).unwrap();
    /// println!("{}", emu.mem_get(&index).unwrap());
    /// ```
    ///
    fn mem_put(&mut self, index: &usize, value: u8) -> Result<(), MemError>;

    ///
    /// Return a memory slice for a given range
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::mem::Mem;
    ///
    /// let mut emu = Emu::new();
    ///
    /// println!("Full memory slice - {:?}", emu.mem_read(..).unwrap());
    /// println!("First 3 members - {:?}", emu.mem_read(..3).unwrap());
    /// println!("From the 3rd member to the end - {:?}", emu.mem_read(3..).unwrap());
    /// println!("Partial slice - {:?}", emu.mem_read(25..37).unwrap());
    /// ```
    ///
    fn mem_read<T: RangeBounds<usize> + SliceIndex<[u8]> + Clone>(
        &self,
        range: T,
    ) -> Result<&<T as SliceIndex<[u8]>>::Output, MemError>;

    ///
    /// Replace the given range with a given slice.
    ///
    /// If the given slice contains less members than the given range, only the given slice members
    /// will be inserted into the memory
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::mem::Mem;
    ///
    /// let mut emu = Emu::new();
    ///
    /// emu.mem_write(4090..4096, &[3, 4]).unwrap();
    /// ```
    ///
    fn mem_write<T: RangeBounds<usize> + SliceIndex<[u8]> + Clone>(
        &mut self,
        range: T,
        slice: &[u8],
    ) -> Result<(), MemError>;
}

///
/// Possible variants for memory access error
///
#[derive(Debug)]
pub enum MemErrorVariant {
    ///
    /// Variant for illegal index memory access
    ///
    AccessViolation(usize),

    ///
    /// Variant for not contained range in memory access
    ///
    AccessRangeViolation(usize, usize),
}

impl fmt::Display for MemErrorVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// 
/// Memory errors implementation
///
pub struct MemError {
    variant: MemErrorVariant,
    message: String,
}

impl MemError {
    /// 
    /// Returns a new MemError instance
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::mem::{MemError, MemErrorVariant};
    ///
    /// let invalid_index = MemError::new(MemErrorVariant::AccessViolation(25));
    /// let invalid_range = MemError::new(MemErrorVariant::AccessRangeViolation(25, 36));
    /// ```
    ///
    pub fn new(param: MemErrorVariant) -> MemError {
        let (variant, message) = match param {
            MemErrorVariant::AccessViolation(a) => (param, format!("Illegal address '{}'!", a)),
            MemErrorVariant::AccessRangeViolation(a, b) => (
                MemErrorVariant::AccessRangeViolation(a, b - 1),
                format!("Illegal range '{}..{}'!", a, b - 1),
            ),
        };
        MemError { variant, message }
    }
}

impl fmt::Display for MemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Debug for MemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MemError {{ variant: {}, message: {} }}",
            self.variant, self.message
        )
    }
}
