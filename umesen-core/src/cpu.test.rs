use crate::cpu::{Cpu, Flags};

fn execute(cpu: &mut Cpu, rom: &[u8]) {
    let start_pc = cpu.pc as usize;
    for (i, x) in rom.iter().enumerate() {
        cpu.bus.ram[i + start_pc] = *x;
    }
    while cpu.execute_next().is_ok() {}
    cpu.bus.cpu_cycles -= 1;
    cpu.pc -= 1;
}

const A: u8 = 0xff;
const X: u8 = 0xfe;
const Y: u8 = 0xfd;

fn test(rom: &[u8], assert_fn: impl Fn(Cpu)) {
    let mut cpu = Cpu {
        a: A,
        x: X,
        y: Y,
        ..Default::default()
    };
    cpu.bus.ram[0x132] = 69;
    cpu.bus.ram[0x12] = 69;
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
    test(&[0xb5, 0x14], |cpu| {
        assert_eq!(cpu.a, 69);
        assert_eq!(cpu.bus.cpu_cycles, 4);
        assert_eq!(cpu.pc, 2);
    });

    // Zero page y mode ldx (first ldy)
    test(&[0xb6, 0x15], |cpu| {
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
    test(&[0xbd, 0x34, 0x00], |cpu| {
        assert_eq!(cpu.a, 69);
        assert_eq!(cpu.bus.cpu_cycles, 5);
        assert_eq!(cpu.pc, 3);
    });

    // Absolute mode x lda (no page cross)
    test(&[0xbd], |cpu| assert_eq!(cpu.bus.cpu_cycles, 4));

    // Absolute mode x sta (always extra clock)
    test(&[0x9d, 0x13, 0x00], |mut cpu| {
        assert_eq!(cpu.bus.cpu_cycles, 5);
        assert_eq!(cpu.bus.read_byte(0x111), 0xff);
    });

    // Absolute mode y lda
    test(&[0xb9, 0x35, 0x00], |cpu| {
        assert_eq!(cpu.a, 69);
        assert_eq!(cpu.bus.cpu_cycles, 5);
        assert_eq!(cpu.pc, 3);
    });

    // Indirect x mode lda
    test(&[0xa1, 0x05, 0, 0x32, 0x01], |cpu| {
        assert_eq!(cpu.a, 69);
        assert_eq!(cpu.bus.cpu_cycles, 6);
        assert_eq!(cpu.pc, 2);
    });

    // Indirect y mode lda
    test(&[0xb1, 0x03, 0, 0x35, 0x00], |cpu| {
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
fn adc() {
    test(&[0x69, 69], |mut cpu| {
        assert_eq!(cpu.a, 68);
        assert_eq!(cpu.flags, Flags::CARRY);

        execute(&mut cpu, &[0x69, 69]);
        assert_eq!(cpu.a, 138);
        assert_eq!(cpu.flags, Flags::OVERFLOW | Flags::NEGATIVE);

        cpu.flags.set(Flags::DECIMAL, true);
        execute(&mut cpu, &[0x69, 0x69]);
        assert_eq!(cpu.a, 0x59);
        assert_eq!(cpu.flags, Flags::CARRY | Flags::DECIMAL);
    });
}

#[test]
fn sbc() {
    test(&[0xe9, 69], |cpu| {
        assert_eq!(cpu.a, 186);
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
    test(&[0x85, 2], |mut cpu| assert_eq!(cpu.bus.read_byte(2), 0xff));
}

#[test]
fn stx() {
    test(&[0x86, 2], |mut cpu| assert_eq!(cpu.bus.read_byte(2), 0xfe));
}

#[test]
fn sty() {
    test(&[0x84, 2], |mut cpu| assert_eq!(cpu.bus.read_byte(2), 0xfd));
}

#[test]
fn tax() {
    test(&[0xaa], |cpu| {
        assert_eq!(cpu.x, 0xff);
        assert_eq!(cpu.bus.cpu_cycles, 2)
    });
}

#[test]
fn tay() {
    test(&[0xa8], |cpu| assert_eq!(cpu.y, 0xff));
}

#[test]
fn tsx() {
    test(&[0xba], |cpu| assert_eq!(cpu.x, 0));
}

#[test]
fn txa() {
    test(&[0x8a], |cpu| assert_eq!(cpu.a, 0xfe));
}

#[test]
fn txs() {
    test(&[0x9a], |cpu| assert_eq!(cpu.sp, 0xfe));
}

#[test]
fn tya() {
    test(&[0x98], |cpu| assert_eq!(cpu.a, 0xfd));
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
    test(&[0x29, 0x69], |cpu| assert_eq!(cpu.a, 0x69));
}

#[test]
fn eor() {
    test(&[0x49, 0x69], |cpu| assert_eq!(cpu.a, 0xff ^ 0x69));
}

#[test]
fn ora() {
    test(&[0x09, 0x69], |cpu| assert_eq!(cpu.a, 0xff | 0x69));
}

#[test]
fn bit() {
    test(&[0x24, 0x03, 0, 0b1100_0000], |cpu| {
        assert_eq!(cpu.flags, Flags::OVERFLOW | Flags::NEGATIVE);
    });
    test(&[0x2c, 0x03, 0, 0b0100_0000], |cpu| {
        assert_eq!(cpu.flags, Flags::OVERFLOW);
    });
    test(&[0x24, 0x03, 0, 0b0000_0000], |cpu| {
        assert_eq!(cpu.flags, Flags::ZERO)
    });
}

#[test]
fn cmp() {
    test(&[0xc9, 0xff], |cpu| {
        assert_eq!(cpu.flags, Flags::ZERO | Flags::CARRY)
    });
}

#[test]
fn cpx() {
    test(&[0xb0, 0xff], |cpu| assert_eq!(cpu.flags, Flags::empty()));
}

#[test]
fn cpy() {
    test(&[0xc0, 0x2], |cpu| {
        assert_eq!(cpu.flags, Flags::CARRY | Flags::NEGATIVE)
    });
}

#[test]
fn nop() {
    test(&[0xea], |cpu| assert_eq!(cpu.bus.cpu_cycles, 2));
}
