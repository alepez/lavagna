use clap::Parser;
use super::prepare_collab_options;
use super::Opt;

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
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn options_from_args() -> Opt {
    let args = Args::parse();

    let collab = prepare_collab_options(args.collab_url, args.collab_id);

    Opt {
        collab,
        show_debug_pane: args.show_debug_pane,
        verbose: args.verbose,
        ui: args.ui,
    }
}

