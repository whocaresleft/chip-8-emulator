use std::time;

pub mod cpu;
pub mod memory;
pub mod display;
pub mod keypad;

pub struct Chip8 {
    pub cpu: cpu::CPU,
    pub memory: memory::Memory,
    pub display: display::Display,
    pub keypad: keypad::Keypad,

    pub opcode: u16,
    resume_from: u16,

    pub waiting_for_key: bool,
    last_poll: std::time::Instant,

    rom: Vec<u8>,
    pub new_draw: bool
}

impl Chip8 {

    pub fn new() -> Self { 
        // Load sprites in memory
        let mut mem = memory::Memory::new();
        let ref sprites = display::DEFAULT_SPRITES;

        let mut base_addr = 0x0000;
        for i in 0..sprites.len() as u16 {

            for j in 0..sprites[i as usize].len() as u16 {
                mem.write_u8(base_addr + i + j, sprites[i as usize][j as usize]);
            }
            base_addr += 4;
        }
        
        let mut cpu= cpu::CPU::new();
        cpu.set_sp(0x4e);
        cpu.set_pc(0x0200);

        Chip8 {
            cpu: cpu,
            memory: mem,
            display: display::Display::new(),
            keypad: keypad::Keypad::new(),

            opcode: 0,
            resume_from: 0,

            waiting_for_key: false,

            last_poll: time::Instant::now(),
            rom: vec![],
            new_draw: false
        }
    }

    pub fn reset(&mut self) {
        self.display.reset();
        self.keypad = keypad::Keypad::new();
        self.opcode = 0;
        self.resume_from = 0;
        self.waiting_for_key = false;
        self.last_poll = time::Instant::now();

        let mut mem = memory::Memory::new();
        let ref sprites = display::DEFAULT_SPRITES;

        let mut base_addr = 0x0000;
        for i in 0..sprites.len() as u16 {

            for j in 0..sprites[i as usize].len() as u16 {
                mem.write_u8(base_addr + i + j, sprites[i as usize][j as usize]);
            }
            base_addr += 4;
        }
        
        let mut cpu= cpu::CPU::new();
        cpu.set_sp(0x4e);
        cpu.set_pc(0x0200);

        self.cpu = cpu;
        self.memory = mem;
        self.load();
    }

    pub fn insert_rom(&mut self, program: Vec<u8>) {
        self.rom = program;
    }

    pub fn load(&mut self) {
        for i in 0..self.rom.len() {
            self.memory.write_u8(0x0200 + i as u16, self.rom[i]);
        }
    }
    
    pub fn resume(&mut self) {
        self.waiting_for_key = false;
        self.cpu.set_pc(self.resume_from);
    }

    pub fn fetch(&mut self) {
        if self.waiting_for_key { return }

        self.opcode = self.memory.read_u16(self.cpu.read_pc());
        println!("Ho fetchato 0x{:x} all'indirizzo 0x{:x}", self.opcode, self.cpu.program_counter);
    }

    pub fn decode_execute(&mut self) {
        if self.waiting_for_key { return }

        self.cpu.increment_pc();
        self.resume_from = self.cpu.read_pc();

        match (self.opcode & 0xF000) >> 12 {

            0x0 => {
                match self.opcode & 0x0FFF {
                    0x0E0 => self.cls(),
                    0x0EE => self.ret(),
                    _ => self.sys_addr(),
                }
            }

            0x1 => self.jp_addr(),

            0x2 => self.call_addr(),

            0x3 => self.se_vx_byte(),

            0x4 => self.sne_vx_byte(),

            0x5 => if self.opcode & 0x000F == 0 { self.se_vx_vy() },

            0x6 => self.ld_vx_byte(),

            0x7 => self.add_vx_byte(),

            0x8 => {
                match self.opcode & 0x000F {
                    0x0 => self.ld_vx_vy(),
                    0x1 => self.or_vx_vy(),
                    0x2 => self.and_vx_vy(),
                    0x3 => self.xor_vx_vy(),
                    0x4 => self.add_vx_vy(),
                    0x5 => self.sub_vx_vy(),
                    0x6 => self.shr_vx(),
                    0x7 => self.subn_vx_vy(),
                    0xE => self.shl_vx(),
                    _ => panic!("Impossible opcode {:2x}", self.opcode)
                }
            }

            0x9 => if self.opcode & 0x000F == 0 { self.sne_vx_vy() },

            0xA => self.ld_i_addr(),

            0xB => self.jp_v0_addr(),

            0xC => self.rnd_vx_byte(),

            0xD => self.drw_vx_vy_nibble(),

            0xE => {
                match self.opcode & 0x00FF {
                    0x9E => self.skp_vx(),
                    0xA1 => self.sknp_vx(),
                    _ => panic!("Impossible opcode {:2x}", self.opcode)
                }
            }

            0xF => {
                match self.opcode & 0x00FF {
                    0x07 => self.ld_vx_dt(),
                    0x0A => self.ld_vx_k(),
                    0x15 => self.ld_dt_vx(),
                    0x18 => self.ld_st_vx(),
                    0x1E => self.add_i_vx(),
                    0x29 => self.ld_f_vx(),
                    0x33 => self.ld_b_vx(),
                    0x55 => self.ld_i_vx(),
                    0x65 => self.ld_vx_i(),
                    _ => panic!("Impossible opcode {:2x}", self.opcode)
                }
            }

            _ => { panic!("Impossible opcode {:2x}", self.opcode) }
        }
    }


    pub fn stack_pop(&mut self) -> Option<u16> {
        match self.cpu.get_sp() {
            0x4e => None,
            0x50 | 0x52 | 0x54 | 0x56 | 0x58 | 0x5A | 0x5C | 0x5E | 
            0x60 | 0x62 | 0x64 | 0x66 | 0x68 | 0x6A | 0x6C | 0x6E => {
                let result = self.memory.read_u16(self.cpu.get_sp() as u16);
                println!("Ho letto {:x} dallo stacke", result);
                self.cpu.set_sp(self.cpu.get_sp() - 2);
                Some(result)
            },
            _ => panic!("Incorrect stack pointer 0x{:x}", self.cpu.get_sp())
        }
    }

    pub fn stack_push(&mut self, value: u16) {
        match self.cpu.get_sp() {
            0x4e | 0x50 | 0x52 | 0x54 | 0x56 | 0x58 | 0x5A | 0x5C |  
            0x5E | 0x60 | 0x62 | 0x64 | 0x66 | 0x68 | 0x6A | 0x6C => {
                self.cpu.set_sp(self.cpu.get_sp() + 2);
                println!("Sto per scrivere {:x} nello stacke", value);
                self.memory.write_u16(self.cpu.get_sp() as u16, value);
            },
            _ => panic!("Incorrect stack pointer 0x{:x}", self.cpu.get_sp())
        }
    }

    fn sys_addr(&mut self) {
        unimplemented!("Modern interpreters ignore this apparently")
    }

    fn cls(&mut self) {
        self.display.reset()
    }

    fn ret(&mut self) {
        let new_pc = self.stack_pop().expect("Something went wrong");
        println!("STACK POP {:x}", new_pc);
        self.cpu.set_pc(new_pc);
    }

    fn jp_addr(&mut self) {
        let addr = self.opcode & 0x0FFF;
        self.cpu.set_pc(addr);
    }

    fn call_addr(&mut self) {
        let addr = self.opcode & 0x0FFF;
        let old_pc = self.cpu.read_pc();
        println!("STACK PUSH {:x}", old_pc);
        self.stack_push(old_pc);

        self.cpu.set_pc(addr);
    }

    fn se_vx_byte(&mut self) {
        let byte: u8 = (self.opcode & 0x00FF) as u8;
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;

        if self.cpu.v_registers[x] == byte {
            self.cpu.increment_pc();
        }
    }

    fn sne_vx_byte(&mut self) {
        let byte: u8 = (self.opcode & 0x00FF) as u8;
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;

        if self.cpu.v_registers[x] != byte {
            self.cpu.increment_pc();
        }
    }

    fn se_vx_vy(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((self.opcode & 0x00F0) >> 4) as usize;

        if self.cpu.v_registers[x] == self.cpu.v_registers[y] {
            self.cpu.increment_pc();
        }
    }

    fn ld_vx_byte(&mut self) {
        let byte: u8 = (self.opcode & 0x00FF) as u8;
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;

        self.cpu.v_registers[x] = byte;
    }

    fn add_vx_byte(&mut self) {
        let byte: u8 = (self.opcode & 0x00FF) as u8;
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;

        self.cpu.v_registers[x] = self.cpu.v_registers[x].wrapping_add(byte);
    }

    fn ld_vx_vy(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((self.opcode & 0x00F0) >> 4) as usize;

        self.cpu.v_registers[x] = self.cpu.v_registers[y]
    }

    fn or_vx_vy(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((self.opcode & 0x00F0) >> 4) as usize;

        self.cpu.v_registers[x] |= self.cpu.v_registers[y]
    }

    fn and_vx_vy(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((self.opcode & 0x00F0) >> 4) as usize;

        self.cpu.v_registers[x] &= self.cpu.v_registers[y]
    }

    fn xor_vx_vy(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((self.opcode & 0x00F0) >> 4) as usize;

        self.cpu.v_registers[x] ^= self.cpu.v_registers[y]
    }

    fn add_vx_vy(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((self.opcode & 0x00F0) >> 4) as usize;

        let vx = self.cpu.v_registers[x] as u16;
        let vy = self.cpu.v_registers[y] as u16;

        self.cpu.v_registers[0xF] = if vx + vy > 0x00FF { 
            1
        } else { 
            0
        };

        self.cpu.v_registers[x] = ((vx + vy) & 0x00FF) as u8;
    }

    fn sub_vx_vy(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((self.opcode & 0x00F0) >> 4) as usize;

        let vx = self.cpu.v_registers[x];
        let vy = self.cpu.v_registers[y];

        self.cpu.v_registers[0xF] = if vx > vy { 
            1
        } else { 
            0
        };

        self.cpu.v_registers[x] = vx.wrapping_sub(vy);
    }

    fn shr_vx(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let vx = self.cpu.v_registers[x];

        self.cpu.v_registers[0xF] = if vx & 0x01 != 0 {
            1
        } else {
            0
        };
        self.cpu.v_registers[x] = vx >> 1;
    }

    fn subn_vx_vy(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((self.opcode & 0x00F0) >> 4) as usize;

        let vx = self.cpu.v_registers[x];
        let vy = self.cpu.v_registers[y];

        self.cpu.v_registers[0xF] = if vy > vx { 
            1
        } else { 
            0
        };

        self.cpu.v_registers[x] = vy.wrapping_sub(vx);
    }

    fn shl_vx(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let vx = self.cpu.v_registers[x];

        self.cpu.v_registers[0xF] = if vx & 0x80 != 0 {
            1
        } else {
            0
        };
        self.cpu.v_registers[x] = vx << 1;
    }

    fn sne_vx_vy(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((self.opcode & 0x00F0) >> 4) as usize;

        if self.cpu.v_registers[x] != self.cpu.v_registers[y] {
            self.cpu.increment_pc();
        }
    }

    fn ld_i_addr(&mut self) {
        let addr = self.opcode & 0x0FFF;
        self.cpu.i_register = addr;
    }

    fn jp_v0_addr(&mut self) {
        let addr= self.opcode & 0x0FFF;
        self.cpu.set_pc(
            addr.wrapping_add(self.cpu.v_registers[0x0] as u16)
        );
    }

    fn rnd_vx_byte(&mut self) {
        let x: usize = ((self.opcode & 0x0F00) >> 8) as usize;
        let byte = (self.opcode & 0x00FF) as u8;
        let rand = self.random(0, 255);

        self.cpu.v_registers[x] = byte & rand;
    }

    fn drw_vx_vy_nibble(&mut self) {
        let nibble = (self.opcode & 0x000F) as u16;
        let y = ((self.opcode & 0x00F0) >> 4) as usize;
        let x = ((self.opcode & 0x0F00) >> 8) as usize;

        let vx = (self.cpu.v_registers[x] as usize) & 63;
        let vy = (self.cpu.v_registers[y] as usize) & 31;

        let mut sprite: u8;
        let mut changed: bool = false;
        self.cpu.v_registers[0xF] = 0;
        for row in 0..nibble {
            if (vy + row as usize) == 32 { break }
            sprite = self.memory.read_u8(self.cpu.i_register + row);
            if self.display.draw_sprite(vy + row as usize, vx, sprite) {
                changed = true;
            }
        }
        self.cpu.v_registers[0xF] = if changed { 1 } else { 0 };
        self.new_draw = true;
    }

    fn skp_vx(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as usize;
        let vx = self.cpu.v_registers[x];

        if self.keypad.is_down(vx) { self.cpu.increment_pc() }
    }

    fn sknp_vx(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as usize;
        let vx = self.cpu.v_registers[x];

        if self.keypad.is_up(vx) { self.cpu.increment_pc() }
    }

    fn ld_vx_dt(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as usize;
        self.cpu.v_registers[x] = self.cpu.delay;
    }

    fn ld_vx_k(&mut self) {
        self.waiting_for_key = true;
    }

    pub fn resume_ld_vx_k(&mut self, key: u8) {
        let x = ((self.opcode & 0x0F00) >> 8) as usize;
        self.cpu.v_registers[x] = key;
        self.resume();
    }

    fn ld_dt_vx(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as usize;
        self.cpu.delay = self.cpu.v_registers[x];
    }

    fn ld_st_vx(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as usize;
        self.cpu.sound = self.cpu.v_registers[x];
    }

    fn add_i_vx(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as usize;
        let vx = self.cpu.v_registers[x] as u16;
        self.cpu.i_register = self.cpu.i_register.wrapping_add(vx);
    }

    fn ld_f_vx(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as usize;

        let addr = self.cpu.v_registers[x] as u16 * 5;
        self.cpu.i_register = addr;
    }

    fn ld_b_vx(&mut self) {
        let x = ((self.opcode & 0x0F00) >> 8) as usize;
        let vx = self.cpu.v_registers[x];

        let units = vx % 10;
        let tens = (vx % 100) - units;
        let hundreds = vx - tens - units;
        self.memory.write_u8(self.cpu.i_register, hundreds / 100);
        self.memory.write_u8(self.cpu.i_register + 1, tens / 10);
        self.memory.write_u8(self.cpu.i_register + 2, units);
    }

    fn ld_i_vx(&mut self) {
        let x = (self.opcode & 0x0F00) >> 8;
        for j in 0x0u16..=x {
            let vj = self.cpu.v_registers[j as usize];
            self.memory.write_u8(self.cpu.i_register + j, vj);
        }
    }

    fn ld_vx_i(&mut self) {
        let x = (self.opcode & 0x0F00) >> 8;
        for j in 0x0u16..=x {
            self.cpu.v_registers[j as usize] = self.memory.read_u8(self.cpu.i_register + j);
        }
    }

    fn random(&mut self, min_incl: u8, max_incl: u8) -> u8 {
        let min = std::time::Duration::from_secs(min_incl as u64);
        let max = std::time::Duration::from_secs(max_incl as u64);
        let r= std::time::Instant::now().duration_since(self.last_poll).clamp(min, max);
        self.last_poll = std::time::Instant::now();

        (r.as_secs() & 0x0000_0000_0000_FFFF) as u8
    }

    pub fn update_dt(&mut self) {
        if self.cpu.delay > 0 { self.cpu.delay -= 1 }
    }

    pub fn update_st(&mut self) {
        if self.cpu.sound > 0 { self.cpu.sound -= 1 }
    }

    pub fn get_mnemonic(opcode: u16) -> String {
        match (opcode & 0xF000) >> 12 {

            0x0 => {
                match opcode & 0x0FFF {
                    0x0E0 => "CLS".to_owned(),
                    0x0EE => "RET".to_owned(),
                    _ => format!("SYS {:#06x}", opcode & 0x0FFF),
                }
            }

            0x1 => format!("JP {:#06x}", opcode & 0x0FFF),

            0x2 => format!("CALL {:#06x}", opcode & 0x0FFF),

            0x3 => format!("SE V{:1x}, {:#04x}", (opcode & 0x0F00) >> 8, opcode & 0x00FF),

            0x4 => format!("SNE V{:1x}, {:#04x}", (opcode & 0x0F00) >> 8, opcode & 0x00FF),

            0x5 => if opcode & 0x000F == 0 { format!("SE V{:1x}, V{:1x}", (opcode & 0x0F00) >> 8, (opcode & 0x00F0) >> 4) } else { panic!("Unrecognized {:2x}", opcode)},

            0x6 => format!("LD V{:1x}, {:#04x}", (opcode & 0x0F00) >> 8, opcode & 0x00FF),

            0x7 => format!("ADD V{:1x}, {:#04x}", (opcode & 0x0F00) >> 8, opcode & 0x00FF),

            0x8 => {
                match opcode & 0x000F {
                    0x0 => format!("LD V{:1x}, V{:1x}", (opcode & 0x0F00) >> 8, (opcode & 0x00F0) >> 4),
                    0x1 => format!("OR V{:1x}, V{:1x}", (opcode & 0x0F00) >> 8, (opcode & 0x00F0) >> 4),
                    0x2 => format!("AND V{:1x}, V{:1x}", (opcode & 0x0F00) >> 8, (opcode & 0x00F0) >> 4),
                    0x3 => format!("XOR V{:1x}, V{:1x}", (opcode & 0x0F00) >> 8, (opcode & 0x00F0) >> 4),
                    0x4 => format!("ADD V{:1x}, V{:1x}", (opcode & 0x0F00) >> 8, (opcode & 0x00F0) >> 4),
                    0x5 => format!("SUB V{:1x}, V{:1x}", (opcode & 0x0F00) >> 8, (opcode & 0x00F0) >> 4),
                    0x6 => format!("SHR V{:1x}", (opcode & 0x0F00) >> 8),
                    0x7 => format!("SUBN V{:1x}, V{:1x}", (opcode & 0x0F00) >> 8, (opcode & 0x00F0) >> 4),
                    0xE => format!("SHL V{:1x}", (opcode & 0x0F00) >> 8),
                    _ => panic!("Impossible opcode {:2x}", opcode)
                }
            }

            0x9 => if opcode & 0x000F == 0 { format!("SNE V{:1x}, V{:1x}", (opcode & 0x0F00) >> 8, (opcode & 0x00F0) >> 4) } else { panic!("Unrecognized opcode {:2x}", opcode) },

            0xA => format!("LD I, {:#06x}", opcode & 0x0FFF),

            0xB => format!("JP V0, {:#06x}", opcode & 0x0FFF),

            0xC => format!("RND V{:1x}, {:#04x}", (opcode & 0x0F00) >> 8, opcode & 0x00FF),

            0xD => format!("DRW V{:1x}, V{:1x}, {:#03x}", (opcode & 0x0F00) >> 8, (opcode & 0x00F0) >> 4, opcode & 0x000F),

            0xE => {
                match opcode & 0x00FF {
                    0x9E => format!("SKP V{:1x}", (opcode & 0x0F00) >> 8),
                    0xA1 => format!("SKNP V{:1x}", (opcode & 0x0F00) >> 8),
                    _ => panic!("Impossible opcode {:2x}",  opcode)
                }
            }

            0xF => {
                match opcode & 0x00FF {
                    0x07 => format!("LD V{:1x}, DT", (opcode & 0x0F00) >> 8),
                    0x0A => format!("LD V{:1x}, K", (opcode & 0x0F00) >> 8),
                    0x15 => format!("LD DT, V{:1x}", (opcode & 0x0F00) >> 8),
                    0x18 => format!("LD ST, V{:1x}", (opcode & 0x0F00) >> 8),
                    0x1E => format!("ADD I, V{:1x}", (opcode & 0x0F00) >> 8),
                    0x29 => format!("LD F, V{:1x}", (opcode & 0x0F00) >> 8),
                    0x33 => format!("LD B, V{:1x}", (opcode & 0x0F00) >> 8),
                    0x55 => format!("LD I, V{:1x}", (opcode & 0x0F00) >> 8),
                    0x65 => format!("LD V{:1x}, I", (opcode & 0x0F00) >> 8),
                    _ => panic!("Impossible opcode {:2x}", opcode)
                }
            }

            _ => { panic!("Impossible opcode {:2x}", opcode) }
        }
    }
}