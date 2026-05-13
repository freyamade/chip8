const MEMSIZE: usize = 4096;

const FONT_DATA: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];
// Memory offset for where we store the font data
const FONT_OFFSET: usize = 0x050;
// How big each font item is for needing to quickly retrieve it
const FONT_BYTES: usize = 5;

#[derive(Clone)]
pub struct Memory {
    ram: [u8; MEMSIZE],
    stack: Vec<usize>,
}

impl Memory {
    pub fn new() -> Memory {
        let mut mem: Memory = Memory {ram: [0; MEMSIZE], stack: vec![]};
        // add the font data into the memory between bytes 050 and 09F inclusive
        for i in 0..FONT_DATA.len() {
            mem.write(FONT_OFFSET + i, FONT_DATA[i]);
        }
        return mem;
    }

    pub fn read(&self, address: usize) -> u8 {
        if address >= MEMSIZE {
            return 0;
        }
        return self.ram[address]
    }

    pub fn write(&mut self, address: usize, data: u8) {
        if address < MEMSIZE {
            self.ram[address] = data
        }
    }

    pub fn font_character(&self, character: usize) -> usize {
        return FONT_OFFSET + (character * FONT_BYTES);
    }

    pub fn push(&mut self, data: usize) {
        self.stack.push(data);
    }

    pub fn pop(&mut self) -> usize {
        if self.stack.len() == 0 {
            return 0;
        }
        return self.stack.pop().unwrap();
    }
}
