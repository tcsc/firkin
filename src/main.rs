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

mod units;
mod image;
mod distort;

use clap::{App, Arg};
use std::path::{Path, PathBuf};
use std::io;
use units::{DistPx, DistPxFrac, PX};
use image::Image;

fn expand_filename(p: &str) -> io::Result<PathBuf> {
    let mut np = std::env::current_dir()?;
    np.push(PathBuf::from(p));
    Ok(np)
}

#[cfg(test)]
mod test_expand_filename {
    #[test]
    fn rooted_paths_are_not_expanded() {
        let path = "/path/to/file";
        let r = super::expand_filename(path).unwrap();
        assert_eq!(r.to_str().unwrap(), path);
    }
}

struct Options {
    input: PathBuf,
    width: DistPx,
    height: DistPx,
}

mod arg {
    pub const IMAGE: &str = "image";
    pub const WIDTH: &str = "width";
    pub const HEIGHT: &str = "height";
}

fn build_cmd_line<'a, 'b>() -> App<'a, 'b> {
    use arg;

    App::new("Firkin barrel distortion corrector")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name(arg::IMAGE)
                 .long("image")
                 .short("i")
                 .help("The input file")
                 .value_name("FILE")
                 .takes_value(true)
                 .required(true))
        .arg(Arg::with_name(arg::WIDTH)
                 .long("width")
                 .short("w")
                 .help("width of the image")
                 .takes_value(true)
                 .value_name("INT")
                 .default_value("1900"))
        .arg(Arg::with_name(arg::HEIGHT)
                 .long("height")
                 .short("h")
                 .help("Height of the image")
                 .takes_value(true)
                 .value_name("INT")
                 .default_value("800"))
}

fn parse_cmd_line() -> Options {
    use arg;

    let m = build_cmd_line().get_matches();

    let pixel_value =
        |n| value_t!(m, n, isize).unwrap_or_else(|e| e.exit()) * PX;

    let img = m.value_of(arg::IMAGE)
        .and_then(|p| expand_filename(p).ok())
        .unwrap();

    Options {
        input: img,
        width: pixel_value(arg::WIDTH),
        height: pixel_value(arg::HEIGHT),
    }
}

fn main() {
    env_logger::init().unwrap();

    let f = parse_cmd_line();
    debug!("Input file is: {:?} @ {} x {}", f.input, f.width, f.height);

    let i = image::MemoryMappedImage::<i16>::map_file(f.input.as_path(),
                                                      f.width,
                                                      f.height);
}
