#[macro_use]
extern crate bitflags;
extern crate sdl2;

use std::env;

mod gb;

fn main() {
    let cartridge_filepath = env::args().nth(1).unwrap();

    let mut gb = gb::Gameboy::new(&cartridge_filepath);
    gb.reset();
    gb.run();
}
