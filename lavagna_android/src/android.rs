#![deny(clippy::all)]

use lavagna_collab::{CollabOpt, get_collab_uri_from_intent};
use lavagna_pixels::{run, Opt};
use rand::Rng;

#[cfg_attr(
    target_os = "android",
    ndk_glue::main(
        backtrace = "on",
        logger(
            level = "info",
            filter = "debug,wgpu_hal::vulkan=error,wgpu_core=error",
            tag = "lavagna"
        )
    )
)]
#[allow(dead_code)]
fn main() {
    let mut rng = rand::thread_rng();
    let mut opt = Opt { collab: None };

    let uri = get_collab_uri_from_intent().ok();

    if let Some(uri) = uri {
        if let Some(("lavagna", collab_uri)) = uri.split_once('+') {
            log::info!("uri: {:?}", collab_uri);
            opt.collab = Some(CollabOpt {
                url: collab_uri.to_string(),
                pen_id: rng.gen::<u32>().into(),
            });
        }
    }

    run(opt).unwrap();
}
