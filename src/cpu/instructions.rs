use crate::address::Address;

use super::{Mos6502, Status};

pub trait MOS6502Instructions {
    fn adc(&mut self, data: u8);
    fn and(&mut self, data: u8);
    fn asl(&mut self, data: u8);
    fn bcc(&mut self, data: u8);
    fn bcs(&mut self, data: u8);
    fn beq(&mut self, data: u8);
    fn bit(&mut self, data: u8);
    fn bmi(&mut self, data: u8);
    fn bne(&mut self, data: u8);
    fn bpl(&mut self, data: u8);
    fn brk(&mut self);
    fn bvc(&mut self, data: u8);
    fn bvs(&mut self, data: u8);
    fn clc(&mut self, data: u8);
    fn cld(&mut self, data: u8);
    fn cli(&mut self, data: u8);
    fn clv(&mut self, data: u8);
    fn cmp(&mut self, data: u8);
    fn cpx(&mut self, data: u8);
    fn cpy(&mut self, data: u8);
    fn dec(&mut self, data: u8);
    fn dex(&mut self, data: u8);
    fn dey(&mut self, data: u8);
    fn eor(&mut self, data: u8);
    fn inc(&mut self, data: u8);
    fn inx(&mut self, data: u8);
    fn iny(&mut self, data: u8);
    fn jmp(&mut self);
    fn jsr(&mut self);
    fn lda(&mut self, data: u8);
    fn ldx(&mut self, data: u8);
    fn ldy(&mut self, data: u8);
    fn lsr(&mut self, data: u8);
    fn nop(&mut self, data: u8);
    fn ora(&mut self, data: u8);
    fn pha(&mut self, data: u8);
    fn php(&mut self, data: u8);
    fn pla(&mut self, data: u8);
    fn plp(&mut self, data: u8);
    fn rol(&mut self, data: u8);
    fn ror(&mut self, data: u8);
    fn rti(&mut self, data: u8);
    fn rts(&mut self, data: u8);
    fn sbc(&mut self, data: u8);
    fn sec(&mut self, data: u8);
    fn sed(&mut self, data: u8);
    fn sei(&mut self, data: u8);
    fn sta(&mut self) -> u8;
    fn stx(&mut self) -> u8;
    fn sty(&mut self) -> u8;
    fn tax(&mut self, data: u8);
    fn tay(&mut self, data: u8);
    fn tsx(&mut self, data: u8);
    fn txa(&mut self, data: u8);
    fn txs(&mut self, data: u8);
    fn tya(&mut self, data: u8);
}

impl MOS6502Instructions for Mos6502 {
    fn adc(&mut self, data: u8) {
        todo!()
    }

    fn and(&mut self, data: u8) {
        todo!()
    }

    fn asl(&mut self, data: u8) {
        todo!()
    }

    fn bcc(&mut self, data: u8) {
        todo!()
    }

    fn bcs(&mut self, data: u8) {
        todo!()
    }

    fn beq(&mut self, data: u8) {
        todo!()
    }

    fn bit(&mut self, data: u8) {
        todo!()
    }

    fn bmi(&mut self, data: u8) {
        todo!()
    }

    fn bne(&mut self, data: u8) {
        todo!()
    }

    fn bpl(&mut self, data: u8) {
        todo!()
    }

    fn brk(&mut self) {
        self.queue_read(Self::read_pc, |cpu, _| cpu.p.set(Status::INTERRUPT_DISABLE, true));
        self.queue_write(Self::push_stack, |cpu| cpu.pc.get_high());
        self.queue_write(Self::push_stack, |cpu| cpu.pc.get_low());
        self.queue_write(Self::push_stack, |cpu| cpu.p.bits);
        self.queue_read(Self::read_fixed::<0xfffe>, |cpu, data| cpu.pc.set_low(data));
        self.queue_read(Self::read_fixed::<0xffff>, |cpu, data| {
            println!("BRK");
            cpu.pc.set_high(data);
            cpu.p.set(Status::BREAK, true);
        });
    }

    fn bvc(&mut self, data: u8) {
        todo!()
    }

    fn bvs(&mut self, data: u8) {
        todo!()
    }

    fn clc(&mut self, data: u8) {
        todo!()
    }

    fn cld(&mut self, _: u8) {
        self.p.set(Status::DECIMAL, false);
    }

    fn cli(&mut self, data: u8) {
        todo!()
    }

    fn clv(&mut self, data: u8) {
        todo!()
    }

    fn cmp(&mut self, data: u8) {
        todo!()
    }

    fn cpx(&mut self, data: u8) {
        todo!()
    }

    fn cpy(&mut self, data: u8) {
        todo!()
    }

    fn dec(&mut self, data: u8) {
        todo!()
    }

    fn dex(&mut self, data: u8) {
        todo!()
    }

    fn dey(&mut self, data: u8) {
        todo!()
    }

    fn eor(&mut self, data: u8) {
        todo!()
    }

    fn inc(&mut self, data: u8) {
        todo!()
    }

    fn inx(&mut self, data: u8) {
        self.x = self.x.wrapping_add(1);
        self.set_zero_flag(self.x);
        self.set_negative_flag(self.x);
    }

    fn iny(&mut self, data: u8) {
        todo!()
    }

    fn jmp(&mut self) {
        self.queue_read(Self::read_pc_increment, Self::set_address_low);
        self.queue_read(Self::read_pc, |cpu, data| {
            cpu.pc = u16::from_high_low(data, cpu.address.get_low());
            println!("JMP ${:X}", cpu.pc);
        });
    }

    fn jsr(&mut self) {
        self.queue_read(Self::read_pc_increment, |cpu, data| cpu.address.set_low(data));
        self.queue_read(Self::read_stack, |cpu, data| ());
        self.queue_write(Self::push_stack, |cpu| cpu.pc.get_high());
        self.queue_write(Self::push_stack, |cpu| cpu.pc.get_low());
        self.queue_read(Self::read_pc, |cpu, data| {
            cpu.address.set_high(data);
            cpu.pc = cpu.address;
            println!("JSR ${:X}", cpu.address);
        });
    }

    fn lda(&mut self, data: u8) {
        self.a = data;
        self.set_zero_flag(self.a);
        self.set_negative_flag(self.a);
    }

    fn ldx(&mut self, data: u8) {
        self.x = data;
        self.set_zero_flag(self.x);
        self.set_negative_flag(self.x);
    }

    fn ldy(&mut self, data: u8) {
        todo!()
    }

    fn lsr(&mut self, data: u8) {
        todo!()
    }

    fn nop(&mut self, data: u8) {
        todo!()
    }

    fn ora(&mut self, data: u8) {
        todo!()
    }

    fn pha(&mut self, data: u8) {
        todo!()
    }

    fn php(&mut self, data: u8) {
        todo!()
    }

    fn pla(&mut self, data: u8) {
        todo!()
    }

    fn plp(&mut self, data: u8) {
        todo!()
    }

    fn rol(&mut self, data: u8) {
        todo!()
    }

    fn ror(&mut self, data: u8) {
        todo!()
    }

    fn rti(&mut self, data: u8) {
        todo!()
    }

    fn rts(&mut self, data: u8) {
        todo!()
    }

    fn sbc(&mut self, data: u8) {
        todo!()
    }

    fn sec(&mut self, data: u8) {
        todo!()
    }

    fn sed(&mut self, data: u8) {
        todo!()
    }

    fn sei(&mut self, _: u8) {
        self.p.set(Status::INTERRUPT_DISABLE, true);
    }

    fn sta(&mut self) -> u8 {
        self.a
    }

    fn stx(&mut self) -> u8 {
        self.x
    }

    fn sty(&mut self) -> u8 {
        todo!()
    }

    fn tax(&mut self, data: u8) {
        todo!()
    }

    fn tay(&mut self, data: u8) {
        todo!()
    }

    fn tsx(&mut self, data: u8) {
        todo!()
    }

    fn txa(&mut self, _: u8) {
        self.a = self.x;
    }

    fn txs(&mut self, _: u8) {
        self.s = self.x;
    }

    fn tya(&mut self, data: u8) {
        todo!()
    }
}