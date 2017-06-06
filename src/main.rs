#[macro_use]
extern crate clap;
#[macro_use]
extern crate dimensioned as dim;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate memmap;
extern crate num;

#[cfg(test)]
extern crate tempfile;
#[cfg(test)]
extern crate byteorder;

mod cli;
mod units;
mod image;
mod distort;

use image::Image;


fn main() {
    env_logger::init().unwrap();

    let f = cli::parse();
    debug!("Input file is: {:?} @ {} x {}", f.input, f.width, f.height);

    let i = image::MemoryMappedImage::<i16>::map_file(f.input.as_path(),
                                                      f.width,
                                                      f.height);
}
