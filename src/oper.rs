pub enum OperCode {
    /// 00E0-Display
    ///
    /// Clears the screen
    Display00E0,

    /// 00EE-Flow
    ///
    /// Returns from a subroutine.
    Flow00EE,

    /// 1NNN-Flow
    ///
    /// Jumps to address NNN.
    Flow1NNN(u16),

    /// 2NNN-Flow
    ///
    /// Calls subroutine at NNN.
    Flow2NNN(u16),

    /// 3XNN-Cond
    ///
    /// Skips the next instruction if VX equals NN.
    Cond3XNN(usize, u8),

    /// 4XNN-Cond
    ///
    /// Skips the next instruction if VX doesn't equal NN.
    Cond4XNN(usize, u8),

    /// 5XY0-Cond
    ///
    /// Skips the next instruction if VX equals VY.
    Cond5XY0(usize, usize),

    /// 6XNN-Const
    ///
    /// Sets VX to NN.
    Const6XNN(usize, u8),

    /// 7XNN-Const
    ///
    /// Adds NN to VX. (Carry flag is not changed)
    Const7XNN(usize, u8),

    /// 8XY0-Assign
    ///
    /// Sets VX to the value of VY.
    Assign8XY0(usize, usize),

    /// 8XY1-BitOp
    ///
    /// Sets VX to VX or VY. (Bitwise OR operation)
    BitOp8XY1(usize, usize),

    /// 8XY2-BitOp
    ///
    /// Sets VX to VX and VY. (Bitwise AND operation)
    BitOp8XY2(usize, usize),

    /// 8XY3-BitOp
    ///
    /// Sets VX to VX xor VY.
    BitOp8XY3(usize, usize),

    /// 8XY4-Math
    ///
    /// Adds VY to VX. VF is set to 1 when there's a carry, and
    /// to 0 when there isn't.
    Math8XY4(usize, usize),

    /// 8XY5-Math
    ///
    /// VY is subtracted from VX. VF is set to 0 when there's a
    /// borrow, and 1 when there isn't.
    Math8XY5(usize, usize),

    /// 8XY6-BitOp
    ///
    /// Stores the least significant bit of VX in VF and then
    /// shifts VX to the right by 1.
    BitOp8XY6(usize, usize),

    /// 8XY7-Math
    ///
    /// Sets VX to VY minus VX. VF is set to 0 when there's a
    /// borrow, and 1 when there isn't.
    Math8XY7(usize, usize),

    /// 8XYE-BitOp
    ///
    /// Stores the most significant bit of VX in VF and then
    /// shifts VX to the left by 1.
    BitOp8XYE(usize, usize),

    /// 9XY0-Cond
    ///
    /// Skips the next instruction if VX doesn't equal VY.
    Cond9XY0(usize, usize),

    /// ANNN-MEM
    ///
    /// Sets I to the address NNN.
    MemANNN(u16),

    /// BNNN-Flow
    ///
    /// Jumps to the address NNN plus V0.
    FlowBNNN(u16),

    /// CXNN-Rand
    ///
    /// Sets VX to the result of a bitwise and operation on a
    /// random number (Typically: 0 to 255) and NN.
    RandCXNN(usize, u8),

    /// DXYN-Disp
    ///
    /// Draws a sprite at coordinate (VX, VY) that has a width of
    /// 8 pixels and a height of N pixels. Each row of 8 pixels
    /// is read as bit-coded starting from memory location I; I
    /// value doesn’t change after the execution of this
    /// instruction. As described above, VF is set to 1 if any
    /// screen pixels are flipped from set to unset when the
    /// sprite is drawn, and to 0 if that doesn’t happen
    DisplayDXYN(usize, usize, u8),

    /// EX9E-KeyOp
    ///
    /// Skips the next instruction if the key stored in VX is
    /// pressed.
    KeyOpEX9E(usize),

    /// EXA1-KeyOp
    ///
    /// Skips the next instruction if the key stored in VX isn't
    /// pressed.
    KeyOpEXA1(usize),

    /// FX07-Timer
    ///
    /// Sets VX to the value of the delay timer.
    TimerFX07(usize),

    /// FX0A-KeyOp
    ///
    /// A key press is awaited, and then stored in VX. (Blocking
    /// Operation. All instruction halted until next key event)
    KeyOpFX0A(usize),

    /// FX15-Timer
    ///
    /// Sets the delay timer to VX.
    TimerFX15(usize),

    /// FX18-Sound
    ///
    /// Sets the delay timer to VX.
    SoundFX18(usize),

    /// FX1E-MEM
    ///
    /// Adds VX to I.
    MemFX1E(usize),

    /// FX29-MEM
    ///
    /// Sets I to the location of the sprite for the character in
    /// VX. Characters 0-F (in hexadecimal) are represented by a
    /// 4x5 font.
    MemFX29(usize),

    /// FX33-BCD
    ///
    /// Stores the binary-coded decimal representation of VX, with
    /// the most significant of three digits at the address in I,
    /// the middle digit at I plus 1, and the least significant
    /// digit at I plus 2. (In other words, take the decimal
    /// representation of VX, place the hundreds digit in memory
    /// at location in I, the tens digit at location I+1, and the
    /// ones digit at location I+2.)
    BcdFX33(usize),

    /// FX55-MEM
    ///
    /// Stores V0 to VX (including VX) in memory starting at
    /// address I. The offset from I is increased by 1 for each
    /// value written, but I itself is left unmodified.
    MemFX55(usize),

    /// FX65-MEM
    ///
    /// Fills V0 to VX (including VX) with values from memory
    /// starting at address I. The offset from I is increased by 1
    /// for each value written, but I itself is left
    /// unmodified.
    MemFX65(usize),

    /// Not defined operation
    Unknown,
}

pub struct Oper {}

impl Oper {
    /// Returns enum OperCode from a given code and
    /// the registers capacity (typically 16)
    pub fn from_code(code: &u16, rsize: &usize) -> OperCode {
        let a = (code & 0xF000) >> 12;
        let b = (code & 0x0F00) >> 08;
        let c = (code & 0x00F0) >> 04;
        let d = code & 0x000F;

        match (a, b, c, d) {
            (0, 0, 0xE, 0) => OperCode::Display00E0,
            (0, 0, 0xE, 0xE) => OperCode::Flow00EE,
            (0x1, _, _, _) => OperCode::Flow1NNN(get_nnn(code)),
            (0x2, _, _, _) => OperCode::Flow2NNN(get_nnn(code)),
            (0x3, _, _, _) => OperCode::Cond3XNN(get_x(code, rsize), get_nn(code)),
            (0x4, _, _, _) => OperCode::Cond4XNN(get_x(code, rsize), get_nn(code)),
            (0x5, _, _, _) => OperCode::Cond5XY0(get_x(code, rsize), get_y(code, rsize)),
            (0x6, _, _, _) => OperCode::Const6XNN(get_x(code, rsize), get_nn(code)),
            (0x7, _, _, _) => OperCode::Const7XNN(get_x(code, rsize), get_nn(code)),
            (0x8, _, _, 0x0) => OperCode::Assign8XY0(get_x(code, rsize), get_y(code, rsize)),
            (0x8, _, _, 0x1) => OperCode::BitOp8XY1(get_x(code, rsize), get_y(code, rsize)),
            (0x8, _, _, 0x2) => OperCode::BitOp8XY2(get_x(code, rsize), get_y(code, rsize)),
            (0x8, _, _, 0x3) => OperCode::BitOp8XY3(get_x(code, rsize), get_y(code, rsize)),
            (0x8, _, _, 0x4) => OperCode::Math8XY4(get_x(code, rsize), get_y(code, rsize)),
            (0x8, _, _, 0x5) => OperCode::Math8XY5(get_x(code, rsize), get_y(code, rsize)),
            (0x8, _, _, 0x6) => OperCode::BitOp8XY6(get_x(code, rsize), get_y(code, rsize)),
            (0x8, _, _, 0x7) => OperCode::Math8XY7(get_x(code, rsize), get_y(code, rsize)),
            (0x8, _, _, 0xE) => OperCode::BitOp8XYE(get_x(code, rsize), get_y(code, rsize)),
            (0x9, _, _, _) => OperCode::Cond9XY0(get_x(code, rsize), get_y(code, rsize)),
            (0xA, _, _, _) => OperCode::MemANNN(get_nnn(code)),
            (0xB, _, _, _) => OperCode::FlowBNNN(get_nnn(code)),
            (0xC, _, _, _) => OperCode::RandCXNN(get_x(code, rsize), get_nn(code)),
            (0xD, _, _, _) => {
                OperCode::DisplayDXYN(get_x(code, rsize), get_y(code, rsize), get_n(code))
            }
            (0xE, _, 0x9, 0xE) => OperCode::KeyOpEX9E(get_x(code, rsize)),
            (0xE, _, 0xA, 0x1) => OperCode::KeyOpEXA1(get_x(code, rsize)),
            (0xF, _, 0x0, 0x7) => OperCode::TimerFX07(get_x(code, rsize)),
            (0xF, _, 0x0, 0xA) => OperCode::KeyOpFX0A(get_x(code, rsize)),
            (0xF, _, 0x1, 0x5) => OperCode::TimerFX15(get_x(code, rsize)),
            (0xF, _, 0x1, 0x8) => OperCode::SoundFX18(get_x(code, rsize)),
            (0xF, _, 0x1, 0xE) => OperCode::MemFX1E(get_x(code, rsize)),
            (0xF, _, 0x2, 0x9) => OperCode::MemFX29(get_x(code, rsize)),
            (0xF, _, 0x3, 0x3) => OperCode::BcdFX33(get_x(code, rsize)),
            (0xF, _, 0x5, 0x5) => OperCode::MemFX55(get_x(code, rsize)),
            (0xF, _, 0x6, 0x5) => OperCode::MemFX65(get_x(code, rsize)),
            (_, _, _, _) => OperCode::Unknown,
        }
    }
}

fn get_x(code: &u16, rsize: &usize) -> usize {
    let x = ((code & 0x0F00) >> 8) as usize;
    if &x >= rsize {
        panic!("Register overflow!");
    }
    x
}

fn get_y(code: &u16, rsize: &usize) -> usize {
    let y = ((code & 0x00F0) >> 4) as usize;
    if &y >= rsize {
        panic!("Register overflow!");
    }
    y
}

fn get_n(code: &u16) -> u8 {
    (code & 0x000F) as u8
}

fn get_nn(code: &u16) -> u8 {
    (code & 0x00FF) as u8
}

fn get_nnn(code: &u16) -> u16 {
    code & 0x0FFF
}
