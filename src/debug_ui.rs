use super::chip8::Chip8;
use egui::{ColorImage, TextureHandle};

pub struct DebugUI {
    emulator: Chip8,
    texture: Option<TextureHandle>,
    show_memory_window: bool,
    start_addr: u16,
    end_addr: u16,
    running: bool,
    last_timer_update: std::time::Instant,
    executed: bool
}

impl DebugUI {
    pub fn new(emul: Chip8) -> Self {
        DebugUI {
            emulator: emul,
            texture: None,
            show_memory_window: false,
            start_addr: 0x0200,
            end_addr: 0x020F,
            running: false,
            last_timer_update: std::time::Instant::now(),
            executed: true
        }
    }

    fn update_texture(&mut self, ctx: &egui::Context) {
        let mut image = ColorImage::new([64, 32], egui::Color32::BLACK);

        for y in 0..32 {
            for x in 0..64 {
                let color = if self.emulator.display.get_pixel(y, x) == 1 {
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
                    self.emulator.keypad.set_key(chip8_key, true);
                } else {
                    self.emulator.keypad.set_key(chip8_key, false);
                }
            }
        });
    }
    fn handle_key_for_resume(&mut self, ctx: &egui::Context) {
        ctx.input(|input|{
            for (chip8_key, egui_key) in DebugUI::chip8_keymap() {
                if input.key_down(egui_key) {
                    self.emulator.keypad.set_key(chip8_key, true);
                    self.emulator.resume_ld_vx_k(chip8_key);
                    return;
                }
            }
        });
    }

    fn chip8_keymap() -> [(u8, egui::Key); 16] {
        use egui::Key::*;
        [
            (0x00, X),
            (0x01, Num1),
            (0x02, Num2),
            (0x03, Num3),
            (0x04, Q),
            (0x05, W),
            (0x06, E),
            (0x07, A),
            (0x08, S),
            (0x09, D),
            (0x0A, Z),
            (0x0B, C),
            (0x0C, Num4),
            (0x0D, R),
            (0x0E, F),
            (0x0F, V),
        ]
    }
}

impl eframe::App for DebugUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        let now = std::time::Instant::now();
        let target_frame_duration = std::time::Duration::from_secs_f64(1.0 / 60.0);
        if self.running && now - self.last_timer_update >= target_frame_duration {
            self.handle_input(ctx);

            if !self.emulator.paused {
                self.emulator.fetch();
                self.emulator.decode_execute();

                if now.duration_since(self.last_timer_update) >= std::time::Duration::from_millis(16) {
                    self.emulator.timer();
                    self.emulator.sound();
                }
                self.last_timer_update = now;
            }
        }
        if self.emulator.paused { self.handle_key_for_resume(ctx); }

        self.update_texture(ctx);

        egui::SidePanel::left("debug").show(ctx, |ui| {
            ui.heading("Debug info");
            ui.set_min_width(200.0);
            ui.add_space(15.0);
            ui.label(format!("Opcode: {:#06x}", self.emulator.opcode));
            ui.add_space(15.0);
            ui.horizontal(|ui| {
                if self.running {
                    if ui.button("Stop").clicked() {
                        self.running = false;
                    }
                } else {
                    if ui.button("Fetch").clicked() {
                        if self.executed {
                            self.handle_input(ctx);
                            self.emulator.fetch();
                            self.executed = false;
                        }
                    }
                    if !self.executed {
                        if ui.button("Execute").clicked() {
                            self.emulator.decode_execute();
                            self.executed = true;
                        }
                    }
                    if ui.button("Step").clicked() {
                        self.handle_input(ctx);
                        self.emulator.fetch();
                        self.emulator.decode_execute();
                    }
                    if ui.button("Run").clicked() {
                        self.running = true;
                        self.last_timer_update = std::time::Instant::now();
                    }
                }
                if ui.button("Exit").clicked() {
                    std::process::exit(0);
                }
            });
            ui.add_space(15.0);

            ui.label("CPU registers");
            let reg = &self.emulator.cpu.v_registers;

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
                    ui.label(format!("PC: {:#06x}", self.emulator.cpu.program_counter));
                    ui.label(format!("SP: {:#04x}", self.emulator.cpu.stack_pointer));
                    ui.label(format!("I:  {:#06x}", self.emulator.cpu.i_register));
                });
                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    ui.label(format!("DT: {}", self.emulator.cpu.delay));
                    ui.label(format!("ST: {}", self.emulator.cpu.sound));
                });
            });
            ui.add_space(15.0);
            ui.label("Stack content");
            let base: u8 = 0x0050;
            ui.group(|ui| {
                for addr in (base..self.emulator.cpu.stack_pointer).step_by(2) {
                    let value =  self.emulator.memory.read_u16(addr as u16);
                    ui.label(format!("{:#04x}: {:#06x}", addr, value));
                    ui.separator();
                }
                if base <= self.emulator.cpu.stack_pointer {
                    let value =  self.emulator.memory.read_u16(self.emulator.cpu.stack_pointer as u16);
                    ui.label(format!("{:#04x}: {:#06x}", self.emulator.cpu.stack_pointer, value));
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
                                            ui.label(format!("{:#04x}", self.emulator.memory.read_u8(addr)));
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
            ui.group(|ui| {
                egui::Grid::new("keypad-status").striped(true).show(ui, |ui| {
                    for i in 0..4 {
                        ui.colored_label(if self.emulator.keypad.is_down(i as u8 * 4) { egui::Color32::LIGHT_RED } else { egui::Color32::WHITE }, format!("{:1x}", self.emulator.keypad.keys[i * 4] & 0x0F));
                        ui.colored_label(if self.emulator.keypad.is_down(i as u8 * 4 + 1) { egui::Color32::LIGHT_RED } else { egui::Color32::WHITE }, format!("{:1x}", self.emulator.keypad.keys[i * 4 + 1] & 0x0F));
                        ui.colored_label(if self.emulator.keypad.is_down(i as u8 * 4 + 2) { egui::Color32::LIGHT_RED } else { egui::Color32::WHITE }, format!("{:1x}", self.emulator.keypad.keys[i * 4 + 2] & 0x0F));
                        ui.colored_label(if self.emulator.keypad.is_down(i as u8 * 4 + 3) { egui::Color32::LIGHT_RED } else { egui::Color32::WHITE }, format!("{:1x}", self.emulator.keypad.keys[i * 4 + 3] & 0x0F));
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