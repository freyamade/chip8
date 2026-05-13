extern crate sdl2;

use rustop::opts;
use sdl2::event::Event;
// use sdl2::AudioSubsystem;
use sdl2::keyboard::Scancode;
use std::fs::File;
use std::io::{BufReader, Read};
use std::time::Duration;

mod memory;
mod display;
mod registers;

const PIXEL_SCALE: usize = 16; // Draw each pixel in the display as a group of this size

fn get_scancode_for_digit(key: u8) -> Scancode {
    // 1 2 3 4 // 1 2 3 C
    // Q W E R // 4 5 6 D
    // A S D F // 7 8 9 E
    // Z X C V // A 0 B F
    return match key {
        0x0 => Scancode::X,
        0x1 => Scancode::Num1,
        0x2 => Scancode::Num2,
        0x3 => Scancode::Num3,
        0x4 => Scancode::Q,
        0x5 => Scancode::W,
        0x6 => Scancode::E,
        0x7 => Scancode::A,
        0x8 => Scancode::S,
        0x9 => Scancode::D,
        0xA => Scancode::Z,
        0xB => Scancode::C,
        0xC => Scancode::Num4,
        0xD => Scancode::R,
        0xE => Scancode::F,
        0xF => Scancode::V,
        _ => Scancode::Space,
    }
}

fn get_digit_for_scancode(key: Scancode) -> u8 {
    // 1 2 3 4 // 1 2 3 C
    // Q W E R // 4 5 6 D
    // A S D F // 7 8 9 E
    // Z X C V // A 0 B F
    return match key {
        Scancode::X => 0x0,
        Scancode::Num1 => 0x1,
        Scancode::Num2 => 0x2,
        Scancode::Num3 => 0x3,
        Scancode::Q => 0x4,
        Scancode::W => 0x5,
        Scancode::E => 0x6,
        Scancode::A => 0x7,
        Scancode::S => 0x8,
        Scancode::D => 0x9,
        Scancode::Z => 0xA,
        Scancode::C => 0xB,
        Scancode::Num4 => 0xC,
        Scancode::R => 0xD,
        Scancode::F => 0xE,
        Scancode::V => 0xF,
        _ => 0xFF,
    }
}

pub fn main() {
    // Setup argument parsing
    let (args, _) = opts! {
        synopsis "A simple chip8 emulator for learning the basics of emulation";
        opt new_shift:bool=false, desc: "Use the new format for 0x8XY6 and 0x8XYE instructions. Defaults false.";
        opt step:bool=false, desc: "Allow for stepping through of execution. Defaults false.";
        param filename:String, desc: "Name of file to load into the emulator.";
    }.parse_or_exit();

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

    let buf = BufReader::new(File::open(args.filename).unwrap());
    let mut pointer = program_counter;
    for byte_or_err in buf.bytes() {
        let byte = byte_or_err.unwrap();
        memory.write(pointer, byte);
        pointer += 1;
    }

    /* 
        Loop; 
            Delay/Sound Timers need to decrement at 60Hz
            FDE loop runs at 600 cycles / s
    */
    let cycles_per_second = 600;
    let cycles_for_60fps = cycles_per_second / 60;
    let mut cycle = 0;
    // every cycle, increment this number and if its divisble by 10 decrement the delay and sound timers

    'mainloop: loop {
        // Fetch - Instructions are 2 bytes in length
        let first_byte: u16 = u16::from(memory.read(program_counter));
        let second_byte = u16::from(memory.read(program_counter + 1));
        let command = first_byte.checked_shl(8).unwrap_or(0) + second_byte;
        // Increment the counter here
        program_counter += 2;
        
        // Check for exit events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { scancode: Some(Scancode::Escape), .. } => {
                    break 'mainloop
                },
                _ => {}
            }
        }

        // Decode & Execute
        // Match command based on initial 4 bits
        let nibbles: [u8; 4] = [
            ((command & 0xF000) >> 12).try_into().unwrap(),
            ((command & 0x0F00) >> 8).try_into().unwrap(),
            ((command & 0x00F0) >> 4).try_into().unwrap(),
            ((command & 0x000F)).try_into().unwrap(),
        ];

        // Keep track of if we find an unknown command
        let mut known = false;
        match nibbles[0] {
            0x0 => {
                match nibbles[2] {
                    0xE => {
                        match nibbles[3] {
                            0x0 => {
                                // 0x00E0 -> clear screen
                                display.clear();
                                known = true;
                            }
                            0xE => {
                                // 0x00EE -> return from subroutine
                                program_counter = memory.pop();
                                known = true;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            0x1 => {
                // 1NNN
                // Jump command, jump to the rest of the command value
                let target = command & 0x0FFF;
                program_counter = target as usize;
                known = true;
            }
            0x2 => {
                // 2NNN
                // Call command, push current PC to stack and then jump to NNN
                memory.push(program_counter);
                let target = command & 0x0FFF;
                program_counter = target as usize;
                known = true;
            }
            0x3 => {
                // 3XNN
                // Skip next instruction if value in vX == NN
                let register = nibbles[1];
                let value = registers.get(register);
                let compare_to: u8 = (command & 0x00FF).try_into().unwrap();
                if value == compare_to {
                    program_counter += 2;
                }
                known = true;
            }
            0x4 => {
                // 0x4XNN
                // Skip next instruction if value in vX != NN
                let register = nibbles[1];
                let value = registers.get(register);
                let compare_to: u8 = (command & 0x00FF).try_into().unwrap();
                if value != compare_to {
                    program_counter += 2;
                }
                known = true;
            }
            0x5 => {
                // 0x5XY0
                // Skip next instruction if value in vX == value in vY
                let x_register = nibbles[1];
                let x_value = registers.get(x_register);
                let y_register = nibbles[2];
                let y_value = registers.get(y_register);
                if x_value == y_value {
                    program_counter += 2;
                }
                known = true;
            }
            0x6 => {
                // 6XNN
                // Set register (pointed to by register X) to value NN
                let register = nibbles[1];
                let value = command & 0x00FF;
                registers.set(register, value as u8);
                known = true;
            }
            0x7 => {
                // 7XNN
                // Add value NN to register (pointed to by register X)
                let register = nibbles[1];
                let value = command & 0x00FF;

                registers.add(register, value as u8);
                known = true;
            }
            0x8 => {
                // 0x8XY?
                // Logical and arithmetic instructions based on value of nibbles[3]
                let x_register = nibbles[1];
                let mut x_value = registers.get(x_register);
                let y_register = nibbles[2];
                let y_value = registers.get(y_register);
                

                // check nibbles[3] and do what needs to be done
                match nibbles[3] {
                    0x0 => {
                        // 0x8XY0
                        // Set register vX to value of register vY
                        known = true;
                        registers.set(x_register, y_value);
                    }
                    0x1 => {
                        // 0x8XY1
                        // Set register vX to its value OR the value of register vY
                        known = true;
                        registers.set(x_register, x_value | y_value);
                    }
                    0x2 => {
                        // 0x8XY2
                        // Set register vX to its value AND the value of register vY
                        known = true;
                        registers.set(x_register, x_value & y_value);
                    }
                    0x3 => {
                        // 0x8XY3
                        // Set register vX to its value XOR the value of register vY
                        known = true;
                        registers.set(x_register, x_value ^ y_value);
                    }
                    0x4 => {
                        // 0x8XY4
                        // Set register vX to its value + the value of register vY
                        // Set the carry flag in vF to 1 if this value overflows
                        known = true;
                        let new_value = x_value.wrapping_add(y_value);
                        registers.set(x_register, new_value);
                        // If it overflowed that means the new wrapped x has to be <= its old value
                        registers.vf = if new_value <= x_value {1} else {0}
                    }
                    0x5 => {
                        // 0x8XY5
                        // Set register vX to its value - the value of register vY
                        // Set the carry flag in vF to 0 if this value underflows, or 1 if it doesn't
                        known = true;
                        registers.set(x_register, x_value.wrapping_sub(y_value));
                        // If it overflowed that means the new wrapped x has to be <= its old value
                        registers.vf = if x_value >= y_value {1} else {0}
                    }
                    0x6 => {
                        // 0x8XY6
                        // Shift the value in vX one bit to the right (optionally in old architecture it first copies the value of vY into vX)
                        // Set carry flag to shifted value
                        if !args.new_shift {
                            registers.set(x_register, y_value);
                            x_value = y_value;
                        }
                        // Grab the least significant bit of x_value
                        let removed_bit = x_value & 0x1;
                        registers.set(x_register, x_value.unbounded_shr(1));
                        registers.vf = removed_bit;
                        known = true;
                    }
                    0x7 => {
                        // 0x8XY7
                        // Set register vX to the value of register vY - the value of register vX
                        // Set the carry flag in vF to 0 if this value underflows, or 1 if it doesn't
                        known = true;
                        registers.set(x_register, y_value.wrapping_sub(x_value));
                        // If it overflowed that means the new wrapped x has to be <= its old value
                        registers.vf = if y_value >= x_value {1} else {0}
                    }
                    0xE => {
                        // 0x8XYE
                        // Shift the value in vX one bit to the left (optionally in old architecture it first copies the value of vY into vX)
                        // Set carry flag to shifted value
                        if !args.new_shift {
                            registers.set(x_register, y_value);
                            x_value = y_value;
                        }
                        // Grab the least significant bit of x_value
                        let removed_bit = (x_value & 0b10000000) >> 7;
                        registers.set(x_register, x_value.unbounded_shl(1));
                        registers.vf = removed_bit;
                        known = true;
                    }
                    _ => {}
                }
            }
            0x9 => {
                // 0x9XY0
                // Skip next instruction if value in vX != value in vY
                let x_register = nibbles[1];
                let x_value = registers.get(x_register);
                let y_register = nibbles[2];
                let y_value = registers.get(y_register);
                if x_value != y_value {
                    program_counter += 2;
                }
                known = true;
            }
            0xA => {
                // ANNN
                // Set the value of the index register to NNN
                let value = command & 0x0FFF;
                registers.i = value.into();
                known = true;
            }
            0xB => {
                // BNNN
                // Jump the PC to the value of NNN + the value in v0
                known = true;
                let value = command & 0x0FFF;
                let target: u16 = value + (registers.v0 as u16);
                program_counter = target.into();
            }
            0xC => {
                // CXNN
                // Generate random 8bit integer, AND it with NN and put the value into vX
                known = true;
                let random_number: u8 = rand::random();
                let x_register = nibbles[1];
                let offset = (command & 0x00FF) as u8;
                registers.set(x_register, random_number & offset);
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
                let sprite_height: usize = nibbles[3].into();

                let mut sprite_bytes: Vec<u8> = vec![];
                for i in sprite_index..(sprite_index + sprite_height) {
                    sprite_bytes.push(memory.read(i as usize));
                }

                // Tell the display to draw the thing and retrieve info on whether a bit was flipped
                let flipped = display.draw(x_coord, y_coord, sprite_bytes);
                // Lastly store that info in the VF register
                registers.vf = if flipped {1} else {0};
                known = true;
            }
            0xE => {
                // 0xEX??
                // Skip instructions based on key being pressed in this current loop
                let x_register = nibbles[1];
                match [nibbles[2], nibbles[3]] {
                    [0x9, 0xE] => {
                        // 0xEX9E
                        // Skip if the key corresponding to the value in vX is currently pressed
                        known = true;
                        let x_value = registers.get(x_register);
                        if event_pump.keyboard_state().is_scancode_pressed(get_scancode_for_digit(x_value)) {
                            program_counter += 2;
                        }
                    }
                    [0xA, 0x1] => {
                        // 0xEXA1
                        // Skip if the key corresponding to the value in vX is not currently pressed
                        known = true;
                        let x_value = registers.get(x_register);
                        if !event_pump.keyboard_state().is_scancode_pressed(get_scancode_for_digit(x_value)) {
                            program_counter += 2;
                        }
                    }
                    _ => {}
                }
            }
            0xF => {
                // 0xFX??
                // Interact with various other timers/registers/etc
                let x_register = nibbles[1];
                let x_value = registers.get(x_register);
                match [nibbles[2], nibbles[3]] {
                    [0x0, 0x7] => {
                        // 0xFX07
                        // Set the value of vX equal to the current value of the delay timer
                        known = true;
                        registers.set(x_register, delay);
                    }
                    [0x1, 0x5] => {
                        // 0xFX15
                        // Set the delay timer to be the value in register vX
                        known = true;
                        delay = x_value;
                    }
                    [0x1, 0x8] => {
                        // 0xFX18
                        // Set the sound timer to be the value in register vX
                        known = true;
                        sound = x_value;
                    }
                    [0x1, 0xE] => {
                        // 0xFX1E
                        // Add the value in vX to the current index register
                        known = true;
                        registers.i = registers.i.wrapping_add(x_value.into());
                    }
                    [0x0, 0xA] => {
                        // 0xFX0A
                        // Block until a key is pressed
                        // when a key is pressed, put its value in register x
                        known = true;

                        let mut found_valid_key = false;
                        for (scancode, pressed) in event_pump.keyboard_state().scancodes() {
                            if !pressed {
                                continue;
                            }

                            let digit = get_digit_for_scancode(scancode);
                            if digit != 0xFF {
                                registers.set(x_register, digit);
                                found_valid_key = true;
                            }
                        }
                        
                        if !found_valid_key  {
                            // "Block" by re-reading this instruction
                            program_counter -= 2;
                        }
                    }
                    [0x2, 0x9] => {
                        // 0xFX29
                        // Point the index register to the memory space where the font character for the character in vX is contained
                        known = true;
                        registers.i = memory.font_character(x_value.into()).try_into().unwrap();
                    }
                    [0x3, 0x3] => {
                        // 0xFX33 
                        // Take the value of vX, convert it into decimal digits for hundreds, tens, and units
                        // And store each in memory at the current index pointer, hundreds, then tens, then units
                        known = true;
                        let hundreds = x_value / 100;
                        let tens = (x_value % 100) / 10;
                        let units = x_value % 10;
                        memory.write(registers.i, hundreds);
                        memory.write(registers.i + 1, tens);
                        memory.write(registers.i + 2, units);
                    }
                    [0x5, 0x5] => {
                        // 0xFX55 
                        // Store the values of registers 0..=X in memory in subsequent addresses starting at I
                        known = true;
                        for x in 0..=x_register {
                            let value = registers.get(x);
                            let pointer = registers.i + usize::from(x);
                            memory.write(pointer, value);
                        }
                    }
                    [0x6, 0x5] => {
                        // 0xFX65 
                        // Retrieve values from memory starting at I and put them into subsequent registers starting at v0
                        known = true;
                        for x in 0..=x_register {
                            let pointer = registers.i + usize::from(x);
                            let value = memory.read(pointer);
                            registers.set(x, value);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        if !known {
            println!("found unknown command {command:#x}");
        }
        
        // Decrement timers within our 60fps loop
        if cycle % cycles_for_60fps == 0 {
            if sound > 0 {
                // TODO
                sound -= 1;
            }
            if delay > 0 {
                delay -= 1;
            }
        }
        cycle = (cycle + 1) % cycles_per_second;

        if args.step {
            loop {
                let event = event_pump.wait_event();
                if event.is_keyboard() || event.is_window() {
                    break;
                }
            }
        }
        else {
            // 60fps stuff
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / cycles_per_second));
        }
    }
}