pub trait Display {
    fn new() -> Self;
}

pub trait DisplayEmu<D: Display + Sized> {
    fn set_display(&mut self, display: D);
}

pub struct DisplayDummy {}
impl Display for DisplayDummy {
    fn new() -> DisplayDummy {
        DisplayDummy {}
    }
}
