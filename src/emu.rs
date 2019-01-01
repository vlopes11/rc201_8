use crate::display::{Display, DisplayDrawResult, DisplayEmu};
use crate::keypad::{Key, Keypad};
use crate::mem::{Mem, MemError, MemErrorVariant};
use crate::oper::{Oper, OperCode};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::ops::Bound::*;
use std::ops::RangeBounds;
use std::slice::SliceIndex;

const MEM_SIZE: usize = 4096_usize;
const REG_SIZE: usize = 16_usize;
const STK_SIZE: usize = 16_usize;
const KEY_SIZE: usize = 16_usize;
const PRG_INCR: u16 = 2_u16;

///
/// Main emulator structure
///
/// # Example
///
/// ```
/// use rc201_8::emu::Emu;
/// use rc201_8::mem::{Mem, MemError};
/// use rc201_8::display::{Display, DisplayDummy};
///
/// fn try_main() -> Result<(), MemError> {
///     let mut emu = Emu::new(DisplayDummy::new());
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
pub struct Emu<D: Display + Sized> {
    /// Internal memory
    mem: [u8; MEM_SIZE],

    /// CPU registers
    reg: [u8; REG_SIZE],

    /// Memory pointer
    ind: usize,

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
    key: [bool; KEY_SIZE],

    /// Display
    dsp: D,
}

impl<D: Display + Sized> Emu<D> {
    ///
    /// Returns a new Emu instance
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::display::{Display, DisplayDummy};
    ///
    /// let mut emu = Emu::new(DisplayDummy::new());
    /// ```
    ///
    pub fn new(display: D) -> Emu<D> {
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
            key: [false; KEY_SIZE],
            dsp: display,
        }
    }

    ///
    /// Set the VF register to a given value
    ///
    fn set_reg_vf(&mut self, vf: u8) {
        self.reg[REG_SIZE - 1] = vf;
    }

    ///
    /// Skip next program instruction
    ///
    fn skip_next_instruction(&mut self) {
        self.cnt = self.cnt + PRG_INCR;
    }

    ///
    /// Executes an operation from a given code
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::display::{Display, DisplayDummy};
    ///
    /// let mut emu = Emu::new(DisplayDummy::new());
    ///
    /// // Clear the display
    /// emu.recv_opcode(&(0x00E0 as u16));
    /// ```
    ///
    pub fn recv_opcode(&mut self, code: &u16) {
        match Oper::from_code(code, &REG_SIZE) {
            OperCode::Display00E0 => {
                self.dsp.clear();
            }
            OperCode::Flow00EE => {
                // TODO - Catch stack underflow
                self.spt = self.spt - 1;
                self.cnt = self.stk[self.spt];
            }
            OperCode::Flow1NNN(v) => {
                self.cnt = v;
            }
            OperCode::Flow2NNN(n) => {
                // TODO - Catch stack overflow
                self.stk[self.spt] = self.cnt;
                self.spt = self.spt + 1;
                self.cnt = n;
            }
            OperCode::Cond3XNN(x, n) => {
                // TODO - Create a cpu mod and encapsulate register access
                if self.reg[x] == n {
                    self.skip_next_instruction();
                }
            }
            OperCode::Cond4XNN(x, n) => {
                if self.reg[x] != n {
                    self.skip_next_instruction();
                }
            }
            OperCode::Cond5XY0(x, y) => {
                if self.reg[x] == self.reg[y] {
                    self.skip_next_instruction();
                }
            }
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
            OperCode::Math8XY4(x, y) => {
                let sum = self.reg[x] + self.reg[y];
                let vf = if (sum as u16) > 0xFF { 1 } else { 0 };
                self.set_reg_vf(vf);
                self.reg[x] = sum;
            }
            OperCode::Math8XY5(x, y) => {
                let (vx, vy) = (self.reg[x], self.reg[y]);
                let dif: i8 = vx as i8 - vy as i8;
                self.reg[x] = dif as u8;
                let vf = if dif < 0 { 1 } else { 0 };
                self.set_reg_vf(vf);
            }
            OperCode::BitOp8XY6(x, _) => {
                let vf = self.reg[x] & 0x1;
                self.set_reg_vf(vf);
                self.reg[x] >>= 1;
            }
            OperCode::Math8XY7(x, y) => {
                let (vx, vy) = (self.reg[x], self.reg[y]);
                let dif: i8 = vy as i8 - vx as i8;
                self.reg[x] = dif as u8;
                let vf = if dif < 0 { 1 } else { 0 };
                self.set_reg_vf(vf);
            }
            OperCode::BitOp8XYE(x, _) => {
                let vf = self.reg[x] & 0x80;
                self.set_reg_vf(vf);
                self.reg[x] <<= 1;
            }
            OperCode::Cond9XY0(x, y) => {
                if self.reg[x] != self.reg[y] {
                    self.skip_next_instruction();
                }
            }
            OperCode::MemANNN(v) => {
                self.ind = v as usize;
            }
            OperCode::FlowBNNN(v) => {
                self.cnt = v + self.reg[0] as u16;
            }
            OperCode::RandCXNN(x, v) => {
                let r: u8 = self.rng.gen_range(0, 255);
                self.reg[x] = r & v;
            }
            OperCode::DisplayDXYN(x, y, height) => {
                let vf = match self.dsp.draw(&x, &y, &height) {
                    DisplayDrawResult::Collision => 1,
                    DisplayDrawResult::Free => 0,
                };
                self.set_reg_vf(vf);
            }
            OperCode::KeyOpEX9E(x) => {
                let key = self.key_from_u8(&self.reg[x]);
                if self.key_pressed(&key) {
                    self.skip_next_instruction();
                }
            }
            OperCode::KeyOpEXA1(x) => {
                let key = self.key_from_u8(&self.reg[x]);
                if !self.key_pressed(&key) {
                    self.skip_next_instruction();
                }
            }
            OperCode::TimerFX07(x) => {
                self.reg[x] = self.dtm;
            }
            OperCode::KeyOpFX0A(x) => match self.any_key_pressed() {
                Some(k) => self.reg[x] = self.key_to_u8(&k),
                None => self.cnt = self.cnt - PRG_INCR,
            },
            OperCode::TimerFX15(x) => {
                self.dtm = self.reg[x];
            }
            OperCode::SoundFX18(x) => {
                self.stm = self.reg[x];
            }
            OperCode::MemFX1E(x) => {
                self.ind = self.ind + self.reg[x] as usize;
            }
            OperCode::MemFX29(x) => {
                self.ind = self.reg[x] as usize * 5;
            }
            OperCode::BcdFX33(x) => {
                let (i, v) = (self.ind, self.reg[x]);
                self.mem_put(&i, v / 100).unwrap();
                self.mem_put(&(i + 1), (v / 10) % 10).unwrap();
                self.mem_put(&(i + 2), (v % 100) % 10).unwrap();
            }
            OperCode::MemFX55(x) => {
                for i in 0..((self.reg[x] + 1) as usize) {
                    self.mem_put(&(self.ind + i), self.reg[i]).unwrap()
                }
            }
            OperCode::MemFX65(x) => {
                let vx = self.reg[x];
                for i in 0..((vx + 1) as usize) {
                    self.reg[i] = self.mem_get(&(self.ind + i)).unwrap().clone();
                }
            }
            OperCode::Unknown => {}
        }
    }
}

impl<D: Display + Sized> Mem for Emu<D> {
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

impl<D: Display + Sized> Keypad for Emu<D> {
    fn key_from_u8(&self, k: &u8) -> Key {
        match k {
            0x1 => Key::K1,
            0x2 => Key::K2,
            0x3 => Key::K3,
            0x4 => Key::K4,
            0x5 => Key::K5,
            0x6 => Key::K6,
            0x7 => Key::K7,
            0x8 => Key::K8,
            0x9 => Key::K9,
            0xA => Key::KA,
            0xB => Key::KB,
            0xC => Key::KC,
            0xD => Key::KD,
            0xE => Key::KE,
            0xF => Key::KF,
            _ => Key::Unknown,
        }
    }

    fn key_to_u8(&self, k: &Key) -> u8 {
        match k {
            Key::K1 => 0x1,
            Key::K2 => 0x2,
            Key::K3 => 0x3,
            Key::K4 => 0x4,
            Key::K5 => 0x5,
            Key::K6 => 0x6,
            Key::K7 => 0x7,
            Key::K8 => 0x8,
            Key::K9 => 0x9,
            Key::KA => 0xA,
            Key::KB => 0xB,
            Key::KC => 0xC,
            Key::KD => 0xD,
            Key::KE => 0xE,
            Key::KF => 0xF,
            Key::Unknown => 0x0,
        }
    }

    fn key_to_index(&self, k: &Key) -> Option<usize> {
        match k {
            Key::K1 => Some(0),
            Key::K2 => Some(1),
            Key::K3 => Some(2),
            Key::K4 => Some(3),
            Key::K5 => Some(4),
            Key::K6 => Some(5),
            Key::K7 => Some(6),
            Key::K8 => Some(7),
            Key::K9 => Some(8),
            Key::KA => Some(9),
            Key::KB => Some(10),
            Key::KC => Some(11),
            Key::KD => Some(12),
            Key::KE => Some(13),
            Key::KF => Some(14),
            Key::Unknown => None,
        }
    }

    fn key_pressed(&self, key: &Key) -> bool {
        match self.key_to_index(key) {
            Some(i) => self.key[i],
            None => false
        }
    }

    fn any_key_pressed(&self) -> Option<Key> {
        if self.key_pressed(&Key::K1) {
            Some(Key::K1)
        } else if self.key_pressed(&Key::K2) {
            Some(Key::K2)
        } else if self.key_pressed(&Key::K3) {
            Some(Key::K3)
        } else if self.key_pressed(&Key::K4) {
            Some(Key::K4)
        } else if self.key_pressed(&Key::K5) {
            Some(Key::K5)
        } else if self.key_pressed(&Key::K6) {
            Some(Key::K6)
        } else if self.key_pressed(&Key::K7) {
            Some(Key::K7)
        } else if self.key_pressed(&Key::K8) {
            Some(Key::K8)
        } else if self.key_pressed(&Key::K9) {
            Some(Key::K9)
        } else if self.key_pressed(&Key::KA) {
            Some(Key::KA)
        } else if self.key_pressed(&Key::KB) {
            Some(Key::KB)
        } else if self.key_pressed(&Key::KC) {
            Some(Key::KC)
        } else if self.key_pressed(&Key::KD) {
            Some(Key::KD)
        } else if self.key_pressed(&Key::KE) {
            Some(Key::KE)
        } else if self.key_pressed(&Key::KF) {
            Some(Key::KF)
        } else {
            None
        }
    }
}

impl<D: Display + Sized> DisplayEmu<D> for Emu<D> {
    fn set_display(&mut self, display: D) {
        self.dsp = display;
    }
}
