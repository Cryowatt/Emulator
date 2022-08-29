use crate::bus::BusDevice;

pub struct RAM<const SIZE: usize> {
    pub bank: Box<[u8; SIZE]>,
    mask: u16,
}

impl<const SIZE: usize> RAM<SIZE> {
    // pub fn new() -> Self {
    //     let foo = vec![0, SIZE];
    //     RAM {
    //         bank: Box::new([0; SIZE]), 
    //         mask: SIZE as u16
    //     }
    // }

    pub fn new(mask: u16) -> Self {
        RAM { 
            bank: Box::new([0; SIZE]),
            mask,
        }
    }
}

impl<const SIZE: usize> BusDevice for RAM<SIZE> {
    fn read(&self, address: u16) -> u8 {
        (*self.bank)[(address & self.mask) as usize]
    }

    fn write(self: &mut Self, address: u16, data: u8) {
        (*self.bank)[(address & self.mask) as usize] = data;
    }
}
