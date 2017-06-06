use std::env;
use std::io;
use std::path::{Path, PathBuf};
use clap::{App, Arg};

use units::{DistPx, DistPxFrac, PX};

/// Attempts to expand a relative filename into a fully-qualified path.
fn expand_filename(p: &str) -> io::Result<PathBuf> {
    let mut np = env::current_dir()?;
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

pub struct Options {
    pub input: PathBuf,
    pub width: DistPx,
    pub height: DistPx,
}

mod arg {
    pub const IMAGE: &str = "image";
    pub const WIDTH: &str = "width";
    pub const HEIGHT: &str = "height";
}

fn build_cmd_line<'a, 'b>() -> App<'a, 'b> {
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

pub fn parse() -> Options {
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