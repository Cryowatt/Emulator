use crate::system::ConsoleDevices;

pub trait BusDevice {
    fn read(self: &Self, address: u16) -> u8;
    fn write(self: &mut Self, address: u16, data: u8);
}

pub type MemoryMapper = fn(u16, &mut ConsoleDevices) -> &mut dyn BusDevice;

pub struct Bus {
    //pub(crate) deviceMap: [&'a dyn BusDevice; 8],
    pub memory_map: MemoryMapper,
    //0 RAM
    //1 RAM
    //2 PPU regs
    //3 PPU regs
    //4 APU/cart expansion rom
    //5 APU/cart expansion rom
    //6 SRAM
    //7 SRAM
    //8 PRG-ROM
    //9 PRG-ROM
    //10 PRG-ROM
    //11 PRG-ROM
    //12 PRG-ROM
    //13 PRG-ROM
    //14 PRG-ROM
    //15 PRG-ROM
}