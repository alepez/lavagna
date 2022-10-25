#![deny(clippy::all)]
#![forbid(unsafe_code)]

use clap::Parser;
use lavagna_collab::CollabOpt;
use lavagna_pixels::{run, Error, Opt};
use rand::Rng;

/// The uncluttered blackboard
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'u', long, value_parser)]
    collab_url: Option<String>,
    #[clap(short = 'i', long, value_parser)]
    pen_id: Option<u32>,
}

fn main() -> Result<(), Error> {
    let mut rng = rand::thread_rng();

    env_logger::init();

    let args = Args::parse();

    let collab = args.collab_url.map(|url| CollabOpt {
        url,
        pen_id: args.pen_id.unwrap_or_else(|| rng.gen::<u32>()).into(),
    });

    let opt = Opt { collab };

    run(opt)
}
