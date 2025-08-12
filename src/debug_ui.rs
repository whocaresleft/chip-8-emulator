use super::chip8::Chip8;
use egui::{ColorImage, TextureHandle};
use std::sync::{Arc, Mutex};

pub struct DebugUI {
    emulator: Arc<Mutex<Chip8>>,
    texture: Option<TextureHandle>,
    show_memory_window: bool,
    start_addr: u16,
    end_addr: u16,
    last_timer_update: std::time::Instant,
    executed: bool
}

impl DebugUI {
    pub fn new(emul: Arc<Mutex<Chip8>>) -> Self {
        DebugUI {
            emulator: emul,
            texture: None,
            show_memory_window: false,
            start_addr: 0x0200,
            end_addr: 0x020F,
            last_timer_update: std::time::Instant::now(),
            executed: true
        }
    }

    fn update_texture(&mut self, ctx: &egui::Context) {
        let mut image = ColorImage::new([64, 32], egui::Color32::BLACK);
        let display = self.emulator.lock().unwrap().display.screen.clone();

        for y in 0..32 {
            for x in 0..64 {
                let color = if display[y][x] == 1 {
                    egui::Color32::WHITE
                } else {
                    egui::Color32::BLACK
                };
                image.pixels[y * 64 + x] = color;
            }
        }

        if let Some(tex) = &mut self.texture {
            tex.set(image, egui::TextureOptions::NEAREST)
        } else {
            self.texture = Some(ctx.load_texture(
                "chip8-display", 
                image, 
                egui::TextureOptions::NEAREST
            ));
        }
    }
    fn handle_input(&mut self, ctx: &egui::Context) {
        ctx.input(|input|{
            for (chip8_key, egui_key) in DebugUI::chip8_keymap() {
                if input.key_down(egui_key) {
                    self.emulator.lock().unwrap().keypad.set_key(chip8_key, true);
                } else {
                    self.emulator.lock().unwrap().keypad.set_key(chip8_key, false);
                }
            }
        });
    }
    fn handle_key_for_resume(&mut self, ctx: &egui::Context) {
        ctx.input(|input|{
            for (chip8_key, egui_key) in DebugUI::chip8_keymap() {
                if input.key_down(egui_key) {
                    self.emulator.lock().unwrap().keypad.set_key(chip8_key, true);
                    self.emulator.lock().unwrap().resume_ld_vx_k(chip8_key);
                    return;
                }
            }
        });
    }

    fn chip8_keymap() -> [(u8, egui::Key); 16] {
        use egui::Key::*;
        [
            (0x00, Num1),
            (0x01, Num2),
            (0x02, Num3),
            (0x03, Num4),

            (0x04, Q),
            (0x05, W),
            (0x06, E),
            (0x07, R),

            (0x08, A),
            (0x09, S),
            (0x0A, D),
            (0x0B, F),

            (0x0C, Z),
            (0x0D, X),
            (0x0E, C),
            (0x0F, V),
        ]
    }
}

impl eframe::App for DebugUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        self.handle_input(ctx);
        if self.emulator.lock().unwrap().waiting_for_key { self.handle_key_for_resume(ctx); }

        self.update_texture(ctx);

        egui::SidePanel::left("debug").show(ctx, |ui| {
            ui.heading("Debug info");
            ui.label(format!("Emulator is{}running", 
                match self.emulator.lock().unwrap().running {
                    true => " ",
                    false => " not "
                }
            ));
            ui.label(format!("Emulator is{}paused", 
                match self.emulator.lock().unwrap().paused {
                    true => " ",
                    false => " not "
                }
            ));
            ui.set_min_width(200.0);
            ui.add_space(15.0);
            let opcode = self.emulator.lock().unwrap().opcode;
            ui.label(format!("Opcode: {:#06x} - {}", opcode, Chip8::get_mnemonic(opcode)));
            ui.add_space(15.0);
            ui.horizontal(|ui| {
                if !self.emulator.lock().unwrap().paused {
                    if ui.button("Stop").clicked() {
                        self.emulator.lock().unwrap().paused = true;
                    }
                } else {
                    if ui.button("Fetch").clicked() {
                        if self.executed {
                            self.handle_input(ctx);
                            self.emulator.lock().unwrap().fetch();
                            self.executed = false;
                        }
                    }
                    if !self.executed {
                        if ui.button("Execute").clicked() {
                            self.emulator.lock().unwrap().decode_execute();
                            self.executed = true;
                        }
                    }
                    if ui.button("Step").clicked() {
                        self.handle_input(ctx);
                        let mut emul = self.emulator.lock().unwrap();
                        emul.fetch();
                        emul.decode_execute();
                    }
                    if ui.button("Run").clicked() {
                        self.emulator.lock().unwrap().paused = false;
                    }
                }
                if ui.button("Exit").clicked() {
                    self.emulator.lock().unwrap().running = false;
                    std::process::exit(0);
                }
            });
            ui.add_space(15.0);

            ui.label("CPU registers");
            let reg = self.emulator.lock().unwrap().cpu.v_registers.clone();

            let sp = self.emulator.lock().unwrap().cpu.stack_pointer;
            ui.group(|ui| {
                
                egui::Grid::new("cpu-registers").striped(true).show(ui, |ui| {
                    for i in 0..4 {
                        ui.label(format!("V{:1x}: {:#04x}", i * 4, reg[i * 4]));
                        ui.label(format!("V{:1x}: {:#04x}", i * 4 + 1, reg[i * 4 + 1]));
                        ui.label(format!("V{:1x}: {:#04x}", i * 4 + 2, reg[i * 4 + 2]));
                        ui.label(format!("V{:1x}: {:#04x}", i * 4 + 3, reg[i * 4 + 3]));
                        ui.end_row();
                    }
                });
                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    ui.label(format!("PC: {:#06x}", self.emulator.lock().unwrap().cpu.program_counter));
                    ui.label(format!("SP: {:#04x}", sp));
                    ui.label(format!("I:  {:#06x}", self.emulator.lock().unwrap().cpu.i_register));
                });
                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    ui.label(format!("DT: {}", self.emulator.lock().unwrap().cpu.delay));
                    ui.label(format!("ST: {}", self.emulator.lock().unwrap().cpu.sound));
                });
            });
            ui.add_space(15.0);
            ui.label("Stack content");
            let base: u8 = 0x0050;
            ui.group(|ui| {
                for addr in (base..sp).step_by(2) {
                    let value =  self.emulator.lock().unwrap().memory.read_u16(addr as u16);
                    ui.label(format!("{:#04x}: {:#06x}", addr, value));
                    ui.separator();
                }
                if base <= sp {
                    let value =  self.emulator.lock().unwrap().memory.read_u16(sp as u16);
                    ui.label(format!("{:#04x}: {:#06x}", sp, value));
                }
            });
            ui.add_space(25.0);
            if ui.button("Memory view").clicked() {
                self.show_memory_window = !self.show_memory_window;
            }

            if self.show_memory_window {
                egui::Window::new("Memory").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Address range:");
                        let (s, e) = (self.start_addr, self.end_addr);
                        ui.add(egui::Slider::new(&mut self.start_addr, 0x0200..=0x0FFF).text(format!("({:#06x})", s)));
                        ui.add(egui::Slider::new(&mut self.end_addr, self.start_addr..=0x0FFF).text(format!("({:#06x})", e)));
                    });

                    if self.end_addr >= self.start_addr {
                        
                        let bytes_per_row = 16;
                        let rows = (self.end_addr - self.start_addr + bytes_per_row - 1) / bytes_per_row;

                        egui::Grid::new("memory_grid")
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Addr (+)");
                                for i in 0..bytes_per_row {
                                    ui.label(format!("{:#03x}", i));
                                }
                                ui.end_row();
                                for row in 0..rows {
                                    let base_addr = self.start_addr + row * bytes_per_row;
                                    ui.label(format!("{:#04x}:", base_addr));

                                    for col in 0..bytes_per_row {
                                        let addr = base_addr + col;
                                        if addr <= self.end_addr {
                                            ui.label(format!("{:#04x}", self.emulator.lock().unwrap().memory.read_u8(addr)));
                                        } else {
                                            ui.label("    ");
                                        }
                                    }   
                                    ui.end_row();
                                }
                            });
                        }
                    }   
                );
            }
            ui.add_space(15.0);
            ui.label("Keypad");
            let keypad = &self.emulator.lock().unwrap().keypad;
            ui.group(|ui| {
                egui::Grid::new("keypad-status").striped(true).show(ui, |ui| {
                    for i in 0..4 {
                        ui.colored_label(if keypad.is_down(i as u8 * 4) { egui::Color32::LIGHT_RED } else { egui::Color32::DARK_GRAY }, format!("{:1x}", keypad.keys[i * 4] & 0x0F));
                        ui.colored_label(if keypad.is_down(i as u8 * 4 + 1) { egui::Color32::LIGHT_RED } else { egui::Color32::DARK_GRAY }, format!("{:1x}", keypad.keys[i * 4 + 1] & 0x0F));
                        ui.colored_label(if keypad.is_down(i as u8 * 4 + 2) { egui::Color32::LIGHT_RED } else { egui::Color32::DARK_GRAY }, format!("{:1x}", keypad.keys[i * 4 + 2] & 0x0F));
                        ui.colored_label(if keypad.is_down(i as u8 * 4 + 3) { egui::Color32::LIGHT_RED } else { egui::Color32::DARK_GRAY }, format!("{:1x}", keypad.keys[i * 4 + 3] & 0x0F));
                        ui.end_row();
                    }
                });
            });
        });


        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tex) = &self.texture {
                ui.add(
                    egui::Image::new(tex).max_size(egui::Vec2::new(640.0, 320.0)).fit_to_exact_size(egui::Vec2::new(640.0, 320.0))
                );
            }
        });
    }
}