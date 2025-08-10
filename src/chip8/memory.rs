pub struct Memory {
    address_space:[u8; 0x1000]
}

impl Memory {

    pub fn new() -> Self {
        Memory {
            address_space: [0; 0x1000]
        }
    }

    pub fn write_u8(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x0FFF => { self.address_space[addr as usize] = data; }

            0x1000..=0xFFFF => { panic!("Address out of range") }
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x0FFF => { self.address_space[addr as usize] }

            0x1000..=0xFFFF => { panic!("Address out of range") }
        }
    }

    pub fn write_u16(&mut self, addr: u16, data: u16) {
        let lo = (data & 0x00FF) as u8;
        let hi = ((data & 0xFF00) >> 8) as u8;
        self.write_u8(addr, hi);
        self.write_u8(addr + 1, lo);
    }

    pub fn read_u16(&self, addr: u16) -> u16 {
        let hi = self.read_u8(addr);
        let lo = self.read_u8(addr + 1);

        ((hi as u16) << 8) | lo as u16
    }
}

#[cfg(test)]
mod memory_test {

    #[test]
    #[should_panic]
    fn access_out_of_bounds() {
        let memory = super::Memory::new();
        memory.read_u8(0x1000);
    }

    #[test]
    #[should_panic]
    fn access_in_interpreter_space() {
        let memory = super::Memory::new();
        memory.read_u8(0x0000);
    }

    #[test]
    fn write_read_u8_test() {
        let mut memory = super::Memory::new();

        memory.write_u8(0x0213, 0xFE);
        assert_eq!(memory.read_u8(0x0213), 0xFE);
    }

    #[test]
    fn write_read_u16_test() {
        let mut memory = super::Memory::new();

        memory.write_u16(0x0213, 0x12FE);
        assert_eq!(memory.read_u16(0x0213), 0x12FE);

        assert_eq!(memory.read_u8(0x0213), 0x12);
        assert_eq!(memory.read_u8(0x0214), 0xFE);

        memory.write_u8(0x0300, 0x12);
        memory.write_u8(0x0301, 0x13);
        assert_eq!(memory.read_u16(0x0300), 0x1213);
    }
}