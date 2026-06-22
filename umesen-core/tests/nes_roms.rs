use umesen_core::Emulator;

// Test rom by kevtris https://www.qmtpro.com/~nes/misc/nestest.txt
#[test]
fn nestest() {
    let correct_logs = include_str!("nestest.log");

    let mut emu = Emulator::default();
    emu.load_nes_rom("tests/nestest.nes").unwrap();
    emu.cpu.pc = 0xc000;

    let mut prev_disassem = "CPU RESET";
    for (i, correct_line) in correct_logs.lines().enumerate() {
        let mut line_split = correct_line.split("//");
        let correct_output = line_split.next().unwrap().trim();
        assert_eq!(
            emu.debug_log(),
            correct_output,
            "Incorrect output on line {i} executing: {prev_disassem}"
        );

        // Current line actually contains the state of the instruction executed last line
        prev_disassem = line_split.next().unwrap().trim();

        emu.cpu.execute_next().unwrap();
    }
}
