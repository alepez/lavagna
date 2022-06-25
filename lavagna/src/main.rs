#![deny(clippy::all)]
#![forbid(unsafe_code)]

use lavagna_pixels::{run, Error};

fn main() -> Result<(), Error> {
    env_logger::init();
    run()
}
