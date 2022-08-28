pub struct Mos6502 {
    pub a: u8,
    pub pc: u16,
    pub p: u8,
    pub s: u8,
    pub x: u8,
    pub y: u8,
}

pub trait RP2A03 {
    fn new() -> Mos6502;
    fn zero_page_indexed() {

    }
}

impl RP2A03 for Mos6502 {
    fn new() -> Mos6502 {
        Mos6502 {
            a: 0x00,
            pc: 0x0000,
            p: 0x34,
            s: 0xFD,
            x: 0x00,
            y: 0x00,
        }
    }
}

fn cycle() {

}
