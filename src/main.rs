#![deny(clippy::all)]
#![forbid(unsafe_code)]
// Disable the Windows Console
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use lavagna::*;

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
}

/// When collab_url is set, collab_id is optional and defaults to a random value
fn prepare_collab_options(collab_url: Option<String>, collab_id: Option<u16>) -> Option<CollabOpt> {
    if let Some(collab_url) = collab_url {
        Some(CollabOpt {
            url: collab_url,
            collab_id: collab_id.unwrap_or_else(rand::random),
        })
    } else {
        None
    }
}

// TODO
/// On wasm, some options are hardcoded, other are read from URL
#[cfg(target_arch = "wasm32")]
fn get_options() -> Opt {
    let collab_url = Some("ws://localhost:3536/lavagna".to_string());
    let collab_id = None;
    let collab = prepare_collab_options(collab_url, collab_id);

    Opt {
        collab,
        show_debug_pane: true,
        verbose: true,
        ui: true,
    }
}

/// On native, options are read from command line arguments
#[cfg(not(target_arch = "wasm32"))]
fn options_from_args() -> Opt {
    let args = Args::parse();

    let collab = prepare_collab_options(args.collab_url, args.collab_id);

    Opt {
        collab,
        show_debug_pane: args.show_debug_pane,
        verbose: args.verbose,
        ui: args.ui,
    }
}

fn main() {
    // This is needed, otherwise bevy logs are not shown on browser console
    #[cfg(target_arch = "wasm32")]
    tracing_wasm::set_as_global_default_with_config(
        tracing_wasm::WASMLayerConfigBuilder::default()
            .set_max_level(tracing::Level::ERROR)
            .build(),
    );

    let options = options_from_args();

    run(options)
}
