pub const DEFAULT_LAYOUT: [u8; 16] = [
    0x01, 0x02, 0x03, 0x0C,
    0x04, 0x05, 0x06, 0x0D,
    0x07, 0x08, 0x09, 0x0E,
    0x0A, 0x00, 0x0B, 0x0F
];

pub struct Keypad {
    
    pub keys: [u8; 16]
}

impl Keypad {

    pub fn new() -> Self {
        Keypad { 
            keys: DEFAULT_LAYOUT
        }
    }

    pub fn is_down(&self, key: u8) -> bool {
        self.keys[(key & 0xF) as usize] & 0xF0 != 0
    }

    pub fn is_up(&self, key: u8) -> bool {
        self.keys[(key & 0xF) as usize] & 0xF0 == 0
    }

    pub fn set_key(&mut self, key: u8, down: bool) {
        if down {
            self.keys[(key & 0xF) as usize] |= 0x10;
        } else {
            self.keys[(key & 0xF) as usize] &= 0x0F;
        }
    }
}