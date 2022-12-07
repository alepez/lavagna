#![deny(clippy::all)]
#![forbid(unsafe_code)]

use clap::Parser;
use rand::Rng;

use lavagna_collab::{CollabOpt, CollabUri, CollabUriProvider};
use lavagna_pixels::{run, Error, Opt};

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

    let collab = args.collab_url.map(|uri| CollabOpt {
        pen_id: args.pen_id.unwrap_or_else(|| rng.gen::<u32>()).into(),
        uri_provider: Some(Box::new(StringUriProvider(uri))),
    });

    let opt = Opt { collab };

    run(opt)
}

struct StringUriProvider(String);

impl CollabUriProvider for StringUriProvider {
    fn uri(&self) -> Option<CollabUri> {
        Some(CollabUri::new(self.0.clone()))
    }
}
