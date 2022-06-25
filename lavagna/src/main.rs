#![deny(clippy::all)]
#![forbid(unsafe_code)]

use clap::Parser;
use lavagna_pixels::{run, Error, Opt};

/// The uncluttered blackboard
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, value_parser)]
    collab_url: Option<String>,
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let args = Args::parse();

    let opt = Opt {
        collab_url: args.collab_url,
    };

    run(opt)
}
