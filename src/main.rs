use crate::emu::Emu;
use crate::mem::Mem;

mod emu;
mod mem;

fn main() {
    let mut emu = Emu::new();
    emu.mem_put(&0, 1).unwrap();
    emu.mem_write(4090..4096, &[3, 4]).unwrap();
    emu.mem_write(0..1, &[3]).unwrap();
    println!("{}", emu.mem_get(&2).unwrap());
    println!("{:?}", emu.mem_read(..).unwrap());
    println!("{:?}", emu.mem_read(1..3).unwrap());
    println!("{:?}", emu.mem_read(..3).unwrap());
    println!("{:?}", emu.mem_read(25..37).unwrap());
    println!("The begining and end of this range is {:?}", emu.range_get_start_end(..));
}
