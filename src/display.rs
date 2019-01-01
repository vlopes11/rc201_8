pub enum DisplayDrawResult {
    Collision,
    Free,
}

pub trait Display {
    fn new() -> Self;
    fn clear(&mut self);
    fn draw(&mut self, x: &usize, y: &usize, height: &u8) -> DisplayDrawResult;
    fn refresh(&mut self);
}

pub trait DisplayEmu<D: Display + Sized> {
    fn set_display(&mut self, display: D);
}

pub struct DisplayDummy {}
impl Display for DisplayDummy {
    fn new() -> DisplayDummy {
        DisplayDummy {}
    }

    fn clear(&mut self) {}

    fn draw(&mut self, _: &usize, _: &usize, _: &u8) -> DisplayDrawResult {
        DisplayDrawResult::Free
    }

    fn refresh(&mut self) {}
}
