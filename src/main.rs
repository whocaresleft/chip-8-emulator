mod debug_ui;
mod chip8;
use debug_ui::DebugUI;
fn main() -> eframe::Result<()> {

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 600.0]),
        ..Default::default()
    };
    let game: Vec<u8> = std::fs::read("./roms/test_opcode.ch8").unwrap();
    let mut chip8 = chip8::Chip8::new();
    chip8.cpu.delay = 0xff;
    chip8.cpu.sound = 0xff;
    chip8.load(
        vec![0x6E, 0xFF, 0xF1, 0x0A, 0x6E, 0xFE, 0x6E, 0xFD, 0x6E, 0xFC, 0x12, 0x02]
    );
    eframe::run_native(
        "Debugger",
        options,
        Box::new(|_cc| Ok(Box::new(DebugUI::new(chip8)))),
    )
}