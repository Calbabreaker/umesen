use crate::{
    cartridge::{self, Catridge},
    cpu::{Cpu, Flags},
};

const STOP: u8 = 0xe2;

fn execute(cpu: &mut Cpu, rom: &[u8]) {
    let start_pc = cpu.pc;
    for (i, x) in rom.iter().enumerate() {
        cpu.bus.write_byte(i as u16 + start_pc, *x);
    }
    cpu.bus.cpu_cycles = 0;
    while rom
        .get((cpu.pc.wrapping_sub(start_pc)) as usize)
        .is_some_and(|v| *v != STOP)
    {
        cpu.execute_next().unwrap();
    }
}

const A: u8 = 0xf0;
const X: u8 = 0xff;
const Y: u8 = 0xfe;

fn test(rom: &[u8], assert_fn: impl Fn(Cpu)) {
    let mut cpu = Cpu {
        a: A,
        x: X,
        y: Y,
        ..Default::default()
    };
    cpu.bus.ram[0x132] = 69;
    cpu.bus.ram[0x12] = 69;
    cpu.bus.cartridge = Some(Catridge::new_only_ram(32 * 1024));
    execute(&mut cpu, rom);
    assert_fn(cpu);
}

#[test]
fn addressing_modes() {
    // Immediate mode lda
    test(&[0xa9, 123], |cpu| {
        assert_eq!(cpu.a, 123);
        assert_eq!(cpu.bus.cpu_cycles, 2);
        assert_eq!(cpu.pc, 2);
    });

    // Zero page mode lda
    test(&[0xa5, 0x12], |cpu| {
        assert_eq!(cpu.a, 69);
        assert_eq!(cpu.bus.cpu_cycles, 3);
        assert_eq!(cpu.pc, 2);
    });

    // Zero page x mode lda (first ldx)
    test(&[0xb5, 0x13], |cpu| {
        assert_eq!(cpu.a, 69);
        assert_eq!(cpu.bus.cpu_cycles, 4);
        assert_eq!(cpu.pc, 2);
    });

    // Zero page y mode ldx (first ldy)
    test(&[0xb6, 0x14], |cpu| {
        assert_eq!(cpu.x, 69);
        assert_eq!(cpu.bus.cpu_cycles, 4);
        assert_eq!(cpu.pc, 2);
    });

    // Absolute mode lda
    test(&[0xad, 0x32, 0x01], |cpu| {
        assert_eq!(cpu.a, 69);
        assert_eq!(cpu.bus.cpu_cycles, 4);
        assert_eq!(cpu.pc, 3);
    });

    // Absolute mode x lda
    test(&[0xbd, 0x33, 0x00], |cpu| {
        assert_eq!(cpu.a, 69);
        assert_eq!(cpu.bus.cpu_cycles, 5);
        assert_eq!(cpu.pc, 3);
    });

    // Absolute mode x lda (no page cross)
    test(&[0xbd], |cpu| assert_eq!(cpu.bus.cpu_cycles, 4));

    // Absolute mode x sta (always extra clock)
    test(&[0x9d, 0x12, 0x00], |mut cpu| {
        assert_eq!(cpu.bus.cpu_cycles, 5);
        assert_eq!(cpu.bus.read_byte(0x111), A);
    });

    // Absolute mode y lda
    test(&[0xb9, 0x34, 0x00], |cpu| {
        assert_eq!(cpu.a, 69);
        assert_eq!(cpu.bus.cpu_cycles, 5);
        assert_eq!(cpu.pc, 3);
    });

    // Indirect x mode lda
    test(&[0xa1, 0x04, STOP, 0x32, 0x01], |cpu| {
        assert_eq!(cpu.a, 69);
        assert_eq!(cpu.bus.cpu_cycles, 6);
        assert_eq!(cpu.pc, 2);
    });

    // Indirect y mode lda
    test(&[0xb1, 0x03, STOP, 0x34, 0x00], |cpu| {
        assert_eq!(cpu.a, 69);
        assert_eq!(cpu.bus.cpu_cycles, 6);
        assert_eq!(cpu.pc, 2);
    });
}

#[test]
fn zero_neg_flags() {
    // lda immediate
    test(&[0xa9, 22], |cpu| assert_eq!(cpu.flags, Flags::empty()));
    test(&[0xa9, 0], |cpu| assert_eq!(cpu.flags, Flags::ZERO));
    test(&[0xa9, 128], |cpu| assert_eq!(cpu.flags, Flags::NEGATIVE));
}

#[test]
fn pha() {
    test(&[0x48], |mut cpu| {
        assert_eq!(cpu.bus.cpu_cycles, 3);
        assert_eq!(cpu.bus.read_byte(0x100), A)
    });
}

#[test]
fn php() {
    test(&[0x38, 0x08], |mut cpu| {
        assert_eq!(cpu.bus.cpu_cycles, 2 + 3);
        assert_eq!(cpu.bus.read_byte(0x100), Flags::CARRY.bits())
    });
}

#[test]
fn pla() {
    test(&[0x68], |cpu| {
        assert_eq!(cpu.bus.cpu_cycles, 4);
        assert_eq!(cpu.a, 0);
    });
}

#[test]
fn plp() {
    test(&[0x38, 0x28], |cpu| {
        assert_eq!(cpu.bus.cpu_cycles, 2 + 4);
        assert_eq!(cpu.flags, Flags::empty())
    });
}

#[test]
fn asl() {
    test(&[0x0a], |mut cpu| {
        assert_eq!(cpu.bus.cpu_cycles, 2);
        assert_eq!(cpu.a, A << 1);
        assert_eq!(cpu.flags, Flags::CARRY | Flags::NEGATIVE);
        execute(&mut cpu, &[0x16, 0x13]);
        assert_eq!(cpu.bus.cpu_cycles, 6);
        assert_eq!(cpu.bus.read_byte(0x12), 69 << 1);
    });
}

#[test]
fn lsr() {
    test(&[0x46, 0x12], |mut cpu| {
        assert_eq!(cpu.bus.cpu_cycles, 5);
        assert_eq!(cpu.bus.read_byte(0x12), 69 >> 1);
    });
}

#[test]
fn rol() {
    test(&[0x2a, 0x2a], |cpu| {
        assert_eq!(cpu.flags, Flags::CARRY | Flags::NEGATIVE);
        assert_eq!(cpu.a, A << 2 | 1);
    });
}

#[test]
fn ror() {
    test(&[0x6a, 0x6a], |cpu| assert_eq!(cpu.a, A >> 2));
}

#[test]
fn inc() {
    test(&[0xe6, 0x12], |mut cpu| {
        assert_eq!(cpu.bus.cpu_cycles, 5);
        assert_eq!(cpu.bus.read_byte(0x12), 70);
    });
}

#[test]
fn dec() {
    test(&[0xd6, 0x13], |mut cpu| {
        assert_eq!(cpu.bus.cpu_cycles, 6);
        assert_eq!(cpu.bus.read_byte(0x12), 68);
    });
}

#[test]
fn inx() {
    test(&[0xe8], |cpu| assert_eq!(cpu.x, X.wrapping_add(1)));
}

#[test]
fn iny() {
    test(&[0xc8], |cpu| assert_eq!(cpu.y, Y.wrapping_add(1)));
}

#[test]
fn dex() {
    test(&[0xca], |cpu| assert_eq!(cpu.x, X.wrapping_sub(1)));
}

#[test]
fn dey() {
    test(&[0x88], |cpu| assert_eq!(cpu.y, Y.wrapping_sub(1)));
}

#[test]
fn adc() {
    test(&[0x69, 69], |mut cpu| {
        assert_eq!(cpu.a, 53);
        assert_eq!(cpu.flags, Flags::CARRY);

        execute(&mut cpu, &[0x69, 80]);
        assert_eq!(cpu.a, 134);
        assert_eq!(cpu.flags, Flags::OVERFLOW | Flags::NEGATIVE);

        cpu.flags.set(Flags::DECIMAL, true);
        execute(&mut cpu, &[0x69, 0x69]);
        assert_eq!(cpu.a, 0x55);
        assert_eq!(cpu.flags, Flags::CARRY | Flags::DECIMAL);
    });
}

#[test]
fn sbc() {
    test(&[0xe9, 69], |cpu| {
        assert_eq!(cpu.a, 171);
        assert_eq!(cpu.flags, Flags::CARRY | Flags::NEGATIVE);
    });
}

#[test]
fn lda() {
    test(&[0xa9, 123], |cpu| assert_eq!(cpu.a, 123));
}

#[test]
fn ldx() {
    test(&[0xa2, 69], |cpu| assert_eq!(cpu.x, 69));
}

#[test]
fn ldy() {
    test(&[0xa0, 69], |cpu| assert_eq!(cpu.y, 69));
}

#[test]
fn sta() {
    test(&[0x85, 2], |mut cpu| assert_eq!(cpu.bus.read_byte(2), A));
}

#[test]
fn stx() {
    test(&[0x86, 2], |mut cpu| assert_eq!(cpu.bus.read_byte(2), X));
}

#[test]
fn sty() {
    test(&[0x84, 2], |mut cpu| assert_eq!(cpu.bus.read_byte(2), Y));
}

#[test]
fn tax() {
    test(&[0xaa], |cpu| {
        assert_eq!(cpu.x, A);
        assert_eq!(cpu.bus.cpu_cycles, 2)
    });
}

#[test]
fn tay() {
    test(&[0xa8], |cpu| assert_eq!(cpu.y, A));
}

#[test]
fn tsx() {
    test(&[0xba], |cpu| assert_eq!(cpu.x, 0));
}

#[test]
fn txa() {
    test(&[0x8a], |cpu| assert_eq!(cpu.a, X));
}

#[test]
fn txs() {
    test(&[0x9a], |cpu| assert_eq!(cpu.sp, X));
}

#[test]
fn tya() {
    test(&[0x98], |cpu| assert_eq!(cpu.a, Y));
}

#[test]
fn clc() {
    test(&[0x38, 0x18], |cpu| assert_eq!(cpu.flags, Flags::empty()));
}

#[test]
fn cld() {
    test(&[0xf8, 0xd8], |cpu| assert_eq!(cpu.flags, Flags::empty()));
}

#[test]
fn cli() {
    test(&[0x78, 0x58], |cpu| assert_eq!(cpu.flags, Flags::empty()));
}

#[test]
fn clv() {
    test(&[], |mut cpu| {
        cpu.flags = Flags::OVERFLOW;
        execute(&mut cpu, &[0xb8]);
        assert_eq!(cpu.flags, Flags::empty());
        assert_eq!(cpu.bus.cpu_cycles, 2);
    });
}

#[test]
fn sec() {
    test(&[0x38], |cpu| assert_eq!(cpu.flags, Flags::CARRY));
}

#[test]
fn sed() {
    test(&[0xf8], |cpu| assert_eq!(cpu.flags, Flags::DECIMAL));
}

#[test]
fn sei() {
    test(&[0x78], |cpu| assert_eq!(cpu.flags, Flags::INTERRUPT));
}

#[test]
fn and() {
    test(&[0x29, 0x69], |cpu| assert_eq!(cpu.a, A & 0x69));
}

#[test]
fn eor() {
    test(&[0x49, 0x69], |cpu| assert_eq!(cpu.a, A ^ 0x69));
}

#[test]
fn ora() {
    test(&[0x09, 0x69], |cpu| assert_eq!(cpu.a, A | 0x69));
}

#[test]
fn bit() {
    test(&[0x24, 0x03, STOP, 0b1100_0000], |cpu| {
        assert_eq!(cpu.flags, Flags::OVERFLOW | Flags::NEGATIVE);
    });
    test(&[0x2c, 0x04, 0, STOP, 0b0100_0000], |cpu| {
        assert_eq!(cpu.flags, Flags::OVERFLOW);
    });
    test(&[0x24, 0x03, STOP, 0b0000_0000], |cpu| {
        assert_eq!(cpu.flags, Flags::ZERO)
    });
}

#[test]
fn cmp() {
    test(&[0xc9, 0xff], |cpu| assert_eq!(cpu.flags, Flags::NEGATIVE));
}

#[test]
fn cpx() {
    test(&[0xe0, 0xff], |cpu| {
        assert_eq!(cpu.flags, Flags::CARRY | Flags::ZERO)
    });
}

#[test]
fn cpy() {
    test(&[0xc0, 0x02], |cpu| {
        assert_eq!(cpu.flags, Flags::CARRY | Flags::NEGATIVE)
    });
}

#[test]
fn jmp() {
    test(&[0x4c, 0x23, 0x11], |mut cpu| {
        assert_eq!(cpu.bus.cpu_cycles, 3);
        assert_eq!(cpu.pc, 0x1123);
        cpu.bus.write_word(0x00ff, 0x1234);
        execute(&mut cpu, &[0x6c, 0xff]);
        assert_eq!(cpu.bus.cpu_cycles, 5);
        assert_eq!(cpu.pc, 0x4c34);
    });
}

#[test]
fn jsr() {
    test(&[0x20, 0x69, 0x12], |mut cpu| {
        assert_eq!(cpu.bus.cpu_cycles, 6);
        assert_eq!(cpu.stack_pop_word(), 2);
        assert_eq!(cpu.pc, 0x1269);
    });
}

#[test]
fn rts() {
    test(&[0x20, 0x04, 0x00, STOP, 0x60], |cpu| {
        assert_eq!(cpu.bus.cpu_cycles, 12);
        assert_eq!(cpu.pc, 3);
    });
}

#[test]
fn brk() {
    test(&[], |mut cpu| {
        cpu.bus.write_word(0xfffe, 0x02);
        execute(&mut cpu, &[0x00]);
        assert_eq!(cpu.bus.cpu_cycles, 7);
        assert_eq!(cpu.pc, 0x02);
        assert_eq!(cpu.flags, Flags::INTERRUPT | Flags::BREAK);
        assert_eq!(Flags::from_bits(cpu.stack_pop()).unwrap(), Flags::empty(),);
        assert_eq!(cpu.stack_pop_word(), 1);
    });
}

#[test]
fn rti() {
    test(&[], |mut cpu| {
        cpu.bus.write_word(0xfffe, 0x02);
        cpu.flags.set(Flags::NEGATIVE, true);
        execute(&mut cpu, &[0x00, STOP, 0x40]);
        assert_eq!(cpu.bus.cpu_cycles, 7 + 6);
        assert_eq!(cpu.pc, 1);
        assert_eq!(cpu.flags, Flags::NEGATIVE);
    });
}

#[test]
fn bcs() {
    test(&[0xb0, 1], |mut cpu| {
        assert_eq!(cpu.pc, 2);
        assert_eq!(cpu.bus.cpu_cycles, 2);
        cpu.flags.set(Flags::CARRY, true);
        execute(&mut cpu, &[0xb0, 1]);
        assert_eq!(cpu.bus.cpu_cycles, 3);
        assert_eq!(cpu.pc, 5);
        execute(&mut cpu, &[0xb0, (-8i8) as u8]);
        assert_eq!(cpu.bus.cpu_cycles, 5);
        assert_eq!(cpu.pc, 0xffff);
    });
}

#[test]
fn bcc() {
    test(&[0x90, 1], |cpu| assert_eq!(cpu.pc, 3));
}

#[test]
fn beq() {
    test(&[0xf0, 1], |cpu| assert_eq!(cpu.pc, 2));
}

#[test]
fn bmi() {
    test(&[0x30, 1], |cpu| assert_eq!(cpu.pc, 2));
}

#[test]
fn bne() {
    test(&[0xd0, 1], |cpu| assert_eq!(cpu.pc, 3));
}

#[test]
fn bpl() {
    test(&[0x10, 1], |cpu| assert_eq!(cpu.pc, 3));
}

#[test]
fn bvc() {
    test(&[0x50, 1], |cpu| assert_eq!(cpu.pc, 3));
}

#[test]
fn bvs() {
    test(&[0x70, 1], |cpu| assert_eq!(cpu.pc, 2));
}

#[test]
fn nop() {
    test(&[0xea], |cpu| assert_eq!(cpu.bus.cpu_cycles, 2));
}

#[test]
fn reset() {
    test(&[], |mut cpu| {
        cpu.reset();
        assert_eq!(cpu.bus.cpu_cycles, 7);
        assert_eq!(cpu.pc, 0);
    });
}
