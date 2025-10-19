#![cfg_attr(feature = "release-ver", windows_subsystem = "windows")]

mod chip8;
use chip8::{Chip8};

#[cfg(feature = "debug-ver")]
use std::{thread, time, sync::mpsc};
#[cfg(feature = "debug-ver")]
mod debugui;
#[cfg(feature = "debug-ver")]
use debugui::{DebugUI, Command, Status};
#[cfg(feature = "debug-ver")]
fn run_debug_ver() -> eframe::Result<()> {
    let (tx, rx) = mpsc::channel();
    let (tx_keyboard, rx_keyboard) = mpsc::channel();
    let (tx_framebuffer, rx_framebuffer) = mpsc::channel();
    let (tx_status, rx_status) = mpsc::channel();

    thread::spawn(move || {
        
        // Emulator execution logic
        let mut chip = Chip8::new();

        let mut start = time::Instant::now();
        let mut end = time::Instant::now();
        let mut accumulator = 0.0f64;
        let mut threshold = 1.0/540.0;
        let mut cycles = 0u8;

        let mut running = true;
        let mut paused = true;

        let mut snapshot = false;
        let mut keep_sending = false;
        
        let (mut s, mut e): (u16, u16) = (0x0200, 0x020F);

        while running {

            // Handle messages
            for cmd in rx.try_iter() {
                match cmd {
                    Command::Exit => running = false,
                    Command::Pause => paused = true,
                    Command::Resume => paused = false,

                    Command::Snapshot(start_addr, end_addr) => {
                        (s, e) = (start_addr, end_addr);
                        snapshot = true
                    },

                    Command::Fetch => if paused { chip.fetch() }
                    Command::Execute => if paused {
                        chip.decode_execute();
                        cycles += 1;
                        if cycles == 9 {
                            chip.update_st();
                            chip.update_dt();
                            cycles = 0;
                        }
                    }
                    Command::Step => if paused { 
                        chip.fetch();
                        chip.decode_execute();
                        cycles += 1;
                        if cycles == 9 {
                            chip.update_st();
                            chip.update_dt();
                            cycles = 0;
                        }
                    }

                    Command::KeyDown(key) => {
                        chip.keypad.set_key(key, true);
                        // If waiting for key, resume
                        if chip.waiting_for_key {
                            chip.resume_ld_vx_k(key);
                        }
                        println!("Key {} is now DOWN", key);
                    }

                    Command::KeyUp(key) => {
                        chip.keypad.set_key(key, false);
                        println!("Key {} is now UP", key);
                    }

                    Command::LoadRom(rom) => {
                        paused = true;
                        chip.insert_rom(rom);
                        chip.reset();
                    }

                    Command::ChangeFreq(freq) => {
                        threshold = 1.0/(freq as f64);
                    }

                    Command::Continuous(keep) => {
                        keep_sending = keep;
                    }
                }
            }

            if paused {
                thread::sleep(time::Duration::from_millis(3000));
            }

            else {
                let delta = end - start;
                start = time::Instant::now();
                accumulator += delta.as_secs_f64();

                while accumulator >= threshold {
                    chip.fetch();
                    chip.decode_execute();
                    cycles += 1;
                    if cycles == 9 {
                        chip.update_st();
                        chip.update_dt();
                        cycles = 0;
                    }
                    accumulator -= threshold;
                }
                end = time::Instant::now();
            }

            _ = tx_keyboard.send(chip.keypad.keys);

            if chip.new_draw {
                _ = tx_framebuffer.send(
                    chip.display.screen
                );
                chip.new_draw = false;
            }

            if snapshot || keep_sending {
                _ = tx_status.send(
                    Status::from_emul(&chip, s, e)
                );
                snapshot = false;
            }
        }
    });

    let image = image::load_from_memory(include_bytes!("..\\icons\\logo-debug.png"))
        .unwrap().into_rgba8();
    let (w, h) = image.dimensions();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]).with_icon(
            std::sync::Arc::new(egui::IconData {
                rgba: image.into_raw(),
                width: w,
                height: h
            })
        ),
        ..Default::default()
    };
    
    eframe::run_native(
        "Chip8 Emulatxr Debugger", 
        options, 
        Box::new(|_cc| Ok(
            Box::new(
                DebugUI::new(tx, rx_framebuffer, rx_status, rx_keyboard)
            )
        ))
    )
}

#[cfg(feature = "release-ver")]
fn set_window_icon(window: &mut sdl2::video::Window) -> Result<(), String> {
    use sdl2::image::ImageRWops;

    let _img = sdl2::image::init(InitFlag::PNG)?;
    let surface = RWops::from_bytes(include_bytes!("..\\icons\\logo-release.png"))?.load_png()?;
    window.set_icon(surface);
    Ok(())
}

#[cfg(feature = "release-ver")]
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{PixelFormatEnum},
    surface::Surface, rwops::RWops, image::{LoadSurface, InitFlag},
};
#[cfg(feature = "release-ver")]
fn run_release_ver() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err("No ROM selected".to_owned());
    }
    let game_path = &args[1];
    

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let mut window = video_subsystem
        .window("Chip8 Emulatxr", 640, 320)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    set_window_icon(&mut window)?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;
    canvas.set_scale(10.0, 10.0).map_err(|e| e.to_string())?;

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(
            PixelFormatEnum::RGB24,
            64,
            32
        ).map_err(|e| e.to_string())?;

    // Read rom from args
    let program = std::fs::read(game_path).map_err(|_| "Could not read game rom".to_owned())?;

    let mut chip = Chip8::new();
    chip.insert_rom(program);

    chip.load();
    chip.run_with_callbacks(
        move |chip| {
            if let Some(_) = handle_user_input(chip, &mut event_pump) { chip.exit = true; }
        },
        move |chip| {
            if chip.new_draw {

                // Draw screen
                let screen_state = map_chip_display(&chip.display);
                texture.update(None, &screen_state, 64 * 3).ok();
                canvas.copy(&texture, None, None).ok();
                canvas.present();

                chip.new_draw = false;
            }
        },
        540.0
    );

    Ok(())
}
#[cfg(feature = "release-ver")]
fn map_chip_display(display: &chip8::display::Display) -> Vec<u8> {
    let screen_state: Vec<u8> = display.screen
        .iter()
        .flat_map(|row| {
            row.iter().flat_map(|pixel| {
                if *pixel == 0 { [0, 0, 0] }
                else { [255, 255, 255] }
            })
        })
        .collect();
    screen_state
}
#[cfg(feature = "release-ver")]
fn handle_user_input(chip: &mut Chip8, event_pump: &mut sdl2::EventPump) -> Option<()> {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => return Some(()),

            Event::KeyDown { keycode: Some(Keycode::Num1), .. } => { chip.keypad.set_key(0x01, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x01) } },
            Event::KeyDown { keycode: Some(Keycode::Num2), .. } => { chip.keypad.set_key(0x02, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x02) } },
            Event::KeyDown { keycode: Some(Keycode::Num3), .. } => { chip.keypad.set_key(0x03, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x03) } },
            Event::KeyDown { keycode: Some(Keycode::Num4), .. } => { chip.keypad.set_key(0x0C, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x0C) } },
            Event::KeyDown { keycode: Some(Keycode::Q), .. } => { chip.keypad.set_key(0x04, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x04) } },
            Event::KeyDown { keycode: Some(Keycode::W), .. } => { chip.keypad.set_key(0x05, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x05) } },
            Event::KeyDown { keycode: Some(Keycode::E), .. } => { chip.keypad.set_key(0x06, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x06) } },
            Event::KeyDown { keycode: Some(Keycode::R), .. } => { chip.keypad.set_key(0x0D, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x0D) } },
            Event::KeyDown { keycode: Some(Keycode::A), .. } => { chip.keypad.set_key(0x07, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x07) } },
            Event::KeyDown { keycode: Some(Keycode::S), .. } => { chip.keypad.set_key(0x08, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x08) } },
            Event::KeyDown { keycode: Some(Keycode::D), .. } => { chip.keypad.set_key(0x09, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x09) } },
            Event::KeyDown { keycode: Some(Keycode::F), .. } => { chip.keypad.set_key(0x0E, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x0E) } },
            Event::KeyDown { keycode: Some(Keycode::Z), .. } => { chip.keypad.set_key(0x0A, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x0A) } },
            Event::KeyDown { keycode: Some(Keycode::X), .. } => { chip.keypad.set_key(0x00, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x00) } },
            Event::KeyDown { keycode: Some(Keycode::C), .. } => { chip.keypad.set_key(0x0B, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x0B) } },
            Event::KeyDown { keycode: Some(Keycode::V), .. } => { chip.keypad.set_key(0x0F, true); if chip.waiting_for_key { chip.resume_ld_vx_k(0x0F) } },

            Event::KeyUp { keycode: Some(Keycode::Num1), .. } => { chip.keypad.set_key(0x01, false) },
            Event::KeyUp { keycode: Some(Keycode::Num2), .. } => { chip.keypad.set_key(0x02, false) },
            Event::KeyUp { keycode: Some(Keycode::Num3), .. } => { chip.keypad.set_key(0x03, false) },
            Event::KeyUp { keycode: Some(Keycode::Num4), .. } => { chip.keypad.set_key(0x0C, false) },
            Event::KeyUp { keycode: Some(Keycode::Q), .. } => { chip.keypad.set_key(0x04, false) },
            Event::KeyUp { keycode: Some(Keycode::W), .. } => { chip.keypad.set_key(0x05, false) },
            Event::KeyUp { keycode: Some(Keycode::E), .. } => { chip.keypad.set_key(0x06, false) },
            Event::KeyUp { keycode: Some(Keycode::R), .. } => { chip.keypad.set_key(0x0D, false) },
            Event::KeyUp { keycode: Some(Keycode::A), .. } => { chip.keypad.set_key(0x07, false) },
            Event::KeyUp { keycode: Some(Keycode::S), .. } => { chip.keypad.set_key(0x08, false) },
            Event::KeyUp { keycode: Some(Keycode::D), .. } => { chip.keypad.set_key(0x09, false) },
            Event::KeyUp { keycode: Some(Keycode::F), .. } => { chip.keypad.set_key(0x0E, false) },
            Event::KeyUp { keycode: Some(Keycode::Z), .. } => { chip.keypad.set_key(0x0A, false) },
            Event::KeyUp { keycode: Some(Keycode::X), .. } => { chip.keypad.set_key(0x00, false) },
            Event::KeyUp { keycode: Some(Keycode::C), .. } => { chip.keypad.set_key(0x0B, false) },
            Event::KeyUp { keycode: Some(Keycode::V), .. } => { chip.keypad.set_key(0x0F, false) },

            _ => {}
        }
    }
    None
}

fn main() {
    
    #[cfg(feature = "debug-ver")]
    match run_debug_ver() {
        Ok(_) => {},
        Err(e) => eprintln!("{}", e),
    }

    #[cfg(feature = "release-ver")]
    match run_release_ver() {
        Ok(_) => {},
        Err(s) => eprintln!("{}", s),
    }
}