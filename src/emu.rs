use crate::cpu::{Cpu, CpuError, CpuErrorVariant};
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
}

impl<D: Display + Sized> Cpu for Emu<D> {
    /// Executes an operation from a given code
    fn recv_opcode(&mut self, code: &u16) -> Result<(), CpuError> {
        match Oper::from_code(code, &REG_SIZE) {
            OperCode::Display00E0 => {
                self.dsp.clear();
                Ok(())
            }
            OperCode::Flow00EE => {
                self.cnt = self.stk_pop().unwrap();
                Ok(())
            }
            OperCode::Flow1NNN(v) => {
                self.cnt = v;
                Ok(())
            }
            OperCode::Flow2NNN(n) => {
                self.stk_push(n).unwrap();
                self.cnt = n;
                Ok(())
            }
            OperCode::Cond3XNN(x, n) => {
                if self.reg_get(&x).unwrap() == n {
                    self.skip_next_instruction();
                }
                Ok(())
            }
            OperCode::Cond4XNN(x, n) => {
                if self.reg_get(&x).unwrap() != n {
                    self.skip_next_instruction();
                }
                Ok(())
            }
            OperCode::Cond5XY0(x, y) => {
                if self.reg_get(&x).unwrap() == self.reg_get(&y).unwrap() {
                    self.skip_next_instruction();
                }
                Ok(())
            }
            OperCode::Const6XNN(x, v) => {
                self.reg_put(&x, v).unwrap();
                Ok(())
            }
            OperCode::Const7XNN(x, v) => {
                let vx = self.reg_get(&x).unwrap();
                self.reg_put(&x, vx + v).unwrap();
                Ok(())
            }
            OperCode::Assign8XY0(x, y) => {
                let vy = self.reg_get(&y).unwrap();
                self.reg_put(&x, vy).unwrap();
                Ok(())
            }
            OperCode::BitOp8XY1(x, y) => {
                let vx = self.reg_get(&x).unwrap();
                let vy = self.reg_get(&y).unwrap();
                self.reg_put(&x, vx | vy).unwrap();
                Ok(())
            }
            OperCode::BitOp8XY2(x, y) => {
                let vx = self.reg_get(&x).unwrap();
                let vy = self.reg_get(&y).unwrap();
                self.reg_put(&x, vx & vy).unwrap();
                Ok(())
            }
            OperCode::BitOp8XY3(x, y) => {
                let vx = self.reg_get(&x).unwrap();
                let vy = self.reg_get(&y).unwrap();
                self.reg_put(&x, vx ^ vy).unwrap();
                Ok(())
            }
            OperCode::Math8XY4(x, y) => {
                let vx = self.reg_get(&x).unwrap();
                let vy = self.reg_get(&y).unwrap();
                let sum = vx + vy;
                let vf = if (sum as u16) > 0xFF { 1 } else { 0 };
                self.reg_put_vf(vf);
                self.reg_put(&x, sum).unwrap();
                Ok(())
            }
            OperCode::Math8XY5(x, y) => {
                let vx = self.reg_get(&x).unwrap();
                let vy = self.reg_get(&y).unwrap();
                let dif: i8 = vx as i8 - vy as i8;
                self.reg_put(&x, dif as u8).unwrap();
                let vf = if dif < 0 { 1 } else { 0 };
                self.reg_put_vf(vf);
                Ok(())
            }
            OperCode::BitOp8XY6(x, _) => {
                let mut vx = self.reg_get(&x).unwrap();
                let vf = vx & 0x1;
                self.reg_put_vf(vf);
                vx >>= 1;
                self.reg_put(&x, vx).unwrap();
                Ok(())
            }
            OperCode::Math8XY7(x, y) => {
                let vx = self.reg_get(&x).unwrap();
                let vy = self.reg_get(&y).unwrap();
                let dif: i8 = vy as i8 - vx as i8;
                self.reg_put(&x, dif as u8).unwrap();
                let vf = if dif < 0 { 1 } else { 0 };
                self.reg_put_vf(vf);
                Ok(())
            }
            OperCode::BitOp8XYE(x, _) => {
                let mut vx = self.reg_get(&x).unwrap();
                let vf = vx & 0x80;
                self.reg_put_vf(vf);
                vx <<= 1;
                self.reg_put(&x, vx).unwrap();
                Ok(())
            }
            OperCode::Cond9XY0(x, y) => {
                let vx = self.reg_get(&x).unwrap();
                let vy = self.reg_get(&y).unwrap();
                if vx != vy {
                    self.skip_next_instruction();
                }
                Ok(())
            }
            OperCode::MemANNN(v) => {
                self.ind = v as usize;
                Ok(())
            }
            OperCode::FlowBNNN(v) => {
                let v0 = self.reg_get(&0).unwrap() as u16;
                self.cnt = v + v0;
                Ok(())
            }
            OperCode::RandCXNN(x, v) => {
                let r: u8 = self.rng.gen_range(0, 255);
                self.reg_put(&x, r & v).unwrap();
                Ok(())
            }
            OperCode::DisplayDXYN(x, y, height) => {
                let vf = match self.dsp.draw(&x, &y, &height) {
                    DisplayDrawResult::Collision => 1,
                    DisplayDrawResult::Free => 0,
                };
                self.reg_put_vf(vf);
                Ok(())
            }
            OperCode::KeyOpEX9E(x) => {
                let vx = self.reg_get(&x).unwrap();
                let key = self.key_from_u8(&vx);
                if self.key_pressed(&key) {
                    self.skip_next_instruction();
                }
                Ok(())
            }
            OperCode::KeyOpEXA1(x) => {
                let vx = self.reg_get(&x).unwrap();
                let key = self.key_from_u8(&vx);
                if !self.key_pressed(&key) {
                    self.skip_next_instruction();
                }
                Ok(())
            }
            OperCode::TimerFX07(x) => {
                self.reg_put(&x, self.dtm).unwrap();
                Ok(())
            }
            OperCode::KeyOpFX0A(x) => {
                match self.any_key_pressed() {
                    Some(k) => self.reg_put(&x, self.key_to_u8(&k)).unwrap(),
                    None => self.cnt = self.cnt - PRG_INCR,
                }
                Ok(())
            }
            OperCode::TimerFX15(x) => {
                let vx = self.reg_get(&x).unwrap();
                self.dtm = vx;
                Ok(())
            }
            OperCode::SoundFX18(x) => {
                let vx = self.reg_get(&x).unwrap();
                self.stm = vx;
                Ok(())
            }
            OperCode::MemFX1E(x) => {
                let vx = self.reg_get(&x).unwrap() as usize;
                self.ind = self.ind + vx;
                Ok(())
            }
            OperCode::MemFX29(x) => {
                let vx = self.reg_get(&x).unwrap() as usize;
                self.ind = vx * 5;
                Ok(())
            }
            OperCode::BcdFX33(x) => {
                let vx = self.reg_get(&x).unwrap();
                let ind = self.ind;
                self.mem_put(&ind, vx / 100).unwrap();
                self.mem_put(&(ind + 1), (vx / 10) % 10).unwrap();
                self.mem_put(&(ind + 2), (vx % 100) % 10).unwrap();
                Ok(())
            }
            OperCode::MemFX55(x) => {
                let vx = self.reg_get(&x).unwrap();
                let mi = (vx + 1) as usize;
                for i in 0..mi {
                    let vi = self.reg_get(&i).unwrap();
                    self.mem_put(&(self.ind + i), vi).unwrap()
                }
                Ok(())
            }
            OperCode::MemFX65(x) => {
                let vx = self.reg_get(&x).unwrap();
                for i in 0..((vx + 1) as usize) {
                    let v = self.mem_get(&(self.ind + i)).unwrap().clone();
                    self.reg_put(&i, v).unwrap();
                }
                Ok(())
            }
            OperCode::Unknown => Err(CpuError::new(CpuErrorVariant::InvalidOperationCode(*code))),
        }
    }

    /// Skip next processing instruction
    fn skip_next_instruction(&mut self) {
        self.cnt = self.cnt + PRG_INCR;
    }

    ///
    /// Validate register index
    ///
    fn validate_register_index(&self, index: &usize) -> bool {
        index < &REG_SIZE
    }

    /// Return the value for a register
    fn reg_get(&self, index: &usize) -> Result<u8, CpuError> {
        if self.validate_register_index(index) {
            Ok(self.reg[*index])
        } else {
            Err(CpuError::new(CpuErrorVariant::InvalidRegisterIndex(*index)))
        }
    }

    /// Put the value on a register
    fn reg_put(&mut self, index: &usize, value: u8) -> Result<(), CpuError> {
        if self.validate_register_index(index) {
            self.reg[*index] = value;
            Ok(())
        } else {
            Err(CpuError::new(CpuErrorVariant::InvalidRegisterIndex(*index)))
        }
    }

    /// Put the value on the last register
    fn reg_put_vf(&mut self, value: u8) {
        self.reg[REG_SIZE - 1] = value;
    }

    /// Return the stack pointer
    fn spt_get(&self) -> usize {
        self.spt
    }

    /// Validate a given stack pointer
    fn spt_validate(&self, spt: &usize) -> Result<usize, CpuError> {
        if spt < &STK_SIZE {
            Ok(*spt)
        } else {
            Err(CpuError::new(CpuErrorVariant::StackOverflow(*spt)))
        }
    }

    /// Increment the stack pointer
    fn spt_inc(&mut self) -> Result<usize, CpuError> {
        let spt = self.spt_validate(&(self.spt_get() + 1)).unwrap();
        self.spt = spt;
        Ok(self.spt)
    }

    /// Decrement the stack pointer
    fn spt_dec(&mut self) -> Result<usize, CpuError> {
        let spt = self.spt_validate(&(self.spt_get() - 1)).unwrap();
        self.spt = spt;
        Ok(self.spt)
    }

    /// Return a stack pointer
    fn stk_get(&self) -> Result<u16, CpuError> {
        Ok(self.stk[self.spt_get()])
    }

    /// Pop the address from the stack
    fn stk_pop(&mut self) -> Result<u16, CpuError> {
        let stk = self.stk_get().unwrap();
        self.spt_dec().unwrap();
        Ok(stk)
    }

    /// Push the address on the stack
    fn stk_push(&mut self, address: u16) -> Result<(), CpuError> {
        let spt = self.spt_inc().unwrap();
        self.stk[spt] = address;
        Ok(())
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
            None => false,
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
