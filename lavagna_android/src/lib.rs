#![deny(clippy::all)]

// #[cfg(target_os = "android")]
use lavagna_pixels::{run, Opt};

// #[cfg(target_os = "android")]
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
#[allow(dead_code)]
fn main() {
    let uri = get_collab_uri_from_intent().ok();
    log::info!("uri: {:?}", uri);

    // Collaboration is not yet supported on Android
    let opt = Opt { collab: None };

    run(opt).unwrap();
}

fn get_collab_uri_from_intent() -> Result<String, Box<dyn std::error::Error>> {
    // lavagna+wss://lavagna-server.herokuapp.com/9db2ca48-6f12-4f57-8ae7-0c4bf83d0fb0

    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let env = vm.attach_current_thread()?;
    let class_ctx = env.find_class("android/content/Context")?;

    let intent = env
        .call_method(
            ctx.context().cast(),
            "getIntent",
            "()Landroid/content/Intent;",
            &[],
        )?
        .l()?;

    Ok("smile!".to_string())
}
