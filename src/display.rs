use sdl2::{pixels::Color, rect::Point};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

const BLACK: Color = Color::RGB(0x18, 0x19, 0x1c);
const WHITE: Color = Color::RGB(0xee, 0xee, 0xee);

// Make a constant array of bitmaps to extract each bit of a sprite byte to draw horizontally
const BITMAPS: [u8; 8] = [
    0b10000000,
    0b01000000,
    0b00100000,
    0b00010000,
    0b00001000,
    0b00000100,
    0b00000010,
    0b00000001,
];

pub struct Display {
    pixel_scale: usize,  // track the scale of the pixels for when we draw them to the screen
    screen: [[bool; HEIGHT]; WIDTH],
    canvas: sdl2::render::WindowCanvas,
}

/*
The details of the drawing instruction DXYN are found below, but in short, it is used to draw a “sprite” on the screen. 
Each sprite consists of 8-bit bytes, where each bit corresponds to a horizontal pixel; sprites are between 1 and 15 bytes tall. 
They’re drawn to the screen by treating all 0 bits as transparent, 
and all the 1 bits will “flip” the pixels in the locations of the screen that it’s drawn to. (You might recognize this as logical XOR.)
 */

impl Display {
    pub fn new(pixel_scale: usize, video_subsystem: &sdl2::VideoSubsystem) -> Display {
        // create a window canvas from the subsystem and information we've been given
        let window = video_subsystem.window(
            "chip8", 
            (WIDTH * pixel_scale) as u32, 
            (HEIGHT * pixel_scale) as u32
        ).position_centered().build().unwrap();

        let canvas = window.into_canvas().build().unwrap();

        // flag indicates whether we draw to that pixel (white) or leave it blank (black)
        return Display{ pixel_scale: pixel_scale, screen: [[false; HEIGHT]; WIDTH], canvas: canvas };
    }

    pub fn clear(&mut self) {
        self.screen = [[false; HEIGHT]; WIDTH];
        self.render()
    }

    pub fn draw(&mut self, x_coordinate: u8, y_coordinate: u8, bytes: Vec<u8>) -> bool {
        // Starting coordinates can wrap around the screen
        let x_start: usize = x_coordinate as usize % WIDTH;
        let y_start: usize = y_coordinate as usize % HEIGHT;
        let mut pixel_turned_off = false; // only set this to true if a pixel is turned off by this

        let mut y = y_start;
        let mut x;

        // Each byte in bytes is vertical information, each bit in the byte is horizontal information from left->right most->least significant
        for byte in bytes {
            // Each iter of this loop is a vertical increment, so set x back to x_start
            x = x_start;
            // Iterate over each bit in the current byte
            for bitmap in BITMAPS {
                // Retrieve the mapped bit
                let bit = byte & bitmap;
                if bit > 0 {
                    // If the bit is 1, flip the x/y coordinate
                    if self.screen[x][y] {
                        // We're turning a bit off, so mark the flag accordingly
                        pixel_turned_off = true;
                    }
                    self.screen[x][y] = !self.screen[x][y];
                }
                // Increment x and if it's gone off screen we can stop
                x += 1;
                if x >= WIDTH {
                    break;
                }
            }
            // Increment y and break if needed
            y += 1;
            if y >= HEIGHT {
                break;
            }

        }
        
        // Always re-render the screen
        self.render();
        
        // Update the register accordingly
        return pixel_turned_off;
    }

    pub fn render(&mut self) {
        // Draw the display to the supplied screen (skip render scaling for now)
        self.canvas.clear();

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if self.screen[x][y] {
                    // render canvas position x,y in white
                    self.canvas.set_draw_color(WHITE);   
                } 
                else {
                    self.canvas.set_draw_color(BLACK);
                }
                // Each x,y point is actually a square of size pixel_scale
                for i in 0..self.pixel_scale {
                    for j in 0..self.pixel_scale {
                        // Scale up the output based on the chosen pixel scale
                        let x_target = (x * self.pixel_scale) + i;
                        let y_target = (y * self.pixel_scale) + j;
                        self.canvas.draw_point(Point::new(x_target as i32, y_target as i32)).unwrap();
                    }
                }
            }
        }
        self.canvas.present();
    }
}
