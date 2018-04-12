#[macro_use]
extern crate bitflags;
extern crate sdl2;

mod gb;

use std::env;

use gb::Gameboy;

fn main() {
    let cartridge_filepath = env::args().nth(1).unwrap();

    let mut gb = Gameboy::new(&cartridge_filepath);
    gb.reset();

    loop {
        gb.run();
    }
}
