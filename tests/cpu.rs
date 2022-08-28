#[cfg(test)]
mod test {
    #[test]
    fn power_on_state() {
        //let cpu = crate::nes::cpu::RP2A03::new();
        // let cpu = <Mos6502 as RP2A03>::new();
        // assert_eq!(cpu.p, 0x34);
        // assert_eq!(cpu.a, 0x00);
        // assert_eq!(cpu.x, 0x00);
        // assert_eq!(cpu.y, 0x00);
        // assert_eq!(cpu.s, 0xFD);
        /*
        P = $34[1] (IRQ disabled)[2]
        A, X, Y = 0
        S = $FD[3]
        $4017 = $00 (frame irq enabled)
        $4015 = $00 (all channels disabled)
        $4000-$400F = $00
        $4010-$4013 = $00 [4]
        All 15 bits of noise channel LFSR = $0000[5]. The first time the LFSR is clocked from the all-0s state, it will shift in a 1.
        2A03G: APU Frame Counter reset. (but 2A03letterless: APU frame counter powers up at a value equivalent to 15)
        Internal memory ($0000-$07FF) has unreliable startup state. Some machines may have consistent RAM contents at power-on, but others do not.
        Emulators often implement a consistent RAM startup state (e.g. all $00 or $FF, or a particular pattern), and flash carts like the PowerPak may partially or fully initialize RAM before starting a program, so an NES programmer must be careful not to rely on the startup contents of RAM.
        */
    }
}