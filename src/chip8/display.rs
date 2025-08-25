pub const DEFAULT_SPRITES: [[u8; 5]; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0], // 0
    [0x20, 0x60, 0x20, 0x20, 0x70], // 1
    [0xF0, 0x10, 0xF0, 0x80, 0xF0], // 2
    [0xF0, 0x10, 0xF0, 0x10, 0xF0], // 3
    [0x90, 0x90, 0xF0, 0x10, 0x10], // 4
    [0xF0, 0x80, 0xF0, 0x10, 0xF0], // 5
    [0xF0, 0x80, 0xF0, 0x90, 0xF0], // 6
    [0xF0, 0x10, 0x20, 0x40, 0x40], // 7
    [0xF0, 0x90, 0xF0, 0x90, 0xF0], // 8
    [0xF0, 0x90, 0xF0, 0x10, 0xF0], // 9
    [0xF0, 0x90, 0xF0, 0x90, 0x90], // A
    [0xE0, 0x90, 0xE0, 0x90, 0xE0], // B
    [0xF0, 0x80, 0x80, 0x80, 0xF0], // C
    [0xE0, 0x90, 0x90, 0x90, 0xE0], // D
    [0xF0, 0x80, 0xF0, 0x80, 0xF0], // E
    [0xF0, 0x80, 0xF0, 0x80, 0x80], // F
];

pub struct Display {
    pub screen: [[u8; 64]; 32]
}

impl Display {
    pub fn new() -> Self {
        Display { screen: [[0; 64]; 32] }
    }
    pub fn reset(&mut self) {
        self.screen = [[0; 64]; 32];
    }

    pub fn draw_sprite(&mut self, row: usize, starting_column: usize, byte: u8) -> bool {
        let mut at_least_1_flipped: bool = false;

        let effective_row = row & 31;
        let effective_starting_column = starting_column & 63;

        for i in 0..8 {
            let col = effective_starting_column + i;
            if col == 64 { break }
            
            let current_bit = (byte >> (7 - i)) & 1;
            let old_bit = self.screen[effective_row][col];
            self.screen[effective_row][col] ^= current_bit;
            if old_bit == 1 && self.screen[effective_row][col] == 0 { at_least_1_flipped = true }
        }

        at_least_1_flipped
    }
}

/*pub struct Display {

    screen: [
        [
            u8; 8
        ]; 32
    ]
}

pub enum PixelOperation { On, Off, Flip }

impl Display {
    pub fn new() -> Self {
        Display { screen: [[0; 8]; 32] }
    }

    pub fn reset(&mut self) {
        self.screen = [[0; 8]; 32]
    }

    pub fn update_pixel(&mut self, row: usize, column: usize, op: PixelOperation) {
        match (row, column) {
            (0..=31, 0..=63) => {
                let quo = column >> 3;
                let rem = column & 7;
                let bit_mask = 1u8 << (7 - rem);
                match op {
                    PixelOperation::On => {
                        self.screen[row][quo] |= bit_mask;
                    }
                    PixelOperation::Off => {
                        self.screen[row][quo] &= !bit_mask;
                    }
                    PixelOperation::Flip => {
                        self.screen[row][quo] ^= bit_mask;
                    }
                }
            }
            _ => { panic!("Pixel out of bounds") }
        }
    }
    pub fn get_pixel(&self, row: usize, column: usize) -> u8 {
        let quo = column >> 3;
        let rem = column & 7;
        (self.get_batch(row, quo) & (1 << rem)) >> rem
    }

    pub fn get_batch(&self, row: usize, column: usize) -> u8 {
        match (row, column) {
            (0..=31, 0..=7) => {
                self.screen[row][column]
            }
            _ => { panic!("Pixel out of bounds") }
        }
    }

    pub fn draw_byte(&mut self, row: usize, starting_column: usize, byte: u8) -> bool {
        let mut at_least_1_flipped: bool = false;

        let effective_row = row & 31;
        let effective_starting_column = starting_column & 63;

        for i in 0..8 {
            let bit_index = 7 - i;
            let quo = effective_starting_column.wrapping_add(i) >> 3;
            if quo == 8 { break }
            let rem = effective_starting_column.wrapping_add(i) & 7;
            let current_bit = (byte >> bit_index) & 1;
            print!("Before: screen[{}][{}] = {} ->", effective_row, quo, self.screen[effective_row][quo]);
            self.screen[effective_row][quo] ^= current_bit << rem;
            println!("After: screen[{}][{}] = {}", effective_row, quo, self.screen[effective_row][quo]);
            if self.screen[effective_row][quo] & (1 << rem) == 0 { at_least_1_flipped = true }
        }

        at_least_1_flipped
    }

    pub fn screen_state(&self) -> Vec<u8> {
        self.screen.iter().flatten().copied().collect()
    }
}



#[cfg(test)]
mod display_test {

    #[test]
    #[should_panic]
    fn pixel_out_of_bounds() {
        let mut dis = super::Display::new();
        dis.update_pixel(100, 1, super::PixelOperation::Flip);
    }

    #[test]
    fn get_pixel_test() {
        let dis = super::Display::new();
        assert_eq!(dis.get_pixel(0, 0), 0u8);
    }

    #[test]
    fn pixel_batch_test() {
        let dis = super::Display::new();
        assert_eq!(dis.get_batch(0, 0), 0u8);
    }

    #[test]
    fn pixel_on_test() {
        let mut dis = super::Display::new();
        dis.update_pixel(0, 2, super::PixelOperation::On);

        assert_eq!(dis.get_pixel(0, 2), 1u8);
        assert_eq!(dis.get_batch(0, 0), 4u8);

        dis.update_pixel(31, 8, super::PixelOperation::On);
        dis.update_pixel(31, 9, super::PixelOperation::On);
        dis.update_pixel(31, 10, super::PixelOperation::On);
        dis.update_pixel(31, 11, super::PixelOperation::On);
        dis.update_pixel(31, 12, super::PixelOperation::On);
        dis.update_pixel(31, 13, super::PixelOperation::On);
        dis.update_pixel(31, 14, super::PixelOperation::On);
        //dis.update_pixel(31, 15, super::PixelOperation::On);
        assert_eq!(dis.get_batch(31, 1), 0x7F);
    }

    #[test]
    fn pixel_off_test() {
        let mut dis = super::Display::new();
        dis.update_pixel(0, 2, super::PixelOperation::On);

        assert_eq!(dis.get_pixel(0, 2), 1u8);
        assert_eq!(dis.get_batch(0, 0), 4u8);

        dis.update_pixel(31, 8, super::PixelOperation::On);
        dis.update_pixel(31, 9, super::PixelOperation::On);
        dis.update_pixel(31, 10, super::PixelOperation::On);
        dis.update_pixel(31, 11, super::PixelOperation::On);
        dis.update_pixel(31, 12, super::PixelOperation::On);
        dis.update_pixel(31, 13, super::PixelOperation::On);
        dis.update_pixel(31, 14, super::PixelOperation::On);
        dis.update_pixel(31, 15, super::PixelOperation::On);
        assert_eq!(dis.get_batch(31, 1), 0xFF);
        
        dis.update_pixel(31, 15, super::PixelOperation::Off);
        assert_eq!(dis.get_batch(31, 1), 0x7F);
    }

    #[test]
    fn pixel_flip_test() {
        let mut dis = super::Display::new();
        dis.update_pixel(0, 2, super::PixelOperation::On);

        assert_eq!(dis.get_pixel(0, 2), 1u8);
        assert_eq!(dis.get_batch(0, 0), 4u8);

        dis.update_pixel(31, 8, super::PixelOperation::On);
        dis.update_pixel(31, 9, super::PixelOperation::On);
        dis.update_pixel(31, 10, super::PixelOperation::On);
        dis.update_pixel(31, 11, super::PixelOperation::On);
        dis.update_pixel(31, 12, super::PixelOperation::On);
        dis.update_pixel(31, 13, super::PixelOperation::On);
        dis.update_pixel(31, 14, super::PixelOperation::On);
        dis.update_pixel(31, 15, super::PixelOperation::On);
        assert_eq!(dis.get_batch(31, 1), 0xFF);
        
        dis.update_pixel(31, 15, super::PixelOperation::Flip);
        assert_eq!(dis.get_batch(31, 1), 0x7F);
        
        dis.update_pixel(31, 15, super::PixelOperation::Flip);
        assert_eq!(dis.get_batch(31, 1), 0xFF);
    }
}*/