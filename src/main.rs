use std::{thread, time};
use std::sync::mpsc::{self};
mod debugui;
mod chip8;
use chip8::*;
use debugui::DebugUI;

fn main() -> eframe::Result<()> {
    
    let (tx, rx) = mpsc::channel();
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

        while running {
            let (mut s, mut e): (u16, u16) = (0x0200, 0x020F);

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

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Chipx8 Emulator Debugger", 
        options, 
        Box::new(|_cc| Ok(
            Box::new(
                DebugUI::new(tx, rx_framebuffer, rx_status)
            )
        ))
    )
}

enum Command {
    Pause,
    Resume,
    Exit,

    Snapshot(u16, u16),

    Fetch,
    Execute,
    Step,

    KeyDown(u8),
    KeyUp(u8),

    LoadRom(Vec<u8>),

    ChangeFreq(u32),

    Continuous(bool)
}

struct Status {
    pc: u16,
    sp: u8,
    i: u16,

    dt: u8,
    st: u8,

    v: [u8; 16],
    stack: [u8; 32],

    mem_view: Vec<u8>,

    opcode: u16,
    mnemonic: String,

    keypad: [u8; 16]
}
impl Status {
    pub fn empty() -> Self {
        Status{pc: 0, sp: 0, i: 0, dt: 0, st: 0, v: [0; 16], stack: [0; 32], mem_view: vec![], opcode: 0, mnemonic: String::new(), keypad: chip8::keypad::DEFAULT_LAYOUT}
    }

    pub fn from_emul(chip: &Chip8, start: u16, end: u16) -> Self {
        let mut stack = [0u8; 32];
        stack.copy_from_slice(&chip.memory.address_space[0x50..0x70]);

        

        Status {
            pc: chip.cpu.program_counter,
            sp: chip.cpu.stack_pointer,
            i: chip.cpu.i_register,
            dt: chip.cpu.delay,
            st: chip.cpu.sound,
            v: chip.cpu.v_registers.clone(),
            stack: stack,
            mem_view: chip.memory.address_space[start as usize..=end as usize].to_vec(),
            opcode: chip.opcode,
            mnemonic: Chip8::get_mnemonic(chip.opcode),
            keypad: chip.keypad.keys
        }
    }
}