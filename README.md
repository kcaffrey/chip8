# chip8 
[![Build Status](https://travis-ci.org/kcaffrey/chip8.svg?branch=master)](https://travis-ci.org/kcaffrey/chip8)
[![codecov](https://codecov.io/gh/kcaffrey/chip8/branch/master/graph/badge.svg)](https://codecov.io/gh/kcaffrey/chip8)

[CHIP-8](https://en.wikipedia.org/wiki/CHIP-8) emulator in Rust. 
This is a project I'm using to learn Rust. 

![Tetris Screenshot](/screenshots/screenshot_tetris.gif?raw=true "Tetris Screenshot")

## Requirements
- Rust 2018 edition
  - First install [rustup](https://www.rust-lang.org/en-US/install.html).
  - Install the beta rust compiler: `rustup install beta`
  - Tell rust to use the beta compiler for this project: `cd chip8 && rustup override set beta`
- SDL2
  - macOS: `brew install sdl2`
  - For other platforms, follow the instructions [here](https://github.com/Rust-SDL2/rust-sdl2).

## Usage
```
cargo run --release -- [PATH_TO_ROM]
```

### Controls
The CHIP-8 had a 16 button keypad, which has been mapped to the following keys:
<pre>
1 2 3 4
q w e r
a s d f
z x c v
</pre>

ROMs vary in what controls they use. If the ROM you are using did not come with instructions, some experimentation
may be required.

### Clock Speed
By default, the emulator runs with a clock speed of 1200 Hz which should result in reasonable performance on most ROMS. Use the `--clock-speed` command-line argument to override the clock speed to a different value. Many older ROMs 
work best with slower clock speeds  (around 500-1000 Hz), while some of the programs created in OCTO work better with 
clock speeds of around 60 KHz.  For example, 
[Cave Explorer](https://github.com/JohnEarnest/Octo/blob/gh-pages/examples/caveexplorer.8o) runs a lot smoother with faster clock speeds, and uses the built-in timer to control the frame rate.

### Finding ROMs
Any valid CHIP-8 ROM should work with this project. ROMs can be found to freely download at:
- [Zophar's Chip-8 Game Pack](https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html)
- [CHIP-8 Collection on Github](https://github.com/dmatlack/chip8/tree/master/roms)
- [Octo](http://johnearnest.github.io/Octo/), a web-based CHIP-8 assembler and interpreter with a collection of premade programs. Download compiled ROMs using the "Binary Tools" button or by using the [command-line mode](https://github.com/JohnEarnest/Octo#command-line-mode).

## Quirks
Due to undocumented behavior in the original devices, CHIP-8 emulators have slightly different behavior.
Some of the choices around quirks are listed here:
- When drawing a sprite, if the sprite would go beyond the edge of the screen it is wrapped around the edge.
- When loading and storing registers, the value of `I` (the address register) is modified
- Instructions which set the carry flag (in `VF`), do so last. If `VF` is used as an operand, its value will be overwritten with the carry flag.
- The two shift instructions operate on `VY` and store the result in `VX` as indicated in [Mastering CHIP-8](http://mattmik.com/files/chip8/mastering/chip8.html) and implemented by Octo, despite Wikipedia and [Cowgod](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#8xy6) documenting otherwise.

## Resources
- [CHIP-8 Wikipedia page](https://en.wikipedia.org/wiki/CHIP-8)
- [Mastering CHIP-8](http://mattmik.com/files/chip8/mastering/chip8.html)
- [Cowgod's Chip-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [Octo](https://github.com/JohnEarnest/Octo), a CHIP-8 assembler and browser-based virtual machine
