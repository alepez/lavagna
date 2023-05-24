#![deny(clippy::all)]
#![forbid(unsafe_code)]
// Disable the Windows Console
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use lavagna::*;

fn main() {
    // This is needed, otherwise bevy logs are not shown on browser console
    #[cfg(target_arch = "wasm32")]
    lavagna::web::setup_log();

    run(lavagna::options())
}
