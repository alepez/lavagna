#![deny(clippy::all)]
#![forbid(unsafe_code)]

use clap::Parser;
use lavagna_pixels::{run, Error, Opt};

/// The uncluttered blackboard
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'u', long, value_parser)]
    collab_url: Option<String>,
    #[clap(short = 'i', long, value_parser)]
    collab_id: Option<u32>,
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let args = Args::parse();

    let opt = Opt {
        collab_url: args.collab_url,
    };

    run(opt)
}
