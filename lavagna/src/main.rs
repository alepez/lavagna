#![deny(clippy::all)]
#![forbid(unsafe_code)]

use lavagna_pixels::{run, Error, Opt};

fn main() -> Result<(), Error> {
    env_logger::init();

    let opt = Opt {
        collab_url: Some("ws://localhost:3536/example_room".to_string()),
    };

    run(opt)
}
