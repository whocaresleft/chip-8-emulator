use std::sync::mpsc::{Sender, Receiver};
use super::{Command, Status};

pub struct DebugUI {
    tx: Sender<Command>,
    rx_framebuffer: Receiver<[[u8; 64]; 32]>,
    rx_status: Receiver<Status>,

    framebuffer: [[u8; 64]; 32],
    texture: Option<egui::TextureHandle>,
    debug: bool,
    executed: bool,

    running: bool,
    paused: bool,

    status: Status,
    show_memory_window: bool,

    start_addr: u16,
    end_addr: u16,

    pressed: std::collections::HashSet<egui::Key>,

    color_on: [f32; 3],
    color_off: [f32; 3],
    picked_file: Option<String>,
    frequency: u32
}

impl DebugUI {
    pub fn new(tx: Sender<Command>, rx_framebuffer: Receiver<[[u8; 64]; 32]>, rx_status: Receiver<Status>) -> Self {
        DebugUI {
            tx: tx,
            rx_framebuffer: rx_framebuffer,
            rx_status: rx_status,

            framebuffer: [[0u8; 64]; 32],
            texture: None,
            debug: false,
            executed: false,

            running: true,
            paused: true,

            status: Status::empty(),
            show_memory_window: false,

            start_addr: 0x0200,
            end_addr: 0x020F,

            pressed: std::collections::HashSet::<egui::Key>::new(),

            picked_file: None,
            color_on: [1.0; 3],
            color_off: [0.0; 3],
            frequency: 540

        }
    }

    fn update_texture(&mut self, ctx: &egui::Context) {
        let mut image = egui::ColorImage::new(
            [64, 32],
            vec![egui::Color32::BLACK; 64 * 32]
        );
        for y in 0..32 {
            for x in 0..64 {
                let color = if self.framebuffer[y][x] == 1 {
                    DebugUI::rgb_to_color(self.color_on) 
                } else {
                    DebugUI::rgb_to_color(self.color_off) 
                };
                image.pixels[y * 64 + x] = color;
            }
        }
        if let Some(tex) = &mut self.texture {
            tex.set(image, egui::TextureOptions::NEAREST)
        } else {
            self.texture = Some(ctx.load_texture("disp", image, egui::TextureOptions::NEAREST))
        }
    }

    fn handle_input(&mut self, ctx: &egui::Context) {
        ctx.input(|input| {
            for(chip8_key, egui_key) in KEYMAP {
                let was_pressed = self.pressed.contains(&egui_key);
                let is_down = input.key_down(egui_key);
                if is_down && !was_pressed {
                    self.pressed.insert(egui_key);
                    _ = self.tx.send(Command::KeyDown(chip8_key));
                } else if !is_down && was_pressed {
                    self.pressed.remove(&egui_key);
                    _ = self.tx.send(Command::KeyUp(chip8_key));
                }
            }
        });
    }
    fn rgb_to_color(rgb: [f32; 3]) -> egui::Color32 {
        egui::Color32::from_rgb(
            (rgb[0] * 255.0).round() as u8,
            (rgb[1] * 255.0).round() as u8,
            (rgb[2] * 255.0).round() as u8
        )
    }

}

use egui::Key::*;
const KEYMAP: [(u8, egui::Key); 16] = 
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
    ];
    

impl eframe::App for DebugUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        self.handle_input(ctx);
        match self.rx_framebuffer.try_recv() {
            Ok(fb) => {
                self.framebuffer = fb;
                self.update_texture(ctx);
            },
            Err(_) => { /* no change */ }
        }

        egui::SidePanel::left("debug").show(ctx, |ui| {
            ui.heading("Debug info");
            ui.label(format!("Emulator is{}running", 
                match self.running {
                    true => " ",
                    false => " not "
                }
            ));
            ui.label(format!("Emulator is{}paused", 
                match self.paused {
                    true => " ",
                    false => " not "
                }
            ));
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.debug, "Debug mode");
                if self.debug {
                    if ui.button("Snapshot").clicked() {
                        _ = self.tx.send(Command::Snapshot(self.start_addr, self.end_addr));
                    }
                }
            });
            ui.set_min_width(200.0);
            ui.add_space(15.0);
            if self.debug {
                ui.label(format!("Opcode: {:#06x} - {}", self.status.opcode, self.status.mnemonic));
                ui.add_space(15.0);
            }
            ui.horizontal(|ui| {
                if !self.paused {
                    if ui.button("Stop").clicked() {
                        _ = self.tx.send(Command::Pause);
                        self.paused = true;
                    }
                } else {
                    if ui.button("Fetch").clicked() {
                        if self.executed {
                            self.handle_input(ctx);
                            _ = self.tx.send(Command::Fetch);
                            self.executed = false;
                        }
                    }
                    if !self.executed {
                        if ui.button("Execute").clicked() {
                            _ = self.tx.send(Command::Execute);
                            self.executed = true;
                        }
                    }
                    if ui.button("Step").clicked() {
                            self.handle_input(ctx);
                        _ = self.tx.send(Command::Step);
                    }
                    if ui.button("Run").clicked() {
                        _ = self.tx.send(Command::Resume);
                        self.paused = false;
                    }
                }
                if ui.button("Exit").clicked() {
                    _ = self.tx.send(Command::Exit);
                    self.running = false;
                    std::thread::sleep(
                        std::time::Duration::from_millis(
                            400
                        )
                    );
                    std::process::exit(0);
                }
            });
            ui.add_space(15.0);

            if self.debug {
                match self.rx_status.try_recv() {
                    Ok(status) => self.status = status,
                    Err(_) => {} // no change
                }
                ui.label("CPU registers");
                ui.group(|ui| {
                    
                    let reg = &self.status.v;
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
                        ui.label(format!("PC: {:#06x}", self.status.pc));
                        ui.label(format!("SP: {:#04x}", self.status.sp));
                        ui.label(format!("I:  {:#06x}", self.status.i));
                    });
                    ui.add_space(15.0);
                    ui.horizontal(|ui| {
                        ui.label(format!("DT: {}", self.status.dt));
                        ui.label(format!("ST: {}", self.status.st));
                    });
                });
                ui.add_space(15.0);
                ui.label("Stack content");
                let base: u8 = 0x0050;
                ui.group(|ui| {
                    for addr in (base..self.status.sp).step_by(2) {
                        let value = (self.status.stack[(addr - base) as usize] as u16) << 8 
                            |
                            (self.status.stack[(addr - base + 1) as usize] as u16);
                        ui.label(format!("{:#04x}: {:#06x}", addr, value));
                        ui.separator();
                    }
                    if base <= self.status.sp {
                        let value = (self.status.stack[(self.status.sp - base) as usize] as u16) << 8 
                            |
                            (self.status.stack[(self.status.sp - base + 1) as usize] as u16);
                        ui.label(format!("{:#04x}: {:#06x}", self.status.sp, value));
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
                                                ui.label( 
                                                    match self.status.mem_view.get(addr as usize - 0x0200) {
                                                        Some(value) => format!("{:#04x}", value),
                                                        None => format!("----"),
                                                    }
                                                );
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
                let f = self.frequency;
                if ui.add(egui::Slider::new(&mut self.frequency, 1..=600).text(format!("Frequency: {} Hz", f))).changed() {
                    _ = self.tx.send(Command::ChangeFreq(self.frequency));
                }
            }
            
            ui.add_space(15.0);
            ui.label("Keypad");
            ui.group(|ui| {
                egui::Grid::new("keypad-status").striped(true).show(ui, |ui| {
                    let keypad = &self.status.keypad;
                    for i in 0..4 {
                        ui.colored_label(
                            if keypad[i * 4] & 0xF0 != 0 {
                                egui::Color32::LIGHT_RED
                            } else {
                                egui::Color32::DARK_GRAY
                            },
                            format!("{:1x}", keypad[i * 4] & 0x0F)
                        );

                        ui.colored_label(
                            if keypad[i * 4 + 1] & 0xF0 != 0 {
                                egui::Color32::LIGHT_RED
                            } else {
                                egui::Color32::DARK_GRAY
                            },
                            format!("{:1x}", keypad[i * 4 + 1] & 0x0F)
                        );

                        ui.colored_label(
                            if keypad[i * 4 + 2] & 0xF0 != 0 {
                                egui::Color32::LIGHT_RED
                            } else {
                                egui::Color32::DARK_GRAY
                            },
                            format!("{:1x}", keypad[i * 4 + 2] & 0x0F)
                        );

                        ui.colored_label(
                            if keypad[i * 4 + 3] & 0xF0 != 0 {
                                egui::Color32::LIGHT_RED
                            } else {
                                egui::Color32::DARK_GRAY
                            },
                            format!("{:1x}", keypad[i * 4 + 3] & 0x0F)
                        );
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
        egui::TopBottomPanel::bottom("Tweaks").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("ON color: ");
                ui.color_edit_button_rgb(&mut self.color_on);
                ui.add_space(10.0);
                ui.label("OFF color: ");
                ui.color_edit_button_rgb(&mut self.color_off);
                ui.add_space(10.0);
                if ui.button("Insert ROM: ").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.picked_file = Some(path.display().to_string());
                    }
                }
                if let Some(picked_path) = &self.picked_file {
                    ui.monospace(picked_path);
                    
                    if ui.button("Load ROM").clicked() {
                        if let Ok(rom) = std::fs::read(picked_path) {
                            _ = self.tx.send(super::Command::LoadRom(rom));
                        }
                        self.picked_file = None;
                        self.paused = true;
                        self.update_texture(ctx);
                    }
                }
            });
        });
    }
}