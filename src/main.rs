extern crate sdl2;

use sdl2::AudioSubsystem;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Duration;
use std::env;

mod memory;
mod display;
mod registers;

const PIXEL_SCALE: usize = 8; // Draw each pixel in the display as a group of 4

pub fn main() {
    // Set up required SDL stuff
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    // let audio_subsystem: AudioSubsystem = sdl_context.audio().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // Set up required components
    let mut display = display::Display::new(PIXEL_SCALE, &video_subsystem);
    let mut memory = memory::Memory::new();
    let mut registers = registers::Registers::new();
    let mut program_counter: usize = 0x200; // program code should start at memory address 200
    let mut delay: u8 = 0;
    let mut sound: u8 = 0;

    // Load ROM into memory
    if env::args().len() != 2 {
        panic!("please supply a .ch8 file path to open");
    }
    let filename = env::args().last().unwrap();
    let buf = BufReader::new(File::open(filename).unwrap());
    let mut pointer = program_counter;
    for byte_or_err in buf.bytes() {
        let byte = byte_or_err.unwrap();
        memory.write(pointer, byte);
        pointer += 1;
    }

    /* 
        Loop; 
            Delay/Sound Timers need to decrement at 60Hz
            FDE loop should handle 700 commands a second
    */

    'mainloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'mainloop;
                },
                _ => {}
            }
        }

        // Fetch - Instructions are 2 bytes in length
        let first_byte: u16 = u16::from(memory.read(program_counter));
        let second_byte = u16::from(memory.read(program_counter + 1));
        let command = first_byte.checked_shl(8).unwrap_or(0) + second_byte;
        // Increment the counter here
        program_counter += 2;
        
        // Decode & Execute
        // Match command based on initial 4 bits
        let nibbles: [u8; 4] = [
            ((command & 0xF000) >> 12).try_into().unwrap(),
            ((command & 0x0F00) >> 8).try_into().unwrap(),
            ((command & 0x00F0) >> 4).try_into().unwrap(),
            ((command & 0x000F)).try_into().unwrap(),
        ];

        match nibbles[0] {
            0x0 => {
                // 00E0 -> clear display
                if command == 0x00E0 {
                    display.clear();
                }
            }
            0x1 => {
                // 1NNN
                // Jump command, jump to the rest of the command value
                let target = command & 0x0FFF;
                program_counter = target as usize;
            }
            0x6 => {
                // 6XNN
                // Set register (pointed to by register X) to value NN
                let register = nibbles[1];
                let value = command & 0x00FF;
                registers.set(register, value as u8);
            }
            0x7 => {
                // 7XNN
                // Add value NN to register (pointed to by register X)
                let register = nibbles[1];
                let value = command & 0x00FF;
                registers.add(register, value as u8);
            }
            0xA => {
                // ANNN
                // Set the value of the index register to NNN
                let value = command & 0x0FFF;
                registers.i = value;
            }
            0xD => {
                // DXYN
                // Draw sprite and render screen
                // Sprite is N pixels tall, located in memory at the value of the index register, at the horizontal coordinate contained in vx, and the vertical coordinate contained in vy
                let x_register = nibbles[1];
                let x_coord = registers.get(x_register as u8);
                let y_register = nibbles[2];
                let y_coord = registers.get(y_register as u8);

                // Need to then retrieve the sprite by getting all the bytes from I to I+N
                let sprite_index = registers.i;
                let sprite_height: u16 = nibbles[3].into();

                let mut sprite_bytes: Vec<u8> = vec![];
                for i in sprite_index..(sprite_index + sprite_height) {
                    sprite_bytes.push(memory.read(i as usize));
                }

                // Tell the display to draw the thing and retrieve info on whether a bit was flipped
                let flipped = display.draw(x_coord, y_coord, sprite_bytes);
                // Lastly store that info in the VF register
                registers.vf = if flipped {1} else {0};

            }
            _ => {}
        }

        // 60fps
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}