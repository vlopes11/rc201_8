use crate::mem::{Mem, MemError, MemErrorVariant};
use crate::oper::{Oper, OperCode};
use crate::keypad::{Keypad, Key};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::ops::Bound::*;
use std::ops::RangeBounds;
use std::slice::SliceIndex;

const MEM_SIZE: usize = 4096_usize;
const REG_SIZE: usize = 16_usize;
const STK_SIZE: usize = 16_usize;
const KEY_SIZE: usize = 16_usize;

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
    /// Internal memory
    mem: [u8; MEM_SIZE],

    /// CPU registers
    reg: [u8; REG_SIZE],

    /// Memory pointer
    ind: u16,

    /// Program counter
    cnt: u16,

    /// Random generator
    rng: ThreadRng,

    /// Delay timer
    dtm: u8,

    /// Sound timer
    stm: u8,
    
    /// Call stack
    stk: [u16; STK_SIZE],
    
    /// Stack pointer
    spt: usize,
    
    /// Keypad
    key: [u8; KEY_SIZE]
}

impl Emu {
    ///
    /// Returns a new Emu instance
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    ///
    /// let mut emu = Emu::new();
    /// ```
    ///
    pub fn new() -> Emu {
        Emu {
            mem: [0; MEM_SIZE],
            reg: [0; REG_SIZE],
            ind: 0,
            cnt: 0,
            rng: rand::thread_rng(),
            dtm: 0,
            stm: 0,
            stk: [0; STK_SIZE],
            spt: 0,
            key: [0; KEY_SIZE],
        }
    }

    ///
    /// Executes an operation from a given code
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    ///
    /// let mut emu = Emu::new();
    ///
    /// // Clear the display
    /// emu.recv_opcode(&(0x00E0 as u16));
    /// ```
    ///
    pub fn recv_opcode(&mut self, code: &u16) {
        match Oper::from_code(code, &REG_SIZE) {
            OperCode::Display00E0 => {}
            OperCode::Flow00EE => {}
            OperCode::Flow1NNN(v) => {
                self.cnt = v;
            }
            OperCode::Flow2NNN(_) => {}
            OperCode::Cond3XNN(_, _) => {}
            OperCode::Cond4XNN(_, _) => {}
            OperCode::Cond5XY0(_, _) => {}
            OperCode::Const6XNN(x, v) => {
                self.reg[x] = v;
            }
            OperCode::Const7XNN(x, v) => {
                self.reg[x] = self.reg[x] + v;
            }
            OperCode::Assign8XY0(x, y) => {
                self.reg[x] = self.reg[y];
            }
            OperCode::BitOp8XY1(x, y) => {
                self.reg[x] = self.reg[x] | self.reg[y];
            }
            OperCode::BitOp8XY2(x, y) => {
                self.reg[x] = self.reg[x] & self.reg[y];
            }
            OperCode::BitOp8XY3(x, y) => {
                self.reg[x] = self.reg[x] ^ self.reg[y];
            }
            OperCode::Math8XY4(_, _) => {}
            OperCode::Math8XY5(_, _) => {}
            OperCode::BitOp8XY6(_, _) => {}
            OperCode::Math8XY7(_, _) => {}
            OperCode::BitOp8XYE(_, _) => {}
            OperCode::Cond9XY0(_, _) => {}
            OperCode::MemANNN(v) => {
                self.ind = v;
            }
            OperCode::FlowBNNN(v) => {
                self.cnt = v + self.reg[0] as u16;
            }
            OperCode::RandCXNN(x, v) => {
                let r: u8 = self.rng.gen_range(0, 255);
                self.reg[x] = r & v;
            }
            OperCode::DisplayDXYN(_, _, _) => {}
            OperCode::KeyOpEX9E(_) => {}
            OperCode::KeyOpEXA1(_) => {}
            OperCode::TimerFX07(x) => {
                self.reg[x] = self.dtm;
            }
            OperCode::KeyOpFX0A(_) => {}
            OperCode::TimerFX15(x) => {
                self.dtm = self.reg[x];
            }
            OperCode::SoundFX18(x) => {
                self.stm = self.reg[x];
            }
            OperCode::MemFX1E(x) => {
                self.ind = self.ind + self.reg[x] as u16;
            }
            OperCode::MemFX29(_) => {}
            OperCode::BcdFX33(_) => {}
            OperCode::MemFX55(_) => {}
            OperCode::MemFX65(_) => {}
            OperCode::Unknown => {}
        }
    }
}

impl Mem for Emu {
    /// Returns the maximum memory size
    fn max_size(&self) -> usize {
        MEM_SIZE
    }

    /// Validates if a given index belongs to the memory range
    fn validate_index(&self, index: &usize) -> bool {
        index < &self.max_size()
    }

    /// Returns a tuple (start, end) for a given range
    fn range_get_start_end<T: RangeBounds<usize> + SliceIndex<[u8]> + Clone>(
        &self,
        range: T,
    ) -> Result<(usize, usize), MemError> {
        let start = match range.start_bound() {
            Included(i) => i.clone(),
            Excluded(i) => i + 1,
            Unbounded => 0,
        };
        let end = match range.end_bound() {
            Included(i) => i.clone(),
            Excluded(i) => i.clone(),
            Unbounded => self.max_size(),
        };
        if self.validate_index(&start) && self.validate_index(&(end - 1)) {
            Ok((start, end))
        } else {
            Err(MemError::new(MemErrorVariant::AccessRangeViolation(
                start, end,
            )))
        }
    }

    /// Validates if a given range belongs to the memory range
    fn validate_range<T: RangeBounds<usize> + SliceIndex<[u8]> + Clone>(&self, range: T) -> bool {
        match self.range_get_start_end(range) {
            Ok((start, end)) => end > start,
            _ => false,
        }
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
        match self.range_get_start_end(range.clone()) {
            Ok(_) => Ok(&self.mem[range]),
            Err(e) => Err(e),
        }
    }

    /// Replace the given range with a given slice
    fn mem_write<T: RangeBounds<usize> + SliceIndex<[u8]> + Clone>(
        &mut self,
        range: T,
        slice: &[u8],
    ) -> Result<(), MemError> {
        match self.range_get_start_end(range.clone()) {
            Ok((start, _)) => {
                for (i, v) in slice.iter().enumerate() {
                    self.mem[start + i] = v.clone();
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl Keypad for Emu {
    fn key_pressed(&self, _key: &Key) -> bool {
        // TODO
        false
    }
}
