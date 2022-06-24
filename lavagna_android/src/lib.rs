#![deny(clippy::all)]
#![forbid(unsafe_code)]

use lavagna_core::doc::MutSketch;
use lavagna_core::App;
use lavagna_core::doc::OwnedSketch;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::PhysicalSize;
use winit::event::{TouchPhase, WindowEvent};
use winit::window::{CursorIcon, Window};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn lavagna() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut canvas_size = PhysicalSize::new(640, 480);

    let window = {
        WindowBuilder::new()
            .with_title("lavagna")
            .with_inner_size(canvas_size)
            .with_min_inner_size(canvas_size)
            .build(&event_loop)
            .unwrap()
    };

    window.set_cursor_icon(CursorIcon::Crosshair);

    let mut pixels: Option<Pixels> = None;
    let mut app = App::new();
    let mut frozen_sketch: Option<OwnedSketch> = None;

    event_loop.run(move |event, _, control_flow| {
        if let Event::Resumed = event {
            canvas_size = window.inner_size();
            pixels = resume(&window, canvas_size, frozen_sketch.take());

            // Prevent drawing a line from the last location when resuming
            app.set_pressed(false);
        }

        if let Event::Suspended = event {
            if let Some(mut pixels) = pixels.take() {
                let sketch =
                    MutSketch::new(pixels.get_frame(), canvas_size.width, canvas_size.height);
                frozen_sketch = Some(sketch.to_owned());
            }
        }

        if let Some(pixels) = pixels.as_mut() {
            match event {
                Event::RedrawRequested(_) => {
                    let sketch =
                        MutSketch::new(pixels.get_frame(), canvas_size.width, canvas_size.height);
                    app.update(sketch);

                    if pixels
                        .render()
                        .map_err(|e| error!("pixels.render() failed: {}", e))
                        .is_err()
                    {
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }

                Event::WindowEvent {
                    event: WindowEvent::Touch(touch),
                    ..
                } => {
                    match touch.phase {
                        TouchPhase::Started => {
                            app.set_pressed(true);
                        }
                        TouchPhase::Ended => {
                            app.set_pressed(false);
                        }
                        _ => (),
                    }

                    if let Ok((x, y)) = pixels.window_pos_to_pixel(touch.location.into()) {
                        app.set_cursor_position(x as isize, y as isize);
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    ..
                } => {
                    log::info!("input: {:?}", input);
                }

                _ => (),
            }
        }
    });
}

fn resume(
    window: &Window,
    new_size: PhysicalSize<u32>,
    frozen_sketch: Option<OwnedSketch>,
) -> Option<Pixels> {
    let surface_texture = SurfaceTexture::new(new_size.width, new_size.height, &window);
    let mut pixels = Pixels::new(new_size.width, new_size.height, surface_texture).ok()?;

    pixels.get_frame().fill(0x00);

    let mut new_sketch = MutSketch::new(pixels.get_frame(), new_size.width, new_size.height);

    if let Some(old_sketch) = &frozen_sketch {
        new_sketch.copy_from(&old_sketch.as_sketch());
    }

    Some(pixels)
}

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
    lavagna().unwrap();
}

