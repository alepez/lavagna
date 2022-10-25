#![deny(clippy::all)]
#![forbid(unsafe_code)]

use clap::Parser;
use lavagna_collab::CollabOpt;
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

    let collab = args.collab_url.map(|url| CollabOpt {
        url,
        id: args.collab_id.unwrap_or(0).into(),
    });

    let opt = Opt { collab };

    run(opt)
}
