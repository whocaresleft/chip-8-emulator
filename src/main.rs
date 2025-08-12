mod debug_ui;
mod chip8;
use debug_ui::DebugUI;
use std::{sync::{Arc, Mutex}, thread, time};
fn main() -> eframe::Result<()> {

    let game: Vec<u8> = std::fs::read("./roms/BC_test.ch8").unwrap();
    let mut chip8 = chip8::Chip8::new();
    chip8.load(game);

    let emulator_logic: Arc<Mutex<chip8::Chip8>> = Arc::new( Mutex::new( chip8 ) );
    let emulator_graphic = Arc::clone(&emulator_logic);

    thread::spawn(move || {
        // Chip8 logic execution
        let mut start = time::Instant::now();
        let mut end = time::Instant::now();
        let mut accumulator = 0.0f64;
        let threshold = 1.0 / 540.0;
        let mut cycles: u8 = 0;

        while emulator_logic.lock().unwrap().running {
            if emulator_logic.lock().unwrap().paused {
                std::thread::sleep(time::Duration::from_millis(2000));
            } else {
                let delta = end - start;
                start = time::Instant::now();
                accumulator += delta.as_secs_f64();

                while accumulator >= threshold {

                    let mut chip = emulator_logic.lock().unwrap();
                    chip.fetch();
                    chip.decode_execute();
                    cycles += 1;
                    if cycles == 9 {
                        chip.sound();
                        chip.timer();
                        cycles = 0;
                    }
                    accumulator -= threshold;
                }
                end = time::Instant::now();
            }
        }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Graphic thread",
        options,
        Box::new(|_cc| Ok(Box::new(DebugUI::new(emulator_graphic)))),
    )

}