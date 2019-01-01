use std::fmt;

pub trait Cpu {
    ///
    /// Receive an operation code and process it
    ///
    fn recv_opcode(&mut self, code: &u16) -> Result<(), CpuError>;

    ///
    /// Skip next processing instruction
    ///
    fn skip_next_instruction(&mut self);

    ///
    /// Validate register index
    ///
    fn validate_register_index(&self, index: &usize) -> bool;

    ///
    /// Return the value for a register
    ///
    fn reg_get(&self, index: &usize) -> Result<u8, CpuError>;

    ///
    /// Put the value on a register
    ///
    fn reg_put(&mut self, index: &usize, value: u8) -> Result<(), CpuError>;

    ///
    /// Put the value on the last register
    ///
    fn reg_put_vf(&mut self, value: u8);

    ///
    /// Validate a given stack pointer
    ///
    fn spt_validate(&self, spt: &usize) -> Result<usize, CpuError>;

    ///
    /// Return the stack pointer
    ///
    fn spt_get(&self) -> usize;

    ///
    /// Increment the stack pointer
    ///
    fn spt_inc(&mut self) -> Result<usize, CpuError>;

    ///
    /// Decrement the stack pointer
    ///
    fn spt_dec(&mut self) -> Result<usize, CpuError>;

    ///
    /// Return a stack pointer
    ///
    fn stk_get(&self) -> Result<u16, CpuError>;

    ///
    /// Pop the address from the stack
    ///
    fn stk_pop(&mut self) -> Result<u16, CpuError>;

    ///
    /// Push the address on the stack
    ///
    fn stk_push(&mut self, address: u16) -> Result<(), CpuError>;
}

///
/// Possible variants for Cpu operations error
///
#[derive(Debug)]
pub enum CpuErrorVariant {
    ///
    /// Variant for illegal register index access
    ///
    InvalidRegisterIndex(usize),

    ///
    /// Variant for invalid operation code
    ///
    InvalidOperationCode(u16),

    ///
    /// Stack overflow
    ///
    StackOverflow(usize),
}

impl fmt::Display for CpuErrorVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

///
/// Cpu errors implementation
///
pub struct CpuError {
    variant: CpuErrorVariant,
    message: String,
}

impl CpuError {
    ///
    /// Returns a new CpuError instance
    ///
    /// # Example
    ///
    /// ```
    /// use rc201_8::emu::Emu;
    /// use rc201_8::cpu::{CpuError, CpuErrorVariant};
    ///
    /// let invalid_register_index = CpuError::new(CpuErrorVariant::InvalidRegisterIndex(28));
    /// ```
    ///
    pub fn new(param: CpuErrorVariant) -> CpuError {
        let (variant, message) = match param {
            CpuErrorVariant::InvalidRegisterIndex(a) => {
                (param, format!("Illegal register index '{}'!", a))
            }
            CpuErrorVariant::InvalidOperationCode(a) => {
                (param, format!("Illegal operation code '{}'!", a))
            }
            CpuErrorVariant::StackOverflow(a) => (param, format!("Stack overflow '{}'!", a)),
        };
        CpuError { variant, message }
    }
}

impl fmt::Display for CpuError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Debug for CpuError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "MemError {{ variant: {}, message: {} }}",
            self.variant, self.message
        )
    }
}
