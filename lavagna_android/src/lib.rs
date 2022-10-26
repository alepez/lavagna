#![deny(clippy::all)]

// #[cfg(target_os = "android")]
use lavagna_collab::CollabOpt;
use lavagna_pixels::{run, Opt};
use rand::Rng;

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
    let mut rng = rand::thread_rng();
    let mut opt = Opt { collab: None };

    let uri = get_collab_uri_from_intent().ok();

    if let Some(uri) = uri {
        if let Some(("lavagna", collab_uri)) = uri.split_once('+') {
            log::info!("uri: {:?}", collab_uri);
            // TODO Collaboration is not yet supported on Android
            opt.collab = Some(CollabOpt {
                url: collab_uri.to_string(),
                pen_id: rng.gen::<u32>().into(),
            });
        }
    }

    run(opt).unwrap();
}

fn get_collab_uri_from_intent() -> Result<String, Box<dyn std::error::Error>> {
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
    let env = vm.attach_current_thread()?;

    let intent = env.call_method(
        ctx.context().cast(),
        "getIntent",
        "()Landroid/content/Intent;",
        &[],
    )?;

    let uri = env.call_method(intent.l()?, "getData", "()Landroid/net/Uri;", &[])?;
    let uri = env.call_method(uri.l()?, "toString", "()Ljava/lang/String;", &[])?;
    let uri: String = env.get_string(uri.l()?.into())?.into();

    Ok(uri)
}
