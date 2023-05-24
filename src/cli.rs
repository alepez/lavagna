use crate::CollabOpt;
use crate::Opt;
use clap::Parser;

/// The uncluttered blackboard
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short = 'u', long)]
    collab_url: Option<String>,
    #[clap(short = 'i', long)]
    collab_id: Option<u16>,
    #[clap(long)]
    show_debug_pane: bool,
    #[clap(short = 'v', long)]
    verbose: bool,
    #[clap(long)]
    ui: bool,
    #[clap(long)]
    width: Option<String>,
    #[clap(long)]
    height: Option<String>,
}

/// On native, options are read from command line arguments
pub(crate) fn options_from_args() -> Opt {
    let args = Args::parse();

    // If collab-url is set, then collab-id must be set too. Randomize it if not.
    let collab = if let Some(collab_url) = args.collab_url {
        let collab_id = args.collab_id.unwrap_or_else(rand::random);
        Some(CollabOpt {
            url: collab_url,
            collab_id,
        })
    } else {
        None
    };

    Opt {
        collab,
        show_debug_pane: args.show_debug_pane,
        verbose: args.verbose,
        ui: args.ui,
    }
}
