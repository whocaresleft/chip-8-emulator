pub struct CPU {
    pub v_registers: [u8; 16],
    pub i_register: u16, // Only 12bits actually used

    pub delay: u8,
    pub sound: u8,
    
    pub program_counter: u16,
    pub stack_pointer: u8
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            v_registers: [0; 16],
            i_register: 0,

            delay: 0,
            sound: 0,

            program_counter: 0,
            stack_pointer: 0
        }
    }

    pub fn read_pc(&self) -> u16 {
        self.program_counter & 0x0FFF
    }

    pub fn increment_pc(&mut self) {
        self.set_pc(self.program_counter + 2);
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.program_counter = pc & 0x0FFF;
    }

    pub fn get_sp(&self) -> u8 { self.stack_pointer }
    pub fn set_sp(&mut self, sp: u8) { self.stack_pointer = sp }
}