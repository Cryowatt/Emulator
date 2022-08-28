use std::{fs::File, path::Path};
use nes::roms::RomImage;

#[test]
fn basic_load_test() {
    let path = Path::new("./nes-test-roms/instr_test-v5/rom_singles/01-basics.nes");
    let mut rom_file = File::open(path).expect("Test rom open error");
    let image = RomImage::from(&mut rom_file).unwrap();
    assert_eq!(image.header.program_rom_size, 2);
}