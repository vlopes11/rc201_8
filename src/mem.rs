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
    /// use rc201_8::display::{Display, DisplayDummy};
    ///
    /// let emu = Emu::new(DisplayDummy::new());
    ///
    /// assert_eq!(emu.max_size(), 4096);
    /// ```
    ///
    fn max_size(&self) -> usize;

    ///
    /// Validates if a given index belongs to the memory range
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::mem::Mem;
    /// use rc201_8::display::{Display, DisplayDummy};
    ///
    /// let emu = Emu::new(DisplayDummy::new());
    /// let index = 15000;
    ///
    /// assert!(! emu.validate_index(&index));
    /// ```
    ///
    fn validate_index(&self, index: &usize) -> bool;

    ///
    /// Returns a tuple (start, end) for a given range
    ///
    /// End will be the last index + 1
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::mem::Mem;
    /// use rc201_8::display::{Display, DisplayDummy};
    ///
    /// let emu = Emu::new(DisplayDummy::new());
    ///
    /// assert_eq!(emu.range_get_start_end(..).unwrap(), (0, 4096));
    /// ```
    ///
    fn range_get_start_end<T: RangeBounds<usize> + SliceIndex<[u8]> + Clone>(
        &self,
        range: T,
    ) -> Result<(usize, usize), MemError>;

    ///
    /// Validates if a given range belongs to the memory range
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::mem::Mem;
    /// use rc201_8::display::{Display, DisplayDummy};
    ///
    /// let emu = Emu::new(DisplayDummy::new());
    ///
    /// assert!(! emu.validate_range(..15000));
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
    /// use rc201_8::display::{Display, DisplayDummy};
    ///
    /// let emu = Emu::new(DisplayDummy::new());
    ///
    /// assert_eq!(emu.mem_get(&2_usize).unwrap(), &0_u8);
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
    /// use rc201_8::display::{Display, DisplayDummy};
    ///
    /// let mut emu = Emu::new(DisplayDummy::new());
    /// let index = 15;
    /// let value = 25;
    ///
    /// emu.mem_put(&index, value).unwrap();
    /// assert_eq!(emu.mem_get(&index).unwrap(), &25_u8);
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
    /// use rc201_8::display::{Display, DisplayDummy};
    ///
    /// let emu = Emu::new(DisplayDummy::new());
    ///
    /// // Full memory slice ]..[
    /// assert_eq!(emu.mem_read(..).unwrap().len(), 4096);
    /// 
    /// // First 3 members [0-2]
    /// assert_eq!(emu.mem_read(..3).unwrap().len(), 3);
    ///
    /// // From the 3rd member to the end [3..[
    /// assert_eq!(emu.mem_read(3..).unwrap().len(), 4093);
    ///
    /// // Partial slice [25..37]
    /// assert_eq!(emu.mem_read(25..37).unwrap().len(), 12);
    /// ```
    ///
    fn mem_read<T: RangeBounds<usize> + SliceIndex<[u8]> + Clone>(
        &self,
        range: T,
    ) -> Result<&<T as SliceIndex<[u8]>>::Output, MemError>;

    ///
    /// Replace the given range with a given slice.
    ///
    /// If the given slice contains less members than the given range,
    /// only the given slice members will be inserted into the memory
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::mem::Mem;
    /// use rc201_8::display::{Display, DisplayDummy};
    ///
    /// let mut emu = Emu::new(DisplayDummy::new());
    ///
    /// emu.mem_write(4090..4096, &[3, 4]).unwrap();
    /// assert_eq!(emu.mem_get(&4090_usize).unwrap(), &3_u8);
    /// assert_eq!(emu.mem_get(&4091_usize).unwrap(), &4_u8);
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
