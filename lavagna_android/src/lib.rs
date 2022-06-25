#![deny(clippy::all)]
#![forbid(unsafe_code)]

#[cfg(target_os = "android")]
use lavagna_pixels::run;

#[cfg(target_os = "android")]
#[cfg_attr(
    target_os = "android",
    ndk_glue::main(
        backtrace = "on",
        logger(
            level = "info",
            filter = "debug,wgpu_hal::vulkan=error",
            tag = "lavagna"
        )
    )
)]
fn main() {
    run().unwrap();
}
