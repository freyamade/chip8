# chip8
Exploring emulation by building a chip8 interpreter in rust, as everyone recommends you do first.

Followed this guide; https://tobiasvl.github.io/blog/write-a-chip-8-emulator/

Game roms obtained from https://github.com/badlogic/chip8 but the vbrix rom appears to have an error so I've included the source;
- line 152 in InitBall has `random vb,BottomLine-TopLine-1`
    - BottomLine is (decimal) 31
    - TopLine is 0
- The assembled bytecode is `cb20`, which would imply that 31-0-1 == 32 (32d == 20x)
- 30 in hex is `1e` so the bytecode *should* be `cb1e`

The issue is, this random command is used to initialise a location of the ball vertically, and having it be outside the bounds of the screen will break the bouncing checking

I've edited the hex of the rom to `cb1e` which makes the program work, maybe I need to make an assembler for the language but just reading the source makes me think something went wrong.
Or maybe I have it wrong..
But every version of the rom I can find has `cb20` which seems to be wrong based on the sourcecode.
