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
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let env = vm.attach_current_thread()?;

    let intent = dbg!(env.call_method(
        ctx.context().cast(),
        "getIntent",
        "()Landroid/content/Intent;",
        &[],
    ))?;

    let uri = dbg!(env.call_method(dbg!(intent.l())?, "getData", "()Landroid/net/Uri;", &[]))?;
    let uri = dbg!(env.call_method(dbg!(uri.l())?, "toString", "()Ljava/lang/String;", &[]))?;
    let uri: String = env.get_string(uri.l()?.into())?.into();

    Ok(uri)
}
