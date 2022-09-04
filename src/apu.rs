//#[struct_layout::explicit(size = 1, align = 1)]
use crate::bus::BusDevice;

pub struct Alu2A03 {
    // #[field(offset = 0)]
    // pub sq1Vol: u8,
    // #[field(offset = 1]
    // pub sq1Sweep: u8,
    // #[field(offset = 2)]
    // pub sq1Lo: u8,
    // #[field(offset = 3)]
    // pub sq1Hi: u8,

    // #[field(offset = 4)]
    // pub sq2Vol: u8,
    // #[field(offset = 5)]
    // pub sq2Sweep: u8,
    // #[field(offset = 6)]
    // pub sq2Lo: u8,
    // #[field(offset = 7)]
    // pub sq2Hi: u8,

    // #[field(offset = 8)]
    // pub triLinear: u8,
    // #[field(offset = 10)]
    // pub triLo: u8,
    // #[field(offset = 11)]
    // pub triHigh: u8,
    
    // #[field(offset = 12)]
    // pub noiseVol: u8,
    // #[field(offset = 14)]
    // pub noiseLo: u8,
    // #[field(offset = 15)]
    // pub noiseHi: u8,

    // #[field(offset = 16)]
    // pub dmcFreq: u8,
    // #[field(offset = 17)]
    // pub dmcRaw: u8,
    // #[field(offset = 18)]
    // pub dmcStart: u8,
    // #[field(offset = 19)]
    // pub dmcLen: u8,

    // #[field(offset = 20)]
    // pub oamDma: u8,
    // #[field(offset = 21)]
    // pub sndChn: u8,

    // #[field(offset = 22)]
    // pub joy1: u8,
    // #[field(offset = 23)]
    // pub joy2: u8,
}

impl BusDevice for Alu2A03{
    fn read(&self, address: u16) -> u8 {
        println!("APU READ!! ${:04X}", address);
        0
    }

    fn write(&mut self, address: u16, _: u8) { 
        println!("APU WRITE!! ${:04X}", address);}
}