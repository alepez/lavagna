#![deny(clippy::all)]
#![forbid(unsafe_code)]
// Disable the Windows Console
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;

use lavagna::*;

/// The uncluttered blackboard
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[cfg(not(target_arch = "wasm32"))]
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

/// When collab_url is set, collab_id is optional and defaults to a random value
fn prepare_collab_options(collab_url: Option<String>, collab_id: Option<u16>) -> Option<CollabOpt> {
    collab_url.map(|collab_url| CollabOpt {
        url: collab_url,
        collab_id: collab_id.unwrap_or_else(rand::random),
    })
}

#[allow(dead_code)]
fn parse_request(request: Option<String>) -> (Option<String>, Option<u16>) {
    let Some(request) = request else { return (None, None) };

    if request.is_empty() {
        return (None, None);
    };

    let mut collab_url = None;
    let mut collab_id = None;
    for param in request.split('&') {
        let mut param = param.split('=');
        let Some(key) = param.next() else { return (None, None) };
        let Some(value) = param.next() else { return (None, None) };
        match key {
            "collab-url" => collab_url = Some(value.to_owned()),
            "collab-id" => collab_id = value.parse().ok(),
            _ => (),
        }
    }
    (collab_url, collab_id)
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use super::*;

    /// On wasm, some options are hardcoded, other are read from URL
    pub(super) fn options() -> Opt {
        let request = decode_request(web_sys::window().unwrap());
        let (collab_url, collab_id) = parse_request(request);
        let collab = prepare_collab_options(collab_url, collab_id);

        Opt {
            collab,
            show_debug_pane: false,
            verbose: false,
            ui: true,
        }
    }

    fn decode_request(window: web_sys::Window) -> Option<std::string::String> {
        Some(
            window
                .location()
                .search()
                .ok()?
                .trim_start_matches('?')
                .to_owned(),
        )
    }
}

#[cfg(target_arch = "wasm32")]
fn options() -> Opt {
    wasm::options()
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

#[cfg(not(target_arch = "wasm32"))]
fn options() -> Opt {
    options_from_args()
}

fn main() {
    // This is needed, otherwise bevy logs are not shown on browser console
    #[cfg(target_arch = "wasm32")]
    tracing_wasm::set_as_global_default_with_config(
        tracing_wasm::WASMLayerConfigBuilder::default()
            .set_max_level(tracing::Level::ERROR)
            .build(),
    );

    let options = options();

    run(options)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_request() {
        let request = "collab-url=ws://127.0.0.1:3536/lavagna&collab-id=6";
        let (collab_url, collab_id) = parse_request(Some(request.to_string()));
        assert_eq!(collab_url, Some("ws://127.0.0.1:3536/lavagna".to_string()));
        assert_eq!(collab_id, Some(6u16));
    }
}
