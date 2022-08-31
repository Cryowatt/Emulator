pub trait Address {
    fn set_low(&mut self, value: u8);
    fn set_high(&mut self, value: u8);
    fn get_low(&mut self) -> u8;
    fn get_high(&mut self) -> u8;
    fn from_high_low(high: u8, low: u8) -> u16;
}

impl Address for u16 {
    fn set_low(&mut self, value: u8) {
        *self = (*self & 0xff00) | value as u16;
    }

    fn set_high(&mut self, value: u8) {
        *self = (*self & 0x00ff) | ((value as u16) << 8);
    }

    fn get_low(&mut self) -> u8 {
        (*self & 0xff) as u8
    }

    fn get_high(&mut self) -> u8 {
        (*self >> 8 & 0xff) as u8
    }

    fn from_high_low(high: u8, low: u8) -> u16 {
        ((high as u16) << 8) | low as u16
    }
}